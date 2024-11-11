use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rsixel::median_cut::ColorHist;
use rsixel::SixelEncoder;

fn color_hist(encoder: &SixelEncoder) -> ColorHist {
    encoder.color_hist()
}

fn criterion_benchmark(c: &mut Criterion) {
    let encoder = SixelEncoder::from("assets/snake.png").unwrap();

    c.bench_function("color_hist snake.png", |b| {
        b.iter(|| color_hist(black_box(&encoder)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
