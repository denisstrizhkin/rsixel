// use crate::median_cut::ColorQuantizer;
use crate::octree::ColorQuantizer;
use image::imageops::dither;
use image::{ImageReader, ImageResult, RgbImage};
use std::io::{Error, Write};

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
        &mut self,
        w: &mut W,
        palette_size: usize,
        is_dither: bool,
    ) -> Result<(), Error> {
        let palette = ColorQuantizer::from(&self.rgb8_img, palette_size);
        let width = self.rgb8_img.width() as usize;
        let height = self.rgb8_img.height() as usize;

        write!(w, "\x1bPq\"")?;
        write!(w, "1;1;{width};{height}")?;
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
                write!(w, "#{i};2;{r};{g};{b}")
            })?;
        // if debug {
        //     sixels.push(b'\n');
        // }

        if is_dither {
            dither(&mut self.rgb8_img, &palette);
        }

        let mut sixel_buf = SixelBuf::default();
        for y in 0..height {
            for x in 0..width {
                let p_i = palette.get_index(self.rgb8_img.get_pixel(x as u32, y as u32));
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
