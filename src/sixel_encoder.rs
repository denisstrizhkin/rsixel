// use crate::median_cut::ColorQuantizer;
use crate::octree::ColorQuantizer;
use anyhow::Result;
use image::{
    imageops::dither,
    {ImageReader, RgbImage},
};
use itertools::chain;
use std::{array, collections::HashMap, io::Write, path::Path};

const SIXEL_SIZE: u8 = 6;
const ESC: char = '\x1b';

#[derive(Debug)]
pub struct EncoderBuilder<'a> {
    img_path: &'a Path,
    debug: bool,
}

impl<'a> EncoderBuilder<'a> {
    pub fn new(img_path: &'a Path) -> Self {
        Self {
            img_path,
            debug: false,
        }
    }

    pub fn debug(mut self, is_debug: bool) -> Self {
        self.debug = is_debug;
        self
    }

    pub fn build(self) -> Result<SixelEncoder> {
        Ok(SixelEncoder {
            rgb8_img: ImageReader::open(self.img_path)?.decode()?.to_rgb8(),
            is_debug: self.debug,
        })
    }
}

pub struct SixelEncoder {
    rgb8_img: RgbImage,
    is_debug: bool,
}

struct Sixel(char, usize);

enum EncoderCmd {
    Color(u8),
    Sixel(Sixel),
    MoveToBegining,
    MoveToNextLine,
}

type SixelPixels = [Option<u8>; SIXEL_SIZE as usize];
type SixelLine = HashMap<u8, Vec<Sixel>>;

fn get_sixel_lines(img: RgbImage, palette: ColorQuantizer) -> impl Iterator<Item = SixelLine> {
    let width = img.width();
    let height = img.height();
    let sixel_size = SIXEL_SIZE as u32;
    (0..height.div_ceil(sixel_size)).map(move |y| {
        let mut colors: SixelLine = HashMap::new();
        for sixel_pixels in (0..width).map(|x| {
            array::from_fn(|i| {
                img.get_pixel_checked(x, y * sixel_size + i as u32)
                    .map(|&c| palette.get_index(c) as u8)
            })
        }) {
            for &color in sixel_pixels.iter().flatten() {
                colors.entry(color).or_default();
            }
            for (&color, sixels) in colors.iter_mut() {
                let sixel = get_sixel(&sixel_pixels, color);
                match sixels.pop() {
                    Some(Sixel(current_sixel, count)) => {
                        if current_sixel != sixel {
                            sixels.push(Sixel(current_sixel, count));
                            sixels.push(Sixel(sixel, 1));
                        } else {
                            sixels.push(Sixel(current_sixel, count + 1))
                        }
                    }
                    None => sixels.push(Sixel(sixel, 1)),
                }
            }
        }
        for (_, sixels) in colors.iter_mut() {
            let total_count = sixels
                .iter()
                .map(|Sixel(_, count)| *count as u32)
                .sum::<u32>();
            if total_count < width {
                sixels.insert(0, Sixel('?', (width - total_count) as usize));
            }
        }
        colors
    })
}

fn get_sixel(sixel_pixels: &SixelPixels, color_idx: u8) -> char {
    (sixel_pixels
        .iter()
        .enumerate()
        .filter(|(_, c)| c.is_some_and(|c| c == color_idx))
        .map(|(i, _)| 1 << i)
        .sum::<u8>()
        + 63) as char
}

fn get_encoder_cmds(lines: impl Iterator<Item = SixelLine>) -> impl Iterator<Item = EncoderCmd> {
    lines.flat_map(|line| {
        let colors_len = line.len();
        line.into_iter()
            .enumerate()
            .flat_map(move |(i, (color, sixels))| {
                chain!(
                    [EncoderCmd::Color(color)],
                    sixels.into_iter().map(EncoderCmd::Sixel),
                    [if i == colors_len.saturating_sub(1) {
                        EncoderCmd::MoveToNextLine
                    } else {
                        EncoderCmd::MoveToBegining
                    }]
                )
            })
    })
}

impl SixelEncoder {
    pub fn image_to_sixel<W: Write>(
        mut self,
        w: &mut W,
        palette_size: usize,
        is_dither: bool,
    ) -> Result<()> {
        let palette = ColorQuantizer::from(&self.rgb8_img, palette_size);
        if is_dither {
            dither(&mut self.rgb8_img, &palette);
        }
        let width = self.rgb8_img.width() as usize;
        let height = self.rgb8_img.height() as usize;
        write!(w, "{ESC}Pq\"1;1;{width};{height}")?;
        if self.is_debug {
            writeln!(w)?;
        }
        for (i, rgb) in palette
            .get_palette()
            .iter()
            .map(|color| color.0.map(|c| c as u16 * 100 / 255))
            .enumerate()
        {
            write!(w, "#{i};2;{};{};{}", rgb[0], rgb[1], rgb[2])?;
        }
        if self.is_debug {
            writeln!(w)?
        }
        for cmd in get_encoder_cmds(get_sixel_lines(self.rgb8_img, palette)) {
            match cmd {
                EncoderCmd::Color(color) => write!(w, "#{color}"),
                EncoderCmd::Sixel(Sixel(sixel, count)) => {
                    if count == 1 {
                        write!(w, "{sixel}")
                    } else {
                        write!(w, "!{count}{sixel}")
                    }
                }
                EncoderCmd::MoveToBegining => {
                    if self.is_debug {
                        writeln!(w, "$")
                    } else {
                        write!(w, "$")
                    }
                }
                EncoderCmd::MoveToNextLine => {
                    if self.is_debug {
                        writeln!(w, "-")
                    } else {
                        write!(w, "-")
                    }
                }
            }?
        }
        write!(w, "{ESC}\\")?;
        Ok(())
    }
}
