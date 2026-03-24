use std::hint::black_box;

use bumpalo::Bump;
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};

use lsm::memtable::Memtable;

fn bench_put(c: &mut Criterion) {
    c.bench_function("memtable_put_100k_u64_u64", |b| {
        b.iter_batched_ref(
            || Bump::with_capacity(8 << 20),
            |bump| {
                let mut mem = Memtable::<u64, u64, &Bump>::new_in(&*bump);

                for i in 0u64..100_000 {
                    black_box(mem.put(black_box(i), black_box(i + 1)));
                }
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_get_hit(c: &mut Criterion) {
    c.bench_function("memtable_get_hit_100k_u64_u64", |b| {
        b.iter_batched_ref(
            || Bump::with_capacity(8 << 20),
            |bump| {
                let mut mem = Memtable::<u64, u64, &Bump>::new_in(&*bump);

                for i in 0u64..100_000 {
                    mem.put(i, i + 1);
                }

                for i in 0u64..100_000 {
                    black_box(mem.get(black_box(&i)));
                }
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_get_miss(c: &mut Criterion) {
    c.bench_function("memtable_get_miss_100k_u64_u64", |b| {
        b.iter_batched_ref(
            || Bump::with_capacity(8 << 20),
            |bump| {
                let mut mem = Memtable::<u64, u64, &Bump>::new_in(&*bump);

                for i in 0u64..100_000 {
                    mem.put(i, i + 1);
                }

                for i in 100_000u64..200_000 {
                    black_box(mem.get(black_box(&i)));
                }
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_mixed_workload(c: &mut Criterion) {
    c.bench_function("memtable_mixed_put_get_100k", |b| {
        b.iter_batched_ref(
            || Bump::with_capacity(8 << 20),
            |bump| {
                let mut mem = Memtable::<u64, u64, &Bump>::new_in(&*bump);

                for i in 0u64..50_000 {
                    mem.put(i, i + 1);
                }

                for i in 50_000u64..100_000 {
                    black_box(mem.put(black_box(i), black_box(i + 1)));
                }

                for i in 0u64..100_000 {
                    black_box(mem.get(black_box(&i)));
                }
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(
    benches,
    bench_put,
    bench_get_hit,
    bench_get_miss,
    bench_mixed_workload
);
criterion_main!(benches);
