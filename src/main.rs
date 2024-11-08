use clap::Parser;
use rsixel::SixelEncoder;
use std::error::Error;
use std::io::stdout;

/// Convert image to sixel format
#[derive(Parser)]
#[command(version, about, long_about= None)]
struct Args {
    /// Input image path
    img: String,
}

fn run(img_path: &str) -> Result<(), Box<dyn Error>> {
    let mut stdout = stdout().lock();
    let sixel_encoder = SixelEncoder::from(img_path)?;
    sixel_encoder.image_to_sixel(&mut stdout)?;
    Ok(())
}

fn main() {
    let args = Args::parse();
    if let Err(e) = run(&args.img) {
        eprintln!("{}: {e}", args.img);
    }
}
