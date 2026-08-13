use criterion::{Criterion, black_box, criterion_group, criterion_main};

// We will test `Tone::from_char` using `to_lowercase()`
// For this we will pull from vnikey_core crate.
use vnikey_core::telex::Tone;

fn bench_tone_from_char(c: &mut Criterion) {
    c.bench_function("Tone::from_char (lowercase char)", |b| {
        b.iter(|| Tone::from_char(black_box('s')))
    });

    c.bench_function("Tone::from_char (uppercase char)", |b| {
        b.iter(|| Tone::from_char(black_box('S')))
    });

    c.bench_function("Tone::from_char (unmatched char)", |b| {
        b.iter(|| Tone::from_char(black_box('a')))
    });
}

criterion_group!(benches, bench_tone_from_char);
criterion_main!(benches);
