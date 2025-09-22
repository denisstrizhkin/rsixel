use std::iter;

pub mod median_cut;
mod octree;
mod queue;
pub mod sixel_encoder;

use image::{imageops::ColorMap, Rgb, RgbImage};
use kuina::stack_vec::StackVec;

pub use octree::OctreeQuantizer;
pub use sixel_encoder::EncoderBuilder;

pub const MAX_COLORS: usize = 256;

pub type Color = Rgb<u8>;

#[derive(Default)]
pub struct Palette {
    colors: StackVec<Color, MAX_COLORS>,
}

impl Palette {
    pub fn push(&mut self, color: Color) {
        self.colors.push(color);
    }

    pub fn len(&self) -> usize {
        self.colors.len()
    }

    pub fn get_palette(&self) -> &[Color] {
        &self.colors
    }
}

impl ColorMap for Palette {
    type Color = Color;
    fn index_of(&self, color: &Self::Color) -> usize {
        self.colors
            .iter()
            .enumerate()
            .map(|(index, pcolor)| {
                let dist = iter::zip(pcolor.0, color.0)
                    .map(|(a, b)| {
                        let diff = a as i32 - b as i32;
                        (diff * diff) as u32
                    })
                    .sum::<u32>();
                (index, dist)
            })
            .min_by(|a, b| a.1.cmp(&b.1))
            .unwrap()
            .0
    }

    fn map_color(&self, color: &mut Self::Color) {
        *color = self.colors[self.index_of(color)]
    }
}

pub trait Quantizer {
    fn quantize(img: &RgbImage, color_count: usize) -> Palette;
}
