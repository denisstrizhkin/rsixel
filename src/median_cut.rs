use image::Rgb;

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
