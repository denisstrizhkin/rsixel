use anyhow::Result;
use clap::Parser;
use log::debug;
use rsixel::{EncoderBuilder, OctreeQuantizer, MAX_COLORS};
use std::{io, path::PathBuf};

/// Convert image to sixel format
#[derive(Parser, Debug)]
#[command(version, about, long_about= None)]
struct Args {
    /// Input image path
    img: PathBuf,

    /// Color palette size
    #[arg(short, long, default_value_t=MAX_COLORS)]
    palette_size: usize,

    /// Use dithering
    #[arg(short, long, default_value_t = false)]
    dither: bool,

    /// Debug
    #[arg(long, default_value_t = false)]
    debug: bool,
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::try_parse()?;
    debug!("Recieved args: {args:#?}");
    let sixel_encoder = EncoderBuilder::<OctreeQuantizer>::new(&args.img)
        .debug(args.debug)
        .build()?;
    sixel_encoder.image_to_sixel(&mut io::stdout().lock(), args.palette_size, args.dither)?;
    Ok(())
}
