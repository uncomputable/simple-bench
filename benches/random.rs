use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::Rng;
use simple_bench::better_add;

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("add random");
    group.sample_size(1000);

    let mut rng = rand::thread_rng();
    group.bench_function("add random", |b| {
        b.iter(|| {
            let a = rng.gen();
            let b = rng.gen();
            better_add(black_box(a), black_box(b))
        })
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
