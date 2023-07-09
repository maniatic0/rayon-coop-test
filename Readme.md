# Test Coop Rayon

## Problem

We want to update entity clusters as the [Hybrid Actor/ECS object model](https://www.youtube.com/watch?v=jjEsB611kxs) proposed by Bobby Anguelov. They could be parallelized by updating them in tasks, but their code might take locks (as gameplay code is kind of unbound). We don't want to sleep a worker thread if it could take other jobs while the lock is taken by another thread.

## Possible solution

Test if it would be better to yield back to rayon rather than sleeping while taking a lock.

## Test

Prepare lots of work with tasks in the middle that take a lock and sleep. This avoids rayon LIFO behavior affecting the test.

## Results

Negative. It's at best the same as taking a normal lock, and at worst 5-10% slower.

From profiler captures it can be seen that some threads start lots of jobs that take a lock, which means their callstacks are really deep. This is due to the nature of due `rayon::yield_now()`, which doesn't really stop the outer job, as it just opens a new one below it and returns to the outer one once it's finished.

## Proposal

Need to use proper fibers, and a runtime that can start and stop them at any point.
