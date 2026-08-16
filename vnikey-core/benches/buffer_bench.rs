use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use vnikey_core::buffer::CharBuffer;

fn manual_remove(buffer: &mut CharBuffer, index: usize) {
    let r_len = buffer.len();
    for k in index..r_len - 1 {
        buffer.replace_at(k, buffer.as_slice()[k + 1]);
    }
    buffer.pop();
}

fn optimized_remove(buffer: &mut CharBuffer, index: usize) {
    buffer.remove(index);
}

fn bench_buffer_remove(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_remove");

    group.bench_function("manual_remove", |b| {
        b.iter(|| {
            let mut buf = CharBuffer::new();
            buf.push('a');
            buf.push('b');
            buf.push('c');
            buf.push('d');
            buf.push('e');
            manual_remove(black_box(&mut buf), black_box(1));
        })
    });

    group.bench_function("optimized_remove", |b| {
        b.iter(|| {
            let mut buf = CharBuffer::new();
            buf.push('a');
            buf.push('b');
            buf.push('c');
            buf.push('d');
            buf.push('e');
            optimized_remove(black_box(&mut buf), black_box(1));
        })
    });

    group.finish();
}

criterion_group!(benches, bench_buffer_remove);
criterion_main!(benches);
