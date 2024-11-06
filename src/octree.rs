use image::Rgb;

const MAX_LEVEL: u8 = 8;

fn get_rgb_index(color: Rgb<u8>, level: u8) -> u8 {
    let level = level.min(MAX_LEVEL);
    let r = (color[0] >> (8 - level)) & 0b1;
    let g = (color[1] >> (8 - level)) & 0b1;
    let b = (color[2] >> (8 - level)) & 0b1;
    (r << 2) + (g << 1) + b
}

pub struct Octree {
    max_level: u8,
    root: Node,
}

impl Octree {
    pub fn new(max_level: u8) -> Self {
        Self {
            max_level: max_level.min(MAX_LEVEL),
            root: Node::default(),
        }
    }

    pub fn add_color(&mut self, color: &Rgb<u8>) {
        self.root.add_color(color, 1);
    }
}

#[derive(Default)]
struct Node {
    red: u64,
    green: u64,
    blue: u64,
    count: usize,
    children: [Option<Box<Node>>; 8],
}

impl Node {
    fn add_color(&mut self, color: &Rgb<u8>, level: u8) {}
}
