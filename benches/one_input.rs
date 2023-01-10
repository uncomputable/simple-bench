use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use simple_bench::better_fibonacci;

fn bench(c: &mut Criterion) {
    let n: u64 = 20;

    c.bench_with_input(BenchmarkId::new("fib one input", n), &n, |b, &i| {
        b.iter(|| better_fibonacci(i));
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
