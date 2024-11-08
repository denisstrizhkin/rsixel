use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rsixel::SixelEncoder;
use std::error::Error;

fn encoder_from_image(img_path: &str) -> Result<SixelEncoder, Box<dyn Error>> {
    Ok(SixelEncoder::from(img_path)?)
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("open_image snake.png", |b| {
        b.iter(|| encoder_from_image(black_box("snake.png")))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
