// use crate::median_cut::ColorQuantizer;
use anyhow::Result;
use image::{
    imageops::{dither, ColorMap},
    ImageReader, RgbImage,
};
use itertools::chain;
use std::{array, io::Write, marker::PhantomData, path::Path};

use crate::{Palette, Quantizer, MAX_COLORS};

const SIXEL_SIZE: u8 = 6;
const SIXEL_OFFSET: u8 = 63;
const SIXEL_ESC: char = '\x1b';

#[derive(Debug)]
pub struct EncoderBuilder<'a, E: Quantizer> {
    img_path: &'a Path,
    debug: bool,
    _q: PhantomData<E>,
}

impl<'a, E: Quantizer> EncoderBuilder<'a, E> {
    pub fn new(img_path: &'a Path) -> Self {
        Self {
            img_path,
            debug: false,
            _q: Default::default(),
        }
    }

    pub fn debug(mut self, is_debug: bool) -> Self {
        self.debug = is_debug;
        self
    }

    pub fn build(self) -> Result<SixelEncoder<E>> {
        Ok(SixelEncoder {
            rgb8_img: ImageReader::open(self.img_path)?.decode()?.to_rgb8(),
            is_debug: self.debug,
            _q: Default::default(),
        })
    }
}

pub struct SixelEncoder<E: Quantizer> {
    rgb8_img: RgbImage,
    is_debug: bool,
    _q: PhantomData<E>,
}

struct Sixel(u8, usize);

enum EncoderCmd {
    Color(u8),
    Sixel(Sixel),
    MoveToBegining,
    MoveToNextLine,
}

#[derive(Default)]
struct SixelLineItem {
    color: u8,
    sixels: Vec<Sixel>,
    count: usize,
}

impl SixelLineItem {
    fn push(&mut self, sixel: u8, sixel_idx: usize) {
        match self.sixels.pop() {
            Some(Sixel(current_sixel, count)) => {
                if current_sixel != sixel {
                    self.sixels.push(Sixel(current_sixel, count));
                    self.sixels.push(Sixel(sixel, 1));
                } else {
                    self.sixels.push(Sixel(current_sixel, count + 1))
                }
            }
            None => {
                if sixel_idx > 0 {
                    self.sixels.push(Sixel(0, sixel_idx));
                    self.count = sixel_idx;
                }
                self.sixels.push(Sixel(sixel, 1));
            }
        }
        self.count += 1;
    }

    fn align(&mut self, sixel_idx: usize) {
        if self.count != sixel_idx + 1 {
            debug_assert!(self.sixels.last().is_some());
            let Sixel(sixel, count) = unsafe { self.sixels.pop().unwrap_unchecked() };
            if sixel == 0 {
                self.sixels.push(Sixel(0, count + 1));
            } else {
                self.sixels.push(Sixel(sixel, count));
                self.sixels.push(Sixel(0, 1));
            }
            self.count += 1;
        }
    }
}

struct SixelLine {
    indices: [Option<usize>; MAX_COLORS],
    count: usize,
    colors: [SixelLineItem; MAX_COLORS],
}

impl Default for SixelLine {
    fn default() -> Self {
        Self {
            indices: [None; MAX_COLORS],
            count: 0,
            colors: array::from_fn(|_| Default::default()),
        }
    }
}

impl SixelLine {
    fn push(&mut self, color: u8, sixel: u8, sixel_idx: usize) {
        let idx = match self.indices[color as usize] {
            Some(idx) => idx,
            None => {
                let idx = self.count;
                self.indices[color as usize] = Some(idx);
                self.colors[idx].color = color;
                self.count += 1;
                idx
            }
        };
        self.colors[idx].push(sixel, sixel_idx);
    }

    fn align(&mut self, sixel_idx: usize) {
        for color in self.colors[0..self.count].iter_mut() {
            color.align(sixel_idx);
        }
    }
}

fn get_sixels_map(colors: impl Iterator<Item = usize>) -> impl Iterator<Item = (u8, u8)> {
    let mut indices = [Option::<usize>::None; MAX_COLORS];
    let mut sixels = [Option::<(u8, u8)>::None; SIXEL_SIZE as usize];
    for (i, color) in colors.take(SIXEL_SIZE as usize).enumerate() {
        let idx = match indices[color] {
            Some(idx) => idx,
            None => {
                indices[color] = Some(i);
                i
            }
        };
        sixels[idx] = Some((color as u8, sixels[idx].unwrap_or_default().1 + (1 << i)));
    }
    sixels.into_iter().flatten()
}

fn get_sixel_lines(img: RgbImage, palette: Palette) -> impl Iterator<Item = SixelLine> {
    let width = img.width();
    let height = img.height();
    let sixel_size = SIXEL_SIZE as u32;
    (0..height + sixel_size - 1)
        .step_by(sixel_size as usize)
        .map(move |y| {
            let (img, palette) = (&img, &palette);
            let mut colors: SixelLine = Default::default();
            for (i, sixels) in (0..width)
                .map(|x| (y..height).map(move |y| palette.index_of(img.get_pixel(x, y))))
                .map(get_sixels_map)
                .enumerate()
            {
                for (color, sixel) in sixels {
                    colors.push(color, sixel, i);
                }
                colors.align(i);
            }
            debug_assert!(colors
                .colors
                .iter()
                .take(colors.count)
                .all(|color| color.count == width as usize));
            colors
        })
}

fn get_encoder_cmds(lines: impl Iterator<Item = SixelLine>) -> impl Iterator<Item = EncoderCmd> {
    lines.flat_map(|line| {
        line.colors
            .into_iter()
            .take(line.count)
            .enumerate()
            .flat_map(move |(i, color)| {
                chain!(
                    [EncoderCmd::Color(color.color)],
                    color.sixels.into_iter().map(EncoderCmd::Sixel),
                    [if i == line.count.saturating_sub(1) {
                        EncoderCmd::MoveToNextLine
                    } else {
                        EncoderCmd::MoveToBegining
                    }]
                )
            })
    })
}

impl<E: Quantizer> SixelEncoder<E> {
    pub fn image_to_sixel<W: Write>(
        mut self,
        w: &mut W,
        palette_size: usize,
        is_dither: bool,
    ) -> Result<()> {
        let palette = E::quantize(&self.rgb8_img, palette_size);
        if is_dither {
            dither(&mut self.rgb8_img, &palette);
        }
        let width = self.rgb8_img.width() as usize;
        let height = self.rgb8_img.height() as usize;
        write!(w, "{SIXEL_ESC}Pq\"1;1;{width};{height}")?;
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
                    let sixel = (sixel + SIXEL_OFFSET) as char;
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
        write!(w, "{SIXEL_ESC}\\")?;
        Ok(())
    }
}
