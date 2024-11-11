use super::median_cut::{u16_to_rgb, ColorHist, ColorPalette};
use super::octree::Octree;
use image::{ImageReader, ImageResult, Rgb, RgbImage};
use std::io::{Error, Write};
use std::time::SystemTime;

#[derive(Default, Debug)]
struct SixelBuf {
    align: u32,
    sixel: usize,
    count: usize,
    result: Option<String>,
}

impl SixelBuf {
    fn add(&mut self, sixel: usize, align: u32) {
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

    pub fn color_hist(&self) -> ColorHist {
        let pixels: Vec<Rgb<u8>> = self.rgb8_img.pixels().copied().collect();
        ColorHist::from_pixels(&pixels)
    }

    pub fn image_to_sixel<W: Write>(&self, w: &mut W, palette_size: usize) -> Result<(), Error> {
        let color_hist = self.color_hist();
        let mut palette = ColorPalette::from_color_hist(color_hist, palette_size);
        let width = self.rgb8_img.width();
        let height = self.rgb8_img.height();

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
                let rgb = u16_to_rgb(color);
                let r = rgb[0] as u16 * 100 / 255;
                let g = rgb[1] as u16 * 100 / 255;
                let b = rgb[2] as u16 * 100 / 255;
                write!(w, "#{};2;{};{};{}", i, r, g, b)
            })?;
        // if debug {
        //     sixels.push(b'\n');
        // }

        let mut sixel_buf = SixelBuf::default();
        for i in 0..height {
            for j in 0..width {
                let p_i = palette.get_index(*self.rgb8_img.get_pixel(j, i));
                sixel_buf.add(p_i, i);
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
            if i < height - 1 {
                if i % 6 == 5 {
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
