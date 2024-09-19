// benches/compression_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fen_compression::compress_fen;

fn compression_benchmark(c: &mut Criterion) {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    c.bench_function("compress_fen", |b| b.iter(|| compress_fen(black_box(fen))));
}

criterion_group!(benches, compression_benchmark);
criterion_main!(benches);
