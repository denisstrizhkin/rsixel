use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rsixel::SixelEncoder;

fn encoder_from_image(img_path: &str) -> SixelEncoder {
    SixelEncoder::from(img_path).unwrap()
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("open_image snake.png", |b| {
        b.iter(|| encoder_from_image(black_box("assets/snake.png")))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
