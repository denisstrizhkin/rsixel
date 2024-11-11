use image::Rgb;

const MAX_HIST_COLORS: usize = 32 * 32 * 32;
const MAX_PALETTE_COLORS: usize = 256;
const RGB_MASK: u8 = 0b11111000;

pub fn median_cut(colors: &mut [Rgb<u8>], level: usize) -> Vec<Rgb<u8>> {
    eprintln!("{}: level: {}", colors.len(), level);
    if colors.is_empty() {
        vec![]
    } else if level <= 1 {
        let r = (colors.iter().map(|c| c[0] as u32).sum::<u32>() / colors.len() as u32) as u8;
        let g = (colors.iter().map(|c| c[1] as u32).sum::<u32>() / colors.len() as u32) as u8;
        let b = (colors.iter().map(|c| c[2] as u32).sum::<u32>() / colors.len() as u32) as u8;
        vec![Rgb::from([r, g, b])]
    } else {
        let r_max = colors.iter().map(|c| c[0]).max().unwrap_or(u8::MAX);
        let r_min = colors.iter().map(|c| c[0]).min().unwrap_or(u8::MIN);
        let g_max = colors.iter().map(|c| c[1]).max().unwrap_or(u8::MAX);
        let g_min = colors.iter().map(|c| c[1]).min().unwrap_or(u8::MIN);
        let b_max = colors.iter().map(|c| c[2]).max().unwrap_or(u8::MAX);
        let b_min = colors.iter().map(|c| c[2]).min().unwrap_or(u8::MIN);
        let r_delta = r_max - r_min;
        let g_delta = g_max - g_min;
        let b_delta = b_max - b_min;
        let max_delta = r_delta.max(g_delta.max(b_delta));
        if max_delta == r_delta {
            colors.sort_by(|a, b| a[0].cmp(&b[0]));
        } else if max_delta == g_delta {
            colors.sort_by(|a, b| a[1].cmp(&b[1]));
        } else {
            colors.sort_by(|a, b| a[2].cmp(&b[2]));
        }
        let (left, right) = colors.split_at_mut(colors.len() / 2);
        let mut left = median_cut(left, level - 1);
        let mut right = median_cut(right, level - 1);
        left.append(&mut right);
        left
    }
}

pub struct ColorHist {
    map: [u16; MAX_HIST_COLORS],
    count: usize,
}

impl ColorHist {
    pub fn from_pixels(pixels: &[Rgb<u8>]) -> Self {
        let mut map = [0; MAX_HIST_COLORS];
        let mut count = 0;
        for rgb in pixels {
            let key = rgb_to_u16(*rgb) as usize;
            if map[key] == 0 {
                count += 1;
            }
            map[key] += 1;
        }
        map.sort_by(|a, b| b.cmp(a));
        Self { map, count }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.count
    }
}

#[inline]
pub fn rgb_to_u16(rgb: Rgb<u8>) -> u16 {
    (((RGB_MASK & rgb[0]) as u16) << 7)
        + (((RGB_MASK & rgb[1]) as u16) << 2)
        + (((RGB_MASK & rgb[2]) as u16) >> 3)
}

#[inline]
pub fn u16_to_rgb(rgb: u16) -> Rgb<u8> {
    Rgb::from([
        ((rgb >> 7) as u8) & RGB_MASK,
        ((rgb >> 2) as u8) & RGB_MASK,
        (rgb << 3) as u8,
    ])
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
