use crate::median_cut::{rgb_to_u16, u16_to_rgb, ColorQuantizer, MAX_HIST_COLORS};
use image::{ImageReader, ImageResult, Rgb, RgbImage};
use std::io::{Error, Write};
use std::time::SystemTime;

#[derive(Default, Debug)]
struct SixelBuf {
    align: usize,
    sixel: usize,
    count: usize,
    result: Option<String>,
}

impl SixelBuf {
    fn add(&mut self, sixel: usize, align: usize) {
        self.align = align;
        if self.count == 0 {
            self.sixel = sixel;
            self.count += 1;
        } else if sixel != self.sixel {
            self.upd_result();
            self.sixel = sixel;
            self.count = 1;
        } else {
            self.count += 1;
        }
    }

    fn upd_result(&mut self) {
        let c: u8 = (1 << (self.align % 6)) + 63;
        if self.count == 1 {
            self.result = Some(format!("#{}{}", self.sixel, c as char));
        } else {
            self.result = Some(format!("#{}!{}{}", self.sixel, self.count, c as char));
        }
    }

    fn flush(&mut self) {
        self.upd_result();
        self.count = 0;
    }

    fn take(&mut self) -> Option<String> {
        self.result.take()
    }
}

#[derive(Default)]
pub struct SixelEncoder {
    rgb8_img: RgbImage,
}

impl SixelEncoder {
    pub fn from(img_path: &str) -> ImageResult<Self> {
        let rgb8_img = ImageReader::open(img_path)?.decode()?.to_rgb8();
        Ok(Self { rgb8_img })
    }

    pub fn image_to_sixel<W: Write>(
        &self,
        w: &mut W,
        palette_size: usize,
        is_dither: bool,
    ) -> Result<(), Error> {
        let palette = ColorQuantizer::from(&self.rgb8_img, palette_size);
        let width = self.rgb8_img.width() as usize;
        let height = self.rgb8_img.height() as usize;

        write!(w, "\x1bPq\"")?;
        write!(w, "1;1;{};{}", width, height)?;
        // if debug {
        //     sixels.push(b'\n');
        // }

        palette
            .get_palette()
            .iter()
            .copied()
            .enumerate()
            .try_for_each(|(i, color)| {
                let r = color[0] as u16 * 100 / 255;
                let g = color[1] as u16 * 100 / 255;
                let b = color[2] as u16 * 100 / 255;
                write!(w, "#{};2;{};{};{}", i, r, g, b)
            })?;
        // if debug {
        //     sixels.push(b'\n');
        // }

        let pixels = if is_dither {
            dither(&self.rgb8_img, &palette)
        } else {
            self.rgb8_img.pixels().copied().collect()
        };

        let mut sixel_buf = SixelBuf::default();
        for y in 0..height {
            for x in 0..width {
                let p_i = palette.get_index(pixels[y * width + x]);
                sixel_buf.add(p_i, y);
                if let Some(sixel) = sixel_buf.take() {
                    // eprintln!("{i}, {j}");
                    write!(w, "{sixel}")?;
                }
            }
            sixel_buf.flush();
            if let Some(sixel) = sixel_buf.take() {
                // eprintln!("flush");
                write!(w, "{sixel}")?;
            }
            if y < height - 1 {
                if y % 6 == 5 {
                    write!(w, "-")?;
                } else {
                    write!(w, "$")?;
                }
            }
            // if debug {
            //     sixels.push(b'\n');
            // }
        }

        write!(w, "\x1b\\")?;
        Ok(())
    }
}

pub fn dither(img: &RgbImage, palette: &ColorQuantizer) -> Vec<Rgb<u8>> {
    let height = img.height() as usize;
    let width = img.width() as usize;
    let u16_color_to_f32 = |color: u16| color as f32 / (MAX_HIST_COLORS - 1) as f32;
    let mut pixels: Vec<f32> = img
        .pixels()
        .map(|rgb| u16_color_to_f32(rgb_to_u16(*rgb)))
        .collect();
    let get_index = |x: usize, y: usize| y * width + x;
    let f32_color_to_u16 = |color: f32| {
        let color = (color * (MAX_HIST_COLORS - 1) as f32) as i32;
        if color < 0 {
            0u16
        } else if color > MAX_HIST_COLORS as i32 - 1 {
            MAX_HIST_COLORS as u16 - 1
        } else {
            color as u16
        }
    };
    let match_color = |color: f32| {
        let color = f32_color_to_u16(color);
        let color = palette.get_palette()[palette.get_index(u16_to_rgb(color))];
        u16_color_to_f32(rgb_to_u16(color))
    };
    for y in 0..height {
        for x in 0..width {
            let old_pixel = &mut pixels[get_index(x, y)];
            let new_pixel = match_color(*old_pixel);
            let error = *old_pixel - new_pixel;
            *old_pixel = new_pixel;
            if x < width - 1 {
                *&mut pixels[get_index(x + 1, y)] += error * 7.0 / 16.0;
            }
            if y < height - 1 {
                if x > 0 {
                    *&mut pixels[get_index(x - 1, y + 1)] += error * 3.0 / 16.0;
                }
                *&mut pixels[get_index(x, y + 1)] += error * 5.0 / 16.0;
                if x < width - 1 {
                    *&mut pixels[get_index(x + 1, y + 1)] += error * 1.0 / 16.0;
                }
            }
        }
    }
    pixels
        .iter()
        .copied()
        .map(f32_color_to_u16)
        .map(u16_to_rgb)
        .collect()
}
