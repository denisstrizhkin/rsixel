mod octree;

use self::octree::Octree;
use image::DynamicImage;

pub fn image_to_sixel(img: DynamicImage) -> Vec<u8> {
    let img = img.to_rgb8();
    let mut octree = Octree::new(2);
    img.pixels().for_each(|p| {
        octree.add_color(p);
    });
    octree.build_palette();

    let width = img.width();
    let height = img.height();
    let mut sixels = Vec::new();

    sixels.extend_from_slice(b"\x1bPq\"1;1;");
    sixels.extend_from_slice(format!("{};{}", width, height).as_bytes());

    octree
        .get_palette()
        .iter()
        .enumerate()
        .for_each(|(i, rgb)| {
            let r = rgb[0] as u16 * 100 / 255;
            let g = rgb[1] as u16 * 100 / 255;
            let b = rgb[2] as u16 * 100 / 255;
            sixels.extend_from_slice(format!("#{};2;{};{};{}", i, r, g, b).as_bytes());
        });

    for i in 0..height {
        for j in 0..width {
            let p_i = octree.get_palette_index(img.get_pixel(j, i));
            let c: u8 = (1 << (i % 6)) + 63;
            sixels.extend_from_slice(format!("#{}", p_i).as_bytes());
            sixels.push(c);
        }
        if i % 6 == 5 {
            sixels.push(b'-');
        } else {
            sixels.push(b'$');
        }
    }

    sixels.extend_from_slice(b"\x1b\\");
    sixels
}

// func sixel_encode(img image.Image, w io.Writer) {
// 	bw := bufio.NewWriter(w)
// 	defer bw.Flush()

// 	width := img.Bounds().Dx()
// 	height := img.Bounds().Dy()
// 	header := fmt.Sprintf("\x1bPq\"1;1;%d;%d", width, height)
// 	bw.Write([]byte(header))

// 	pixels := colors_to_pixels(img)
// 	palette, clusterMap := sixel.Clusterize(pixels, 256, 100)
// 	//save_palette(palette)

// 	for i, p := range palette {
// 		r := p.R * 100 / 255
// 		g := p.G * 100 / 255
// 		b := p.B * 100 / 255
// 		bw.Write([]byte(fmt.Sprintf("#%d;2;%d;%d;%d", i, r, g, b)))
// 	}

// 	for i := range height {
// 		for j := range width {
// 			p_id := clusterMap[pixels[i*width+j]]
// 			c := rune((1 << (i % 6)) + 63)
// 			bw.Write([]byte(fmt.Sprintf("#%d%c", p_id, c)))
// 		}
// 		if i%6 == 5 {
// 			bw.Write([]byte("-"))
// 		} else {
// 			bw.Write([]byte("$"))
// 		}
// 	}

// 	bw.Write([]byte("\x1b\\"))
// }
