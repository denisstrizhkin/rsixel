use image::DynamicImage;

pub fn image_to_sixel(img: DynamicImage) -> Vec<u8> {
    todo!();
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
