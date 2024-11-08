mod median_cut;
mod octree;

use self::octree::Octree;
use image::{DynamicImage, ImageBuffer, Rgb};
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
pub struct SixelEncoder {}

impl SixelEncoder {
    fn setup(&self, img: DynamicImage) -> (ImageBuffer<Rgb<u8>, Vec<u8>>, Octree) {
        let start_time = SystemTime::now();

        let img = img.to_rgb8();
        let mut octree = Octree::new(3);
        img.pixels().for_each(|p| {
            octree.add_color(p);
        });
        octree.build_palette();

        eprintln!(
            "Build color palette: {} colors, {}ms",
            octree.get_palette().len(),
            start_time.elapsed().unwrap().as_millis()
        );

        (img, octree)
    }

    pub fn image_to_sixel<W: Write>(&self, img: DynamicImage, w: &mut W) -> Result<(), Error> {
        let start_time = SystemTime::now();

        let (img, octree) = self.setup(img);
        let width = img.width();
        let height = img.height();

        write!(w, "\x1bPq\"")?;
        write!(w, "1;1;{};{}", width, height)?;
        // if debug {
        //     sixels.push(b'\n');
        // }

        octree
            .get_palette()
            .iter()
            .enumerate()
            .try_for_each(|(i, rgb)| {
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
                let p_i = octree.get_palette_index(img.get_pixel(j, i));
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
