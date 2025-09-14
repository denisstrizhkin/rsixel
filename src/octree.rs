use image::{imageops::ColorMap, Rgb, RgbImage};
use std::{array, collections::VecDeque, iter};

use crate::queue::Queue;

const MAX_LEVEL: u8 = 6;
const MAX_NODES: usize = 768;
const MAX_REDUCIBLE: usize = 256;
const MAX_COLORS: usize = 256;

#[derive(Debug, Default)]
struct Node {
    rgb: [u32; 3],
    count: u32,
    index: u8,
    level: u8,
    children_count: u8,
    is_leaf: bool,
    parent: Option<u32>,
    children: [Option<u32>; 8],
}

impl Node {
    fn merge_color(&mut self, color: Rgb<u8>) {
        self.count += 1;
        iter::zip(&mut self.rgb, color.0).for_each(|(a, b)| *a += b as u32)
    }

    fn merge_node(&mut self, node: Node) {
        self.count += node.count;
        iter::zip(&mut self.rgb, node.rgb).for_each(|(a, b)| *a += b)
    }
}

#[derive(Debug)]
struct Pool {
    nodes: [Node; MAX_NODES],
    ids: Queue<u32, MAX_NODES>,
}

impl Pool {
    fn new() -> Self {
        let mut ids = Queue::new();
        let nodes = array::from_fn(|i| {
            ids.push(i as u32);
            Node::default()
        });
        Self { nodes, ids }
    }

    fn create(&mut self) -> u32 {
        self.ids.pop().unwrap()
    }

    fn get(&self, id: u32) -> &Node {
        &self.nodes[id as usize]
    }

    fn get_mut(&mut self, id: u32) -> &mut Node {
        &mut self.nodes[id as usize]
    }

    fn delete(&mut self, id: u32) -> Node {
        self.ids.push(id);
        std::mem::take(&mut self.nodes[id as usize])
    }
}

fn get_color_index(color: Rgb<u8>, level: u8) -> usize {
    let shift = 8 - level;
    color
        .0
        .into_iter()
        .rev()
        .enumerate()
        .map(|(i, c)| (((c >> shift) & 1) << i) as usize)
        .fold(0, |s, c| s | c)
}

#[derive(Debug)]
struct Reducible {
    levels: [Queue<u32, MAX_REDUCIBLE>; MAX_LEVEL as usize],
}

impl Reducible {
    fn new() -> Self {
        Self {
            levels: array::from_fn(|_| Queue::new()),
        }
    }

    fn push(&mut self, node_id: u32, level: u8) {
        self.levels[level as usize].push(node_id);
    }

    fn pop(&mut self) -> Option<u32> {
        for level in self.levels.iter_mut().rev() {
            if let Some(node_id) = level.pop() {
                return Some(node_id);
            }
        }
        None
    }
}

struct Octree {
    pool: Pool,
    root: u32,
    reducible: Reducible,
    color_count: usize,
    leaf_count: usize,
}

impl Octree {
    fn new(color_count: usize) -> Self {
        let mut pool = Pool::new();
        let root = pool.create();
        pool.get_mut(root).is_leaf = true;
        Self {
            pool,
            root,
            reducible: Reducible::new(),
            color_count,
            leaf_count: 1,
        }
    }

    pub fn traverse<F>(&self, f: F)
    where
        F: FnMut(u32, &Node),
    {
        let mut f = f;
        let mut queue = VecDeque::new();
        queue.push_back(self.root);
        while let Some(node_id) = queue.pop_front() {
            let node = self.pool.get(node_id);
            f(node_id, node);
            for child_id in node.children.iter().flatten() {
                queue.push_back(*child_id);
            }
        }
    }

    pub fn traverse_mut<F>(&mut self, f: F)
    where
        F: FnMut(u32, &mut Node),
    {
        let mut f = f;
        let mut queue = VecDeque::new();
        queue.push_back(self.root);
        while let Some(node_id) = queue.pop_front() {
            let node = self.pool.get_mut(node_id);
            f(node_id, node);
            for child_id in node.children.iter().flatten() {
                queue.push_back(*child_id);
            }
        }
    }

