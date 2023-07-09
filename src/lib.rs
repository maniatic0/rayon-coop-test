pub mod rayon_utils;

#[cfg(windows)]
mod windows;

mod shared;
pub use shared::*;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::{Arc, Mutex};

    use std::time::Duration;

    use std::thread;

    use rayon;

    /// https://en.wikipedia.org/wiki/Collatz_conjecture
    fn collatz(mut n: usize) {
        let mut i: usize = 0;

        while n != 1 {
            if n % 2 == 0 {
                n = n / 2;
            } else {
                n = 3 * n + 1;
            }
            i += 1;
        }

        std::hint::black_box(i);
    }

    const COLLATZ_ITER: usize = 10_000;

    const BLOCKER_MULTIPLIER: usize = 64;

    const BLOCKING_SLEEP_MS: u64 = 1;

    const MUTEXES_DIVIDER: usize = 4;

    #[test]
    fn rayon_lock_timing() {
        rayon::scope(|s| {
            let concurrency = rayon::current_num_threads();

            let mut mutexes = Vec::new();

            for _ in 0..(concurrency / MUTEXES_DIVIDER) {
                mutexes.push(Arc::new(FastLock::default()));
            }

            for i in 1..COLLATZ_ITER {
                let i_ = i;
                s.spawn(move |_| {
                    collatz(i_);
                });
            }

            for i in 0..(concurrency * BLOCKER_MULTIPLIER) {
                let i_ = i;
                let mutex_ = mutexes[i % mutexes.len()].clone();
                s.spawn(move |_| {
                    unsafe {
                        mutex_.lock();
                    }
                    collatz(i_ + COLLATZ_ITER);
                    thread::sleep(Duration::from_millis(BLOCKING_SLEEP_MS));
                    unsafe {
                        mutex_.unlock();
                    }
                });
            }

            for i in 1..COLLATZ_ITER {
                let i_ = i;
                s.spawn(move |_| {
                    collatz(i_);
                });
            }
        });
    }

    #[test]
    fn normal_lock_timing() {
        rayon::scope(|s| {
            let concurrency = rayon::current_num_threads();

            let mut mutexes = Vec::new();

            for _ in 0..(concurrency / MUTEXES_DIVIDER) {
                mutexes.push(Arc::new(Mutex::new(())));
            }

            for i in 1..COLLATZ_ITER {
                let i_ = i;
                s.spawn(move |_| {
                    collatz(i_);
                });
            }

            for i in 0..(concurrency * BLOCKER_MULTIPLIER) {
                let i_ = i;
                let mutex_ = mutexes[i % mutexes.len()].clone();
                s.spawn(move |_| {
                    let _guard = mutex_.lock().unwrap();
                    collatz(i_ + COLLATZ_ITER);
                    thread::sleep(Duration::from_millis(BLOCKING_SLEEP_MS));
                });
            }

            for i in 1..COLLATZ_ITER {
                let i_ = i;
                s.spawn(move |_| {
                    collatz(i_);
                });
            }
        });
    }
}
