use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn to_lower_fast(c: char) -> char {
    if c.is_ascii() {
        c.to_ascii_lowercase()
    } else {
        c.to_lowercase().next().unwrap_or(c)
    }
}

fn bench_lowercase(c: &mut Criterion) {
    c.bench_function("to_lowercase (ascii)", |b| {
        b.iter(|| black_box('A').to_lowercase().next().unwrap_or('A'))
    });

    c.bench_function("to_lower_fast (ascii)", |b| {
        b.iter(|| to_lower_fast(black_box('A')))
    });

    c.bench_function("to_lowercase (vietnamese)", |b| {
        b.iter(|| black_box('À').to_lowercase().next().unwrap_or('À'))
    });

    c.bench_function("to_lower_fast (vietnamese)", |b| {
        b.iter(|| to_lower_fast(black_box('À')))
    });
}

criterion_group!(benches, bench_lowercase);
criterion_main!(benches);
