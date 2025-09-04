use image::{imageops::ColorMap, Rgb, RgbImage};
use std::cmp;

const MAX_LEVEL: u8 = 3;

#[inline(always)]
fn get_color_index(color: &Rgb<u8>, level: u8) -> usize {
    let r = (color[0] >> (MAX_LEVEL - level)) & 1;
    let g = (color[1] >> (MAX_LEVEL - level)) & 1;
    let b = (color[2] >> (MAX_LEVEL - level)) & 1;
    (r << 2 | g << 1 | b) as usize
}

struct Octree {
    levels: [Vec<OctreeNode>; MAX_LEVEL as usize],
}

impl Octree {
    fn new() -> Self {
        let mut octree = Self {
            levels: Default::default(),
        };
        octree.levels[0].push(OctreeNode::default());
        octree
    }

    fn insert(&mut self, color: &Rgb<u8>) {
        let mut node_index_current = 0;
        for level in 1..MAX_LEVEL {
            let child_index = get_color_index(color, level);
            if self.get_node(level, node_index_current).children[child_index].is_none() {
                let node_index = self.new_node(level + 1);
                self.get_node_mut(level, node_index_current).children[child_index] =
                    Some(node_index);
            }
            if let Some(node_index) = self.get_node(level, node_index_current).children[child_index]
            {
                node_index_current = node_index;
            }
        }
        let node = self.get_node_mut(MAX_LEVEL, node_index_current);
        node.color[0] += color[0] as u32;
        node.color[1] += color[1] as u32;
        node.color[2] += color[2] as u32;
        node.count += 1;
    }

    fn remove_leaves(&mut self, level: u8, node_index: usize) -> usize {
        let mut cnt = 0;
        for child_index in 0..8 {
            if let Some(child_index) =
                self.get_node_mut(level, node_index).children[child_index].take()
            {
                let child = self.get_node(level + 1, child_index).clone();
                let node = self.get_node_mut(level, node_index);
                node.count += child.count;
                node.color[0] += child.color[0];
                node.color[1] += child.color[1];
                node.color[2] += child.color[2];
                cnt += 1;
            }
        }
        cnt
    }

    fn reduce_to(&mut self, color_count: usize) -> Vec<Rgb<u8>> {
        let mut color_count_current = self.get_level(MAX_LEVEL).len();
        for level in (1..MAX_LEVEL).rev() {
            for node_index in 0..self.get_level(level).len() {
                color_count_current -= self.remove_leaves(level, node_index);
                if color_count_current <= color_count {
                    break;
                }
            }
        }
        println!("{:?}", self.levels[0]);
        println!("{:?}", self.levels[1]);
        (1..=MAX_LEVEL)
            .flat_map(|level| self.get_level(level))
            .filter(|node| node.is_leaf())
            .map(|node| {
                Rgb::from([
                    (node.color[0] / node.count) as u8,
                    (node.color[1] / node.count) as u8,
                    (node.color[2] / node.count) as u8,
                ])
            })
            .collect()
    }

    fn new_node(&mut self, level: u8) -> usize {
        let level = self.get_level_mut(level);
        let node_index = level.len();
        level.push(OctreeNode::default());
        node_index
    }

    fn get_level(&self, level: u8) -> &Vec<OctreeNode> {
        &self.levels[(level - 1) as usize]
    }

    fn get_level_mut(&mut self, level: u8) -> &mut Vec<OctreeNode> {
        &mut self.levels[(level - 1) as usize]
    }

    fn get_node(&self, level: u8, node_index: usize) -> &OctreeNode {
        &self.get_level(level)[node_index]
    }

    fn get_node_mut(&mut self, level: u8, node_index: usize) -> &mut OctreeNode {
        &mut self.get_level_mut(level)[node_index]
    }
}

#[derive(Debug, Default, Clone)]
struct OctreeNode {
    color: [u32; 3],
    count: u32,
    children: [Option<usize>; 8],
}

impl OctreeNode {
    #[inline(always)]
    fn is_leaf(&self) -> bool {
        self.children.iter().all(Option::is_none)
    }
}

fn compare_colors(a: &Rgb<u8>, b: &Rgb<u8>) -> cmp::Ordering {
    match a[0].cmp(&b[0]) {
        cmp::Ordering::Equal => match a[1].cmp(&b[1]) {
            cmp::Ordering::Equal => a[2].cmp(&b[2]),
            other => other,
        },
        other => other,
    }
}

pub struct ColorQuantizer {
    colors: Vec<Rgb<u8>>,
}

impl ColorQuantizer {
    pub fn from(img: &RgbImage, palette_size: usize) -> Self {
        let palette_size = palette_size.min(256);
        let mut octree = Octree::new();
        for pixel in img.pixels() {
            octree.insert(pixel);
        }
        let mut colors = octree.reduce_to(palette_size);
        colors.sort_by(compare_colors);
        Self { colors }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.colors.len()
    }

    #[inline]
    pub fn get_palette(&self) -> &[Rgb<u8>] {
        &self.colors
    }

    #[inline]
    pub fn get_index(&self, color: &Rgb<u8>) -> usize {
        match self.colors.binary_search_by(|c| compare_colors(c, color)) {
            Ok(index) => index,
            Err(index) => index.clamp(0, self.colors.len() - 1),
        }
    }
}

impl ColorMap for ColorQuantizer {
    type Color = Rgb<u8>;

    #[inline]
    fn index_of(&self, color: &Self::Color) -> usize {
        self.get_index(color)
    }

    #[inline]
    fn map_color(&self, color: &mut Self::Color) {
        *color = self.get_palette()[self.get_index(color)]
    }
}
