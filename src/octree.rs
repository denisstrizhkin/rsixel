use image::{imageops::ColorMap, Rgb, RgbImage};
use priority_queue::PriorityQueue;
use std::{array, collections::VecDeque, iter};

const MAX_LEVEL: u8 = 8;

#[derive(Debug, Default)]
struct Node {
    rgb: [u32; 3],
    count: u32,
    index: usize,
    parent: Option<u32>,
    children: [Option<u32>; 8],
}

impl Node {
    fn is_leaf(&self) -> bool {
        !self.children.iter().any(Option::is_some)
    }

    fn merge_color(&mut self, color: Rgb<u8>) {
        self.count += 1;
        iter::zip(&mut self.rgb, color.0).for_each(|(a, b)| *a += b as u32)
    }

    fn merge_node(&mut self, node: Node) {
        self.count += node.count;
        iter::zip(&mut self.rgb, node.rgb).for_each(|(a, b)| *a += b)
    }
}

#[derive(Debug, Default)]
struct Pool {
    nodes: Vec<Node>,
}

impl Pool {
    fn create(&mut self) -> u32 {
        let id = self.nodes.len();
        self.nodes.push(Node::default());
        id as u32
    }

    fn get(&self, id: u32) -> &Node {
        &self.nodes[id as usize]
    }

    fn get_mut(&mut self, id: u32) -> &mut Node {
        &mut self.nodes[id as usize]
    }

    fn delete(&mut self, id: u32) -> Node {
        std::mem::take(&mut self.nodes[id as usize])
    }
}

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
    pool: Pool,
    root: u32,
}

impl Octree {
    fn new() -> Self {
        let mut pool = Pool::default();
        let root = pool.create();
        Self { pool, root }
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
                    self.pool.get_mut(child_id).parent = Some(node_id);
                    self.pool.get_mut(node_id).children[child_index] = Some(child_id);
                    child_id
                }
            }
        }
        self.pool.get_mut(node_id).merge_color(color);
    }

    fn get_index(&self, color: Rgb<u8>) -> usize {
        let mut node_id = self.root;
        for level in 1..=MAX_LEVEL {
            let node = self.pool.get(node_id);
            if node.is_leaf() {
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
        self.pool.get(node_id).index
    }

    fn prune_node(&mut self, node_id: u32) -> usize {
        std::mem::take(&mut self.pool.get_mut(node_id).children)
            .into_iter()
            .flatten()
            .map(|child_id| {
                let child = self.pool.delete(child_id);
                self.pool.get_mut(node_id).merge_node(child);
            })
            .count()
    }

    fn reduce_to(&mut self, color_count: usize) {
        let mut queue = PriorityQueue::new();
        let mut color_count_current = 0;
        self.traverse(|_, node| {
            if node.is_leaf() {
                color_count_current += 1;
            }
        });
        if color_count_current > color_count {
            'main: loop {
                self.traverse(|_, node| {
                    if node.is_leaf() {
                        let parent_id = node.parent.unwrap();
                        if !queue.contains(&parent_id) {
                            let count = self
                                .pool
                                .get(parent_id)
                                .children
                                .iter()
                                .flatten()
                                .map(|&child_id| self.pool.get(child_id).count)
                                .sum::<u32>();
                            queue.push(parent_id, count);
                        }
                    }
                });
                while let Some((node_id, _)) = queue.pop() {
                    color_count_current -= self.prune_node(node_id) - 1;
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
        self.traverse_mut(|_, node| {
            if node.is_leaf() {
                node.index = palette.len();
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
