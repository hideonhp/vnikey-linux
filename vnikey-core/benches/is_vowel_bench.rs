use criterion::{Criterion, black_box, criterion_group, criterion_main};
use vnikey_core::telex::is_vowel;

fn bench_is_vowel(c: &mut Criterion) {
    let mut group = c.benchmark_group("is_vowel");

    // Benchmark a common lowercase vowel
    group.bench_function("lowercase_a", |b| b.iter(|| is_vowel(black_box('a'))));

    // Benchmark an uppercase vowel
    group.bench_function("uppercase_A", |b| b.iter(|| is_vowel(black_box('A'))));

    // Benchmark a non-vowel consonant
    group.bench_function("consonant_b", |b| b.iter(|| is_vowel(black_box('b'))));

    // Benchmark a Vietnamese lowercase vowel with tone
    group.bench_function("lower_viet_ế", |b| b.iter(|| is_vowel(black_box('ế'))));

    // Benchmark a Vietnamese uppercase vowel with tone
    group.bench_function("upper_viet_Ế", |b| b.iter(|| is_vowel(black_box('Ế'))));

    group.finish();
}

criterion_group!(benches, bench_is_vowel);
criterion_main!(benches);
