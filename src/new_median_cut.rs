use image::Rgb;

const RGB_COMPONENT_SIZE: usize = 32;
const MAX_HIST_COLORS: usize = RGB_COMPONENT_SIZE * RGB_COMPONENT_SIZE * RGB_COMPONENT_SIZE;
pub const MAX_PALETTE_COLORS: usize = 256;
const RGB_MASK: u8 = 0b11111000;

#[derive(Default, Clone, Copy, Debug)]
enum SplitBy {
    #[default]
    Red,
    Green,
    Blue,
}

#[derive(Default, Clone, Copy, Debug)]
struct VBoxBoundaries {
    r_min: u8,
    r_max: u8,
    g_min: u8,
    g_max: u8,
    b_min: u8,
    b_max: u8,
}

impl VBoxBoundaries {
    #[inline]
    pub fn from(r_min: u8, r_max: u8, g_min: u8, g_max: u8, b_min: u8, b_max: u8) -> Self {
        Self {
            r_min,
            r_max,
            g_min,
            g_max,
            b_min,
            b_max,
        }
    }

    #[inline]
    pub fn dimensions(&self) -> (u8, u8, u8) {
        (
            self.r_max - self.r_min + 1,
            self.g_max - self.g_min + 1,
            self.b_max - self.b_min + 1,
        )
    }

    #[inline]
    pub fn volume(&self) -> u16 {
        ((self.r_max - self.r_min + 1) as u16)
            * ((self.g_max - self.g_min + 1) as u16)
            * ((self.b_max - self.b_min + 1) as u16)
    }

    #[inline]
    pub fn iterate<F>(&self, mut f: F)
    where
        F: FnMut(u16, u8, u8, u8),
    {
        for r in self.r_min..=self.r_max {
            for g in self.g_min..=self.g_max {
                for b in self.b_min..=self.b_max {
                    f(rgb_to_u16(Rgb::from([r << 3, g << 3, b << 3])), r, g, b);
                }
            }
        }
    }
}

#[derive(Default, Clone, Copy, Debug)]
struct VBox {
    boundaries: VBoxBoundaries,
    counts: [u32; RGB_COMPONENT_SIZE],
    volume: u16,
    split_by: SplitBy,
}

impl VBox {
    pub fn from(boundaries: VBoxBoundaries, color_hist: &ColorHist) -> Self {
        let mut new_boundaries = VBoxBoundaries::from(
            boundaries.r_max,
            boundaries.r_min,
            boundaries.g_max,
            boundaries.g_min,
            boundaries.b_max,
            boundaries.b_min,
        );
        boundaries.iterate(|color, r, g, b| {
            if color_hist.map[color as usize] > 0 {
                new_boundaries.r_min = new_boundaries.r_min.min(r);
                new_boundaries.r_max = new_boundaries.r_max.max(r);
                new_boundaries.g_min = new_boundaries.g_min.min(g);
                new_boundaries.g_max = new_boundaries.g_max.max(g);
                new_boundaries.b_min = new_boundaries.b_min.min(b);
                new_boundaries.b_max = new_boundaries.b_max.max(b);
            }
        });
        let (r_delta, g_delta, b_delta) = new_boundaries.dimensions();
        let volume = new_boundaries.volume();
        let max_delta = r_delta.max(g_delta).max(b_delta);
        let split_by = if max_delta == r_delta {
            SplitBy::Red
        } else if max_delta == g_delta {
            SplitBy::Green
        } else {
            SplitBy::Blue
        };
        let mut counts = [0; RGB_COMPONENT_SIZE];
        new_boundaries.iterate(|color, r, g, b| {
            let index = match split_by {
                SplitBy::Red => r,
                SplitBy::Green => g,
                SplitBy::Blue => b,
            } as usize;
            counts[index] += color_hist.map[color as usize];
        });
        let a = Self {
            boundaries: new_boundaries,
            counts,
            volume,
            split_by,
        };
        println!("{a:?}");
        a
    }

    pub fn split(&self, color_hist: &ColorHist) -> (VBox, VBox) {
        let boundaries = self.boundaries;
        let (start, end) = match self.split_by {
            SplitBy::Red => (boundaries.r_min, boundaries.r_max),
            SplitBy::Green => (boundaries.g_min, boundaries.g_max),
            SplitBy::Blue => (boundaries.b_min, boundaries.b_max),
        };
        let count = self.counts.iter().sum::<u32>();
        let mut split_at_1 = 0;
        let mut split_count_1 = 0;
        for i in start..end {
            split_at_1 = i as usize + 1;
            split_count_1 += self.counts[i as usize];
            if split_count_1 >= count / 2 {
                break;
            }
        }
        let split_delta_1 = split_count_1.abs_diff(count - split_count_1);
        let mut split_at_2 = 0;
        let mut split_count_2 = 0;
        for i in (start..end).rev() {
            split_at_2 = i as usize;
            split_count_2 += self.counts[i as usize];
            if split_count_2 >= count / 2 {
                break;
            }
        }
        let split_delta_2 = split_count_2.abs_diff(count - split_count_2);
        let split_at = if split_delta_1 < split_delta_2 {
            split_at_1
        } else {
            split_at_2
        } as u8;
        let (boundaries_left, boundaries_right) = match self.split_by {
            SplitBy::Red => (
                VBoxBoundaries {
                    r_max: split_at,
                    ..boundaries
                },
                VBoxBoundaries {
                    r_min: split_at + 1,
                    ..boundaries
                },
            ),
            SplitBy::Green => (
                VBoxBoundaries {
                    g_max: split_at,
                    ..boundaries
                },
                VBoxBoundaries {
                    g_min: split_at + 1,
                    ..boundaries
                },
            ),
            SplitBy::Blue => (
                VBoxBoundaries {
                    b_max: split_at,
                    ..boundaries
                },
                VBoxBoundaries {
                    b_min: split_at + 1,
                    ..boundaries
                },
            ),
        };
        (
            VBox::from(boundaries_left, color_hist),
            VBox::from(boundaries_right, color_hist),
        )
    }
}

