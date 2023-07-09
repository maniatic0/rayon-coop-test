use std::ffi::c_void;
use std::mem::size_of;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{hint, thread};

use rayon;
use windows_sys::Win32::System::Threading::{WaitOnAddress, WakeByAddressAll, INFINITE};

use crate::rayon_utils;

/// Max spinning allowed before just waiting
const MAX_SPIN_COUNT: usize = 10_000;

/// Max idle allowed before sleeping
const MAX_IDLE_COUNT: usize = 1_000;

/// Time out for sleeping rayon thread
const RAYON_IDLE_SLEEP_TIMEOUT_MS: u32 = 10; // We assume this is longer than thread::yield_now()

/// Value when the lock is locked
const LOCKED_LOCK: AtomicBool = AtomicBool::new(true);

/// Lock for quick grab and release workloads
/// 
/// Design based on https://rigtorp.se/spinlock/
/// 
/// With help from https://marabos.nl/atomics/
#[derive(Default, Debug)]
pub struct FastLockImpl {
    locked: AtomicBool,
}

impl FastLockImpl {
    /// Locks this lock assuming this is happening in a rayon thread
    pub unsafe fn lock_rayon(&self) {
        assert!(
            rayon_utils::is_inside_rayon(),
            "Should only run in rayon threads!"
        );

        // Spin a little before giving up
        let mut fail_count = 0;
        while fail_count < MAX_SPIN_COUNT {
            // Sync and try to get bool
            if self
                .locked
                .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Acquire)
                .is_ok()
            {
                return;
            }

            fail_count += 1;

            // Spin while we wait for signal (we don't need to sync)
            while fail_count < MAX_SPIN_COUNT && self.locked.load(Ordering::Relaxed) {
                hint::spin_loop();
                fail_count += 1;
            }
        }

        loop {
            // Sync and try to get bool
            if self
                .locked
                .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Acquire)
                .is_ok()
            {
                return;
            }

            // Spin while we wait for signal (we don't need to sync)
            let mut idle_count = 0;
            while self.locked.load(Ordering::Relaxed) {
                // Try to continue working on something
                let yield_res = rayon::yield_now().unwrap(); // We assume we are on a rayon thread;

                if yield_res == rayon::Yield::Idle {
                    // Sadness
                    // Would be nice to have a WaitOnAddress for rayon
                    if idle_count < MAX_IDLE_COUNT {
                        idle_count += 1;
                        thread::yield_now();
                    } else {
                        idle_count += 1;
                        WaitOnAddress(
                            self.locked.as_ptr() as *const c_void,
                            LOCKED_LOCK.as_ptr() as *const c_void,
                            size_of::<AtomicBool>(),
                            RAYON_IDLE_SLEEP_TIMEOUT_MS,
                        );
                    }
                } else {
                    idle_count = 0;
                }
            }
        }
    }

    /// Locks this lock assuming this is NOT happening in a rayon thread
    pub unsafe fn lock_non_rayon(&self) {
        assert!(
            !rayon_utils::is_inside_rayon(),
            "Should only run in non-rayon threads!"
        );

        // Spin a little before giving up
        let mut fail_count = 0;
        while fail_count < MAX_SPIN_COUNT {
            // Sync and try to get bool
            if self
                .locked
                .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Acquire)
                .is_ok()
            {
                return;
            }

            fail_count += 1;

            // Spin while we wait for signal (we don't need to sync)
            while fail_count < MAX_SPIN_COUNT && self.locked.load(Ordering::Relaxed) {
                hint::spin_loop();
                fail_count += 1;
            }
        }

        loop {
            // Sync and try to get bool
            if self
                .locked
                .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Acquire)
                .is_ok()
            {
                return;
            }

            WaitOnAddress(
                self.locked.as_ptr() as *const c_void,
                LOCKED_LOCK.as_ptr() as *const c_void,
                size_of::<AtomicBool>(),
                INFINITE,
            );
        }
    }

    pub unsafe fn unlock(&self) {
        // Change bool first
        self.locked.store(false, Ordering::Release);
        // Tell everyone sleeping
        WakeByAddressAll(self.locked.as_ptr() as *const c_void)
    }
}
