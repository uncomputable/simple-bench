use criterion::{black_box, criterion_group, criterion_main, Criterion};
use simple_bench::better_fibonacci;

fn bench(c: &mut Criterion) {
    c.bench_function("fib default 20", |b| {
        b.iter(|| better_fibonacci(black_box(20)))
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
