
use crate::rayon_utils;

#[cfg(windows)]
use crate::windows as sys;

/// Lock for quick grab and release workloads
/// 
/// Design based on https://rigtorp.se/spinlock/
/// 
/// With help from https://marabos.nl/atomics/
#[derive(Default, Debug)]
pub struct FastLock {
    inner: sys::FastLockImpl,
}

impl FastLock {

    #[inline]
    pub unsafe fn lock(&self)
    {
        if rayon_utils::is_inside_rayon()
        {
            self.inner.lock_rayon()
        }
        else {
            self.inner.lock_non_rayon()
        }
    }

    #[inline]
    pub unsafe fn unlock(&self)
    {
        self.inner.unlock()
    }
}
