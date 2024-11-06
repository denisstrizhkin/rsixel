use clap::Parser;
use image::ImageReader;
use sixel::image_to_sixel;
use std::error::Error;
use std::io::{stdout, Write};
use std::path::{Path, PathBuf};

/// Convert image to sixel format
#[derive(Parser)]
#[command(version, about, long_about= None)]
struct Args {
    /// Input image path
    img: PathBuf,
}

fn run(path: &Path) -> Result<(), Box<dyn Error>> {
    let img = ImageReader::open(path)?.decode()?;
    let sixels = image_to_sixel(img);
    stdout().write_all(&sixels)?;
    Ok(())
}

fn main() {
    let args = Args::parse();
    if let Err(e) = run(&args.img) {
        eprintln!("{}: {e}", args.img.to_string_lossy());
    }
}
