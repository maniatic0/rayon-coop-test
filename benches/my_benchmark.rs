use rayon_coop::*;

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

use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn criterion_benchmark_collatz(c: &mut Criterion) {
    c.bench_function("collatz", |b| b.iter(|| collatz(black_box(COLLATZ_ITER))));
}

criterion_group! {
    name = collatz_bench;
    config = Criterion::default();
    targets = criterion_benchmark_collatz
}

pub fn criterion_benchmark_rayon_lock(c: &mut Criterion) {
    c.bench_function("rayon lock", |b| b.iter(|| rayon_lock_timing()));
}

pub fn criterion_benchmark_normal_lock(c: &mut Criterion) {
    c.bench_function("normal lock", |b| b.iter(|| normal_lock_timing()));
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10).measurement_time(Duration::from_secs(121));
    targets = criterion_benchmark_rayon_lock, criterion_benchmark_normal_lock
}
criterion_main!(collatz_bench, benches);
