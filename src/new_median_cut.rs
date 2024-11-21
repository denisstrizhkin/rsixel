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
struct VBox {
    r_min: u8,
    r_max: u8,
    g_min: u8,
    g_max: u8,
    b_min: u8,
    b_max: u8,
    counts: [u32; RGB_COMPONENT_SIZE],
    volume: u16,
    split_by: SplitBy,
}

impl VBox {
    pub fn from(
        color_hist: &ColorHist,
        r_min: u8,
        r_max: u8,
        g_min: u8,
        g_max: u8,
        b_min: u8,
        b_max: u8,
    ) -> Self {
        println!("{r_min}, {r_max}, {g_min}, {g_max}, {b_min}, {b_max}");
        let (mut r_min, mut r_max, mut g_min, mut g_max, mut b_min, mut b_max) =
            (r_max, r_min, g_max, g_min, b_max, b_min);
        println!("{r_min}, {r_max}, {g_min}, {g_max}, {b_min}, {b_max}");
        for r in r_max..r_min {
            for g in g_max..g_min {
                for b in b_max..b_min {
                    println!("{r}, {g}, {b}");
                    let color = rgb_to_u16(Rgb::from([r << 3, g << 3, b << 3])) as usize;
                    if color_hist.map[color] > 0 {
                        r_min = r_min.min(r);
                        r_max = r_max.max(r);
                        g_min = g_min.min(g);
                        g_max = g_max.max(g);
                        b_min = b_min.min(b);
                        b_max = b_max.max(b);
                        println!("{r_min}, {r_max}, {g_min}, {g_max}, {b_min}, {b_max}");
                    }
                }
            }
        }
        let r_delta = r_max - r_min;
        let g_delta = g_max - g_min;
        let b_delta = b_max - b_min;
        let volume = (r_delta as u16) * (g_delta as u16) * (b_delta as u16);
        let max_delta = r_delta.max(g_delta).max(b_delta);
        let split_by = if max_delta == r_delta {
            SplitBy::Red
        } else if max_delta == g_delta {
            SplitBy::Green
        } else {
            SplitBy::Blue
        };
        let mut counts = [0; RGB_COMPONENT_SIZE];
        for r in r_min..=r_max {
            for g in g_min..=g_max {
                for b in b_min..=b_max {
                    let color = rgb_to_u16(Rgb::from([r << 3, g << 3, b << 3])) as usize;
                    let index = match split_by {
                        SplitBy::Red => r,
                        SplitBy::Green => g,
                        SplitBy::Blue => b,
                    };
                    let index = (index >> 3) as usize;
                    counts[index] += color_hist.map[color];
                }
            }
        }
        let a = Self {
            r_min,
            r_max,
            g_min,
            g_max,
            b_min,
            b_max,
            counts,
            volume,
            split_by,
        };
        println!("{a:?}");
        a
    }

    pub fn split(&self, color_hist: &ColorHist) -> (VBox, VBox) {
        let (start, end) = match self.split_by {
            SplitBy::Red => (self.r_min, self.r_max),
            SplitBy::Green => (self.g_min, self.g_max),
            SplitBy::Blue => (self.b_min, self.b_max),
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
        match self.split_by {
            SplitBy::Red => (
                VBox::from(
                    color_hist, self.r_min, split_at, self.g_min, self.g_max, self.b_min,
                    self.b_max,
                ),
                VBox::from(
                    color_hist, split_at, self.r_max, self.g_min, self.g_max, self.b_min,
                    self.b_max,
                ),
            ),
            SplitBy::Green => (
                VBox::from(
                    color_hist, self.r_min, self.r_max, self.g_min, split_at, self.b_min,
                    self.b_max,
                ),
                VBox::from(
                    color_hist, self.r_min, self.r_max, split_at, self.g_max, self.b_min,
                    self.b_max,
                ),
            ),
            SplitBy::Blue => (
                VBox::from(
                    color_hist, self.r_min, self.r_max, self.g_min, self.g_max, self.b_min,
                    split_at,
                ),
                VBox::from(
                    color_hist, self.r_min, self.r_max, self.g_min, self.g_max, split_at,
                    self.b_max,
                ),
            ),
        }
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
        match vbox.split_by {
            SplitBy::Red => (vbox.r_max - vbox.r_min) > 1,
            SplitBy::Green => (vbox.g_max - vbox.g_min) > 1,
            SplitBy::Blue => (vbox.b_max - vbox.b_min) > 1,
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
        &palette.color_hist,
        0,
        RGB_COMPONENT_SIZE as u8,
        0,
        RGB_COMPONENT_SIZE as u8,
        0,
        RGB_COMPONENT_SIZE as u8,
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
        for r in vbox.r_min..vbox.r_max {
            for g in vbox.g_min..vbox.g_max {
                for b in vbox.b_min..vbox.b_max {
                    let color = rgb_to_u16(Rgb::from([r << 3, g << 3, b << 3])) as usize;
                    if palette.color_hist.map[color] > 0 {
                        color_sum += color as u32;
                    }
                }
            }
        }
        let color_count = vbox.counts.iter().sum::<u32>();
        let color = color_sum / color_count;
        for r in vbox.r_min..vbox.r_max {
            for g in vbox.g_min..vbox.g_max {
                for b in vbox.b_min..vbox.b_max {
                    let color = rgb_to_u16(Rgb::from([r << 3, g << 3, b << 3])) as usize;
                    if palette.color_hist.map[color as usize] > 0 {
                        palette.color_hist.map[color as usize] = palette.count as u32;
                    }
                }
            }
        }
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
