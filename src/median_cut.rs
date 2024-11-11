use image::Rgb;
use std::array;

const MAX_HIST_COLORS: usize = 32 * 32 * 32;
pub const MAX_PALETTE_COLORS: usize = 256;
const RGB_MASK: u8 = 0b11111000;

pub fn median_cut(mut color_hist: ColorHist, palette_size: usize) -> ColorPalette {
    let mut stack: [(Option<&mut [ColorHistEntry]>, usize); MAX_PALETTE_COLORS * 2] =
        array::from_fn(|_| (None, 0));
    let color_hist_len = color_hist.len();
    stack[0] = (Some(&mut color_hist.map[0..color_hist_len]), 1);
    let mut pos = 0;
    let mut stack_count = 1;
    let mut colors = [0; MAX_PALETTE_COLORS];
    let mut colors_count = 0;
    eprintln!("median_cut: requested palette size: {}", palette_size);
    while pos < stack_count {
        let slice = stack[pos].0.take().unwrap();
        let level = stack[pos].1;
        if (level >= palette_size || slice.len() == 1) && !slice.is_empty() {
            let color = slice.iter().map(|c| c.color as u32 * c.count).sum::<u32>();
            let count_sum = slice.iter().map(|c| c.count).sum::<u32>();
            let color = color / count_sum;
            colors[colors_count] = color as u16;
            colors_count += 1;
        } else if !slice.is_empty() {
            let (r_min, r_max, g_min, g_max, b_min, b_max) = slice.iter().fold(
                (u8::MAX, u8::MIN, u8::MAX, u8::MIN, u8::MAX, u8::MIN),
                |(r_min, r_max, g_min, g_max, b_min, b_max), c| {
                    let red = u16_to_red(c.color);
                    let green = u16_to_green(c.color);
                    let blue = u16_to_blue(c.color);
                    (
                        r_min.min(red),
                        r_max.max(red),
                        g_min.min(green),
                        g_max.max(green),
                        b_min.min(blue),
                        b_max.max(blue),
                    )
                },
            );
            let r_delta = r_max - r_min;
            let g_delta = g_max - g_min;
            let b_delta = b_max - b_min;
            let max_delta = r_delta.max(g_delta.max(b_delta));
            let convert_fn = if max_delta == r_delta {
                u16_to_red
            } else if max_delta == g_delta {
                u16_to_green
            } else {
                u16_to_blue
            };
            slice.sort_by(|a, b| {
                let a = convert_fn(a.color) as u32 * a.count;
                let b = convert_fn(b.color) as u32 * b.count;
                a.cmp(&b)
            });
            let (left, right) = slice.split_at_mut(slice.len() >> 1);
            let new_level = (level << 1) + 1;
            stack[stack_count] = (Some(left), new_level);
            stack_count += 1;
            stack[stack_count] = (Some(right), new_level);
            stack_count += 1;
        }
        pos += 1;
        eprintln!("median_cut: stack_count: {stack_count}, pos {pos}");
    }
    eprintln!("median_cut: colors_count: {}", colors_count);
    ColorPalette::from_colors(colors, colors_count)
}

pub struct ColorHist {
    map: [ColorHistEntry; MAX_HIST_COLORS],
    count: usize,
}

#[derive(Default, Debug, Clone, Copy)]
struct ColorHistEntry {
    color: u16,
    count: u32,
}

impl ColorHist {
    pub fn from_pixels(pixels: &[Rgb<u8>]) -> Self {
        let mut map = [ColorHistEntry::default(); MAX_HIST_COLORS];
        let mut count = 0;
        for rgb in pixels {
            let key = rgb_to_u16(*rgb) as usize;
            if map[key].count == 0 {
                map[key].color = key as u16;
                count += 1;
            }
            map[key].count += 1;
        }
        map.sort_by(|a, b| b.count.cmp(&a.count));
        Self { map, count }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.count
    }
}

