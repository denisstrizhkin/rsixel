use clap::Parser;
use rsixel::{SixelEncoder, MAX_PALETTE_COLORS};
use std::error::Error;
use std::io::stdout;

/// Convert image to sixel format
#[derive(Parser)]
#[command(version, about, long_about= None)]
struct Args {
    /// Input image path
    img: String,

    /// Color palette size
    #[arg(short, long, default_value_t=MAX_PALETTE_COLORS)]
    palette_size: usize,
}

fn run(img_path: &str, palette_size: usize) -> Result<(), Box<dyn Error>> {
    let mut stdout = stdout().lock();
    let sixel_encoder = SixelEncoder::from(img_path)?;
    sixel_encoder.image_to_sixel(&mut stdout, palette_size)?;
    Ok(())
}

fn main() {
    let args = Args::parse();
    if let Err(e) = run(&args.img, args.palette_size) {
        eprintln!("{}: {e}", args.img);
    }
}
