# Test Coop Rayon

Test if it would be better to yield back to rayon rather than sleeping while taking a lock.

## Test

Prepare lots of work with tasks in the middle that take a lock and sleep. This avoids rayon LIFO behavior affecting the test.

## Results

Negative. It's at best the same as taking a normal lock, and at worst 5-10% slower.

From profiler captures it can be seen that some threads start lots of jobs that take the lock, which means their callstacks are really deep. This is due to the nature of due `rayon::yield_now()`, which doesn't really stop the outer job, as it just opens a new one below it and returns to the outer one once it's finished.

## Proposal

Need to use proper fibers, and a runtime that can start and stop them at any point.
