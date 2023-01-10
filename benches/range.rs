use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use simple_bench::better_fibonacci;

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("fib range");

    for n in [10, 20, 30, 40].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(n), n, |b, &i| {
            b.iter(|| better_fibonacci(i));
        });
    }
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
