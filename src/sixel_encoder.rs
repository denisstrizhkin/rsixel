// use crate::median_cut::ColorQuantizer;
use crate::octree::ColorQuantizer;
use anyhow::Result;
use image::{
    imageops::dither,
    {ImageReader, RgbImage},
};
use std::{io::Write, path::Path};

const ESC: char = '\x1b';

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
            debug: self.debug,
        })
    }
}

pub struct SixelEncoder {
    rgb8_img: RgbImage,
    debug: bool,
}

impl SixelEncoder {
    pub fn image_to_sixel<W: Write>(
        &mut self,
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
        if self.debug {
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
        if self.debug {
            writeln!(w)?
        }
        let mut sixel_buf = SixelBuf::default();
        for y in 0..height {
            for x in 0..width {
                let p_i = palette.get_index(*self.rgb8_img.get_pixel(x as u32, y as u32));
                sixel_buf.add(p_i, y);
                if let Some(sixel) = sixel_buf.take() {
                    write!(w, "{sixel}")?;
                }
            }
            sixel_buf.flush();
            if let Some(sixel) = sixel_buf.take() {
                write!(w, "{sixel}")?;
            }
            if y < height - 1 {
                if y % 6 == 5 {
                    write!(w, "-")?;
                } else {
                    write!(w, "$")?;
                }
            }
            if self.debug {
                writeln!(w)?;
            }
        }
        write!(w, "{ESC}\\")?;
        Ok(())
    }
}