struct MedianCutQueue {
    stack: [VBox; MAX_PALETTE_COLORS],
    count: usize,
}

impl MedianCutQueue {
    pub fn new() -> Self {
        Self {
            stack: [VBox::default(); MAX_PALETTE_COLORS],
            count: 0,
        }
    }

    pub fn has_splittable(&self) -> bool {
        if self.is_empty() {
            return false;
        }
        let vbox = self.stack[0];
        let (r_delta, g_delta, b_delta) = vbox.boundaries.dimensions();
        match vbox.split_by {
            SplitBy::Red => r_delta > 1,
            SplitBy::Green => g_delta > 1,
            SplitBy::Blue => b_delta > 1,
        }
    }

    pub fn put(&mut self, vbox: VBox) {
        let mut pos = 0;
        for _ in 0..self.count {
            if vbox.volume < self.stack[pos].volume {
                pos += 1;
            } else {
                break;
            }
        }
        for i in (pos + 1..=self.count).rev() {
            self.stack[i] = self.stack[i - 1];
        }
        self.stack[pos] = vbox;
        self.count += 1;
    }

    pub fn pop(&mut self) -> VBox {
        self.count -= 1;
        self.stack[self.count]
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.count
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub fn median_cut(palette: &mut ColorPalette, palette_size: usize) {
    let mut queue = MedianCutQueue::new();
    let vbox = VBox::from(
        VBoxBoundaries::from(
            0,
            RGB_COMPONENT_SIZE as u8 - 1,
            0,
            RGB_COMPONENT_SIZE as u8 - 1,
            0,
            RGB_COMPONENT_SIZE as u8 - 1,
        ),
        &palette.color_hist,
    );
    queue.put(vbox);
    eprintln!("median_cut: requested palette size: {}", palette_size);
    while queue.has_splittable() && queue.len() < palette_size {
        let vbox = queue.pop();
        let (left, right) = vbox.split(&palette.color_hist);
        queue.put(left);
        queue.put(right);
    }
    while !queue.is_empty() {
        let vbox = queue.pop();
        let mut color_sum: u32 = 0;
        vbox.boundaries.iterate(|color, _, _, _| {
            if palette.color_hist.map[color as usize] > 0 {
                color_sum += color as u32;
            }
        });
        let color_count = vbox.counts.iter().sum::<u32>();
        let color = color_sum / color_count;
        vbox.boundaries.iterate(|color, _, _, _| {
            if palette.color_hist.map[color as usize] > 0 {
                palette.color_hist.map[color as usize] = palette.count as u32;
            }
        });
        palette.colors[palette.count] = u16_to_rgb(color as u16);
        palette.count += 1;
    }
    eprintln!("median_cut: colors_count: {}", palette.count);
}

pub struct ColorHist {
    map: [u32; MAX_HIST_COLORS],
    count: usize,
}

impl ColorHist {
    pub fn from_pixels(pixels: &[Rgb<u8>]) -> Self {
        let mut map = [0; MAX_HIST_COLORS];
        let mut count = 0;
        for rgb in pixels {
            let key = rgb_to_u16(*rgb) as usize;
            if map[key] == 0 {
                map[key] = key as u32;
                count += 1;
            }
            map[key] += 1;
        }
        Self { map, count }
    }
}

pub struct ColorPalette {
    colors: [Rgb<u8>; MAX_PALETTE_COLORS],
    color_hist: ColorHist,
    count: usize,
}

impl ColorPalette {
    pub fn from_pixels(pixels: &[Rgb<u8>], palette_size: usize) -> Self {
        let palette_size = palette_size.min(MAX_PALETTE_COLORS);
        let mut palette = Self {
            colors: [Rgb::from([0, 0, 0]); MAX_PALETTE_COLORS],
            color_hist: ColorHist::from_pixels(pixels),
            count: 0,
        };
        if palette_size >= palette.color_hist.count {
            for color in 0..palette.color_hist.map.len() {
                if palette.color_hist.map[color] > 0 {
                    palette.colors[palette.count] = u16_to_rgb(color as u16);
                    palette.color_hist.map[color] = palette.count as u32;
                    palette.count += 1;
                }
            }
        } else {
            median_cut(&mut palette, palette_size);
        }
        palette
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.count
    }

    #[inline]
    pub fn get_palette(&self) -> &[Rgb<u8>] {
        &self.colors[0..self.len()]
    }

    #[inline]
    pub fn get_index(&self, rgb: Rgb<u8>) -> usize {
        self.color_hist.map[rgb_to_u16(rgb) as usize] as usize
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