    fn insert(&mut self, color: Rgb<u8>) {
        let mut node_id = self.root;
        for level in 1..=MAX_LEVEL {
            let child_index = get_color_index(color, level);
            node_id = match self.pool.get(node_id).children[child_index] {
                Some(child_id) => child_id,
                None => {
                    let child_id = self.pool.create();
                    {
                        let child = self.pool.get_mut(child_id);
                        child.level = level;
                        child.parent = Some(node_id);
                        child.is_leaf = true;
                        self.leaf_count += 1;
                    }
                    {
                        let parent = self.pool.get_mut(node_id);
                        parent.children_count += 1;
                        parent.children[child_index] = Some(child_id);
                        if parent.is_leaf {
                            parent.is_leaf = false;
                            self.leaf_count -= 1;
                            self.reducible.push(node_id, parent.level);
                        }
                    }
                    child_id
                }
            }
        }
        self.pool.get_mut(node_id).merge_color(color);
        self.reduce();
    }

    fn get_index(&self, color: Rgb<u8>) -> usize {
        let mut node_id = self.root;
        for level in 1..=MAX_LEVEL {
            let node = self.pool.get(node_id);
            if node.is_leaf {
                break;
            }
            let child_index = get_color_index(color, level);
            node_id = match node.children[child_index] {
                Some(child_id) => child_id,
                None => {
                    match node
                        .children
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
        self.pool.get(node_id).index as usize
    }

    fn prune_node(&mut self, node_id: u32) {
        let children = {
            let node = self.pool.get_mut(node_id);
            node.is_leaf = true;
            self.leaf_count -= (node.children_count as usize) - 1;
            node.children_count = 0;
            std::mem::take(&mut node.children)
        };
        for child_id in children.into_iter().flatten() {
            let child = self.pool.delete(child_id);
            self.pool.get_mut(node_id).merge_node(child);
        }
    }

    fn reduce(&mut self) {
        if self.leaf_count > self.color_count {
            while let Some(node_id) = self.reducible.pop() {
                self.prune_node(node_id);
                if self.leaf_count <= self.color_count {
                    break;
                }
            }
        }
    }

    fn finalize(&mut self) -> Vec<Rgb<u8>> {
        println!("leaves: {}", self.leaf_count);
        let mut palette = Vec::new();
        self.traverse_mut(|_, node| {
            if node.is_leaf {
                node.index = palette.len() as u8;
                palette.push(Rgb::from(array::from_fn(|i| {
                    (node.rgb[i] / node.count) as u8
                })));
            }
        });
        palette
    }
}

pub struct ColorQuantizer {
    octree: Octree,
    colors: Vec<Rgb<u8>>,
}

impl ColorQuantizer {
    pub fn from(img: &RgbImage, palette_size: usize) -> Self {
        let palette_size = palette_size.min(MAX_COLORS);
        let mut octree = Octree::new(palette_size);
        for pixel in img.pixels() {
            octree.insert(*pixel);
        }
        let colors = octree.finalize();
        println!("final color palette size: {}", colors.len());
        Self { octree, colors }
    }

    #[inline(always)]
    pub fn get_palette(&self) -> &[Rgb<u8>] {
        &self.colors
    }

    #[inline(always)]
    pub fn get_index(&self, color: Rgb<u8>) -> usize {
        self.octree.get_index(color)
    }
}

impl ColorMap for ColorQuantizer {
    type Color = Rgb<u8>;

    #[inline(always)]
    fn index_of(&self, color: &Self::Color) -> usize {
        self.get_index(*color)
    }

    #[inline(always)]
    fn map_color(&self, color: &mut Self::Color) {
        *color = self.get_palette()[self.get_index(*color)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_octree() {
        let mut octree = Octree::new(3);
        octree.insert(Rgb::from([1, 2, 3]));
        octree.insert(Rgb::from([200, 2, 3]));
        octree.insert(Rgb::from([1, 200, 3]));
        assert_eq!(octree.leaf_count, 3);
        octree.insert(Rgb::from([1, 2, 200]));
        assert!(octree.leaf_count <= 3);
    }
}
