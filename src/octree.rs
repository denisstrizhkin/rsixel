use image::{imageops::ColorMap, Rgb, RgbImage};
use std::{array, iter};

const MAX_LEVEL: u8 = 8;

#[inline(always)]
fn get_color_index(color: Rgb<u8>, level: u8) -> usize {
    let shift = MAX_LEVEL - level;
    color
        .0
        .into_iter()
        .rev()
        .enumerate()
        .map(|(i, c)| (((c >> shift) & 1) << i) as usize)
        .fold(0, |s, c| s | c)
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

    fn insert(&mut self, color: Rgb<u8>) {
        let mut current_node_id = OctreeNodeId { level: 1, index: 0 };
        while current_node_id.level < MAX_LEVEL {
            let child_index = get_color_index(color, current_node_id.level);
            current_node_id = match self.get_node(current_node_id).children[child_index] {
                Some(child_id) => child_id,
                None => self.insert_child(current_node_id, child_index),
            }
        }
        assert_eq!(current_node_id.level, MAX_LEVEL);
        self.get_node_mut(current_node_id).add_color(color);
    }

    fn get_index(&self, color: Rgb<u8>) -> usize {
        let mut current_node_id = OctreeNodeId { level: 1, index: 0 };
        while !self.get_node(current_node_id).is_leaf() {
            let child_index = get_color_index(color, current_node_id.level);
            let children = self.get_node(current_node_id).children;
            current_node_id = match children[child_index] {
                Some(child_id) => child_id,
                None => {
                    match children
                        .iter()
                        .enumerate()
                        .filter_map(|(i, c)| c.zip(Some(i)))
                        .map(|(c, i)| {
                            let d = child_index ^ i;
                            ([d >> 2, (d >> 1) & 1, d & 1].into_iter().sum::<usize>(), c)
                        })
                        .min_by(|a, b| a.0.cmp(&b.0))
                        .map(|(_, c)| c)
                    {
                        Some(child_id) => child_id,
                        None => break,
                    }
                }
            }
        }
        self.get_node(current_node_id).index
    }

    fn prune_node(&mut self, node_id: OctreeNodeId) -> usize {
        std::mem::take(&mut self.get_node_mut(node_id).children)
            .into_iter()
            .flatten()
            .inspect(|&child_id| {
                let child = std::mem::take(self.get_node_mut(child_id));
                self.get_node_mut(node_id).add_node(child);
            })
            .count()
    }

    fn reduce_to(&mut self, color_count: usize) {
        let mut color_count_current = self.get_level(MAX_LEVEL).len();
        if color_count_current > color_count {
            'main: for level in (1..MAX_LEVEL).rev() {
                for index in 0..self.get_level(level).len() {
                    let node_id = OctreeNodeId { level, index };
                    color_count_current -= self.prune_node(node_id).saturating_sub(1);
                    if color_count_current <= color_count {
                        break 'main;
                    }
                }
            }
        }
        assert!(
            color_count_current <= color_count,
            "Color palette size {color_count_current} exceeded {color_count}",
        );
    }

    fn finalize(&mut self) -> Vec<Rgb<u8>> {
        let mut palette = Vec::new();
        for level in 1..=MAX_LEVEL {
            for index in 0..self.get_level(level).len() {
                let node_id = OctreeNodeId { level, index };
                if self.get_node(node_id).is_leaf() {
                    self.get_node_mut(node_id).index = palette.len();
                    palette.push(Rgb::from(self.get_node(node_id)));
                }
            }
        }
        palette
    }

    fn insert_child(&mut self, node_id: OctreeNodeId, child_index: usize) -> OctreeNodeId {
        assert!(child_index < 8);
        assert!(node_id.level < MAX_LEVEL);
        let level = node_id.level + 1;
        let child_id = OctreeNodeId {
            level,
            index: self.get_level(level).len(),
        };
        self.get_level_mut(level).push(OctreeNode::default());
        assert!(self.get_node(node_id).children[child_index].is_none());
        self.get_node_mut(node_id).children[child_index] = Some(child_id);
        child_id
    }

    #[inline(always)]
    fn get_level(&self, level: u8) -> &Vec<OctreeNode> {
        &self.levels[(level - 1) as usize]
    }

    #[inline(always)]
    fn get_level_mut(&mut self, level: u8) -> &mut Vec<OctreeNode> {
        &mut self.levels[(level - 1) as usize]
    }

    #[inline(always)]
    fn get_node(&self, node_id: OctreeNodeId) -> &OctreeNode {
        &self.get_level(node_id.level)[node_id.index]
    }

    #[inline(always)]
    fn get_node_mut(&mut self, node_id: OctreeNodeId) -> &mut OctreeNode {
        &mut self.get_level_mut(node_id.level)[node_id.index]
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct OctreeNodeId {
    level: u8,
    index: usize,
}

#[derive(Debug, Default, Clone)]
struct OctreeNode {
    color: [u32; 3],
    count: u32,
    children: [Option<OctreeNodeId>; 8],
    index: usize,
}

impl From<&OctreeNode> for Rgb<u8> {
    #[inline(always)]
    fn from(node: &OctreeNode) -> Self {
        if node.count == 0 {
            Rgb::from([0, 0, 0])
        } else {
            Rgb::from(array::from_fn(|i| (node.color[i] / node.count) as u8))
        }
    }
}

impl OctreeNode {
    #[inline(always)]
    fn is_leaf(&self) -> bool {
        self.count != 0 && self.children.iter().all(Option::is_none)
    }

    #[inline(always)]
    fn add_color(&mut self, color: Rgb<u8>) {
        self.count += 1;
        iter::zip(&mut self.color, color.0).for_each(|(a, b)| *a += b as u32)
    }

    #[inline(always)]
    fn add_node(&mut self, node: OctreeNode) {
        self.count += node.count;
        iter::zip(&mut self.color, node.color).for_each(|(a, b)| *a += b)
    }
}

pub struct ColorQuantizer {
    octree: Octree,
    colors: Vec<Rgb<u8>>,
}

impl ColorQuantizer {
    pub fn from(img: &RgbImage, palette_size: usize) -> Self {
        let palette_size = palette_size.min(256);
        let mut octree = Octree::new();
        for pixel in img.pixels() {
            octree.insert(*pixel);
        }
        octree.reduce_to(palette_size);
        let colors = octree.finalize();
        println!("final color palette size: {}", colors.len());
        Self { octree, colors }
    }

    #[inline(always)]
    pub fn get_palette(&self) -> &[Rgb<u8>] {
        &self.colors
    }

    #[inline(always)]
    pub fn get_index(&self, color: &Rgb<u8>) -> usize {
        self.octree.get_index(*color)
    }
}

impl ColorMap for ColorQuantizer {
    type Color = Rgb<u8>;

    #[inline(always)]
    fn index_of(&self, color: &Self::Color) -> usize {
        self.get_index(color)
    }

    #[inline(always)]
    fn map_color(&self, color: &mut Self::Color) {
        *color = self.get_palette()[self.get_index(color)]
    }
}
