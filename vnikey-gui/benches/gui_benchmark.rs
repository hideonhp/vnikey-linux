use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("string_clone", |b| {
        b.iter(|| {
            let config_key = String::from("space");
            let mut current_key = config_key.clone();
            let changed = black_box(true);
            if changed {
                let _ = current_key.to_lowercase();
            }
            current_key
        })
    });

    c.bench_function("direct_ref", |b| {
        b.iter(|| {
            let mut config_key = String::from("space");
            let changed = black_box(true);
            if changed {
                config_key = config_key.to_lowercase();
            }
            config_key
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
