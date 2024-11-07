use image::Rgb;

const MAX_LEVEL: u8 = 8;

fn get_rgb_index(color: &Rgb<u8>, level: u8) -> u8 {
    let level = level.min(MAX_LEVEL);
    let r = (color[0] >> (8 - level)) & 0b1;
    let g = (color[1] >> (8 - level)) & 0b1;
    let b = (color[2] >> (8 - level)) & 0b1;
    (r << 2) + (g << 1) + b
}

#[derive(Debug)]
pub struct Octree {
    max_level: u8,
    root: Node,
    palette: Vec<Rgb<u8>>,
}

impl Octree {
    pub fn new(max_level: u8) -> Self {
        Self {
            max_level: max_level.min(MAX_LEVEL),
            root: Node::default(),
            palette: Vec::new(),
        }
    }

    pub fn add_color(&mut self, color: &Rgb<u8>) {
        self.root.add_color(color, 1, self.max_level);
    }

    pub fn build_palette(&mut self) {
        self.root.traverse(|node| {
            node.palette_index = self.palette.len();
            self.palette.push(node.to_rgb());
        });
    }

    pub fn get_palette(&self) -> &[Rgb<u8>] {
        &self.palette
    }

    pub fn get_palette_index(&self, color: &Rgb<u8>) -> usize {
        self.root.get_palette_index(color, 1, self.max_level)
    }

    pub fn get_color(&self, color: &Rgb<u8>) -> &Rgb<u8> {
        &self.palette[self.get_palette_index(color)]
    }
}

#[derive(Default, Debug)]
struct Node {
    red: u64,
    green: u64,
    blue: u64,
    count: usize,
    palette_index: usize,
    children: [Option<Box<Node>>; 8],
}

impl Node {
    fn add_color(&mut self, color: &Rgb<u8>, level: u8, max_level: u8) {
        if level > max_level {
            self.red += color[0] as u64;
            self.green += color[1] as u64;
            self.blue += color[2] as u64;
            self.count += 1;
        } else {
            let index = get_rgb_index(color, level) as usize;
            let child = self.children[index].get_or_insert(Box::new(Node::default()));
            child.add_color(color, level + 1, max_level);
        }
    }

    fn traverse<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Self),
    {
        self.traverse_ref(&mut f);
    }

    fn traverse_ref<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut Self),
    {
        self.children.iter_mut().flatten().for_each(|child| {
            if child.count > 0 {
                f(child);
            }
            child.traverse_ref(f);
        })
    }

    fn get_palette_index(&self, color: &Rgb<u8>, level: u8, max_level: u8) -> usize {
        if level > max_level {
            self.palette_index
        } else {
            let index = get_rgb_index(color, level) as usize;
            let child = self.children[index]
                .as_ref()
                .expect("This should not happen, with healthy octree");
            if self.count != 0 {
                child.palette_index
            } else {
                child.get_palette_index(color, level + 1, max_level)
            }
        }
    }

    fn to_rgb(&self) -> Rgb<u8> {
        let r = (self.red / self.count as u64) as u8;
        let g = (self.green / self.count as u64) as u8;
        let b = (self.blue / self.count as u64) as u8;
        Rgb::from([r, g, b])
    }
}