pub struct ColorPalette {
    colors: [u16; MAX_PALETTE_COLORS],
    cache: [ColorPaletteCacheEntry; MAX_HIST_COLORS],
    count: usize,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct ColorPaletteCacheEntry {
    is_hit: bool,
    index: usize,
}

impl ColorPalette {
    pub fn from_color_hist(color_hist: ColorHist, palette_size: usize) -> Self {
        let palette_size = palette_size.min(MAX_PALETTE_COLORS);
        if palette_size == color_hist.len() {
            let mut colors = [0; MAX_PALETTE_COLORS];
            let mut count = 0;
            for key in 0..color_hist.len() {
                colors[count] = color_hist.map[key].color;
                count += 1;
            }
            Self::from_colors(colors, count)
        } else {
            median_cut(color_hist, palette_size)
        }
    }

    pub fn from_colors(mut colors: [u16; MAX_PALETTE_COLORS], count: usize) -> Self {
        colors[0..count].sort();
        ColorPalette {
            colors,
            cache: [ColorPaletteCacheEntry::default(); MAX_HIST_COLORS],
            count,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.count
    }

    #[inline]
    pub fn get_palette(&self) -> &[u16] {
        &self.colors[0..self.len()]
    }

    pub fn get_index(&mut self, rgb: Rgb<u8>) -> usize {
        let color = rgb_to_u16(rgb);
        let color_usize = color as usize;
        if self.cache[color_usize].is_hit {
            self.cache[color_usize].index
        } else {
            let index = self.get_index_internal(color);
            self.cache[color_usize].is_hit = true;
            self.cache[color_usize].index = index;
            index
        }
    }

    fn get_index_internal(&self, color: u16) -> usize {
        let mut left = 0;
        let mut right = self.len();
        while left < right {
            let mid = (right + left) / 2;
            if self.colors[mid] < color {
                left = mid + 1;
            } else {
                right = mid;
            }
        }
        if left == self.len() {
            left - 1
        } else if left > 0 {
            if color - self.colors[left - 1] < self.colors[left] - color {
                left - 1
            } else {
                left
            }
        } else {
            left
        }
    }
}

#[inline]
pub fn rgb_to_u16(rgb: Rgb<u8>) -> u16 {
    (((RGB_MASK & rgb[0]) as u16) << 7)
        + (((RGB_MASK & rgb[1]) as u16) << 2)
        + (((RGB_MASK & rgb[2]) as u16) >> 3)
}

#[inline]
pub fn u16_to_red(rgb: u16) -> u8 {
    ((rgb >> 7) as u8) & RGB_MASK
}

#[inline]
pub fn u16_to_green(rgb: u16) -> u8 {
    ((rgb >> 2) as u8) & RGB_MASK
}

#[inline]
pub fn u16_to_blue(rgb: u16) -> u8 {
    (rgb << 3) as u8
}

#[inline]
pub fn u16_to_rgb(rgb: u16) -> Rgb<u8> {
    Rgb::from([u16_to_red(rgb), u16_to_green(rgb), u16_to_blue(rgb)])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgb_to_u16_and_back() {
        let white = Rgb::from([0, 0, 0]);
        let white_masked = Rgb::from([
            white[0] & RGB_MASK,
            white[1] & RGB_MASK,
            white[2] & RGB_MASK,
        ]);
        let white_u16 = rgb_to_u16(white);
        let white_from_u16 = u16_to_rgb(white_u16);
        assert_eq!(white_masked, white_from_u16);

        let black = Rgb::from([255, 255, 255]);
        let black_masked = Rgb::from([
            black[0] & RGB_MASK,
            black[1] & RGB_MASK,
            black[2] & RGB_MASK,
        ]);
        let black_u16 = rgb_to_u16(black);
        let black_from_u16 = u16_to_rgb(black_u16);
        assert_eq!(black_masked, black_from_u16);

        let lime = Rgb::from([137, 243, 54]);
        let lime_masked = Rgb::from([lime[0] & RGB_MASK, lime[1] & RGB_MASK, lime[2] & RGB_MASK]);
        let lime_u16 = rgb_to_u16(lime);
        let lime_from_u16 = u16_to_rgb(lime_u16);
        assert_eq!(lime_masked, lime_from_u16);
    }
}
