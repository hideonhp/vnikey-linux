use criterion::{Criterion, black_box, criterion_group, criterion_main};
use vnikey_core::validation::is_valid_vietnamese_syllable;

fn bench_validation(c: &mut Criterion) {
    c.bench_function("is_valid_vietnamese_syllable (valid hoang)", |b| {
        let chars: Vec<char> = "hoàng".chars().collect();
        b.iter(|| is_valid_vietnamese_syllable(black_box(&chars)))
    });

    c.bench_function("is_valid_vietnamese_syllable (invalid english)", |b| {
        let chars: Vec<char> = "englí".chars().collect();
        b.iter(|| is_valid_vietnamese_syllable(black_box(&chars)))
    });
}

criterion_group!(benches, bench_validation);
criterion_main!(benches);
