use rayon::current_thread_index;

/// Returns true when the current thread is part of any rayon pool
#[inline]
pub fn is_inside_rayon() -> bool
{
    // It would be cool to query this directly rather than doing this
    current_thread_index().is_some()
}