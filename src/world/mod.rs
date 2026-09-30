mod garden;
pub mod generation;
mod shore;

mod rockwork;
use crate::material::Block;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}
impl Cell {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

pub struct World {
    pub size: Cell,
    blocks: Vec<Block>,
    regions: Vec<u16>,
    region_size: Cell,
}
impl World {
    pub fn new(size: Cell) -> Self {
        assert!(size.x > 0 && size.y > 0 && size.z > 0);
        let region_size = Cell::new((size.x + 7) / 8, (size.y + 7) / 8, (size.z + 7) / 8);
        Self {
            size,
            blocks: vec![Block::Air; (size.x * size.y * size.z) as usize],
            regions: vec![0; (region_size.x * region_size.y * region_size.z) as usize],
            region_size,
        }
    }
    fn index(&self, p: Cell) -> Option<usize> {
        if p.x < 0
            || p.y < 0
            || p.z < 0
            || p.x >= self.size.x
            || p.y >= self.size.y
            || p.z >= self.size.z
        {
            None
        } else {
            Some(((p.y * self.size.z + p.z) * self.size.x + p.x) as usize)
        }
    }
    pub fn get(&self, p: Cell) -> Block {
        self.index(p).map_or(Block::Air, |i| self.blocks[i])
    }
    pub fn set(&mut self, p: Cell, block: Block) -> bool {
        if let Some(i) = self.index(p) {
            let r = self.region_index(p);
            self.regions[r] = (self.regions[r] as i32 + (block != Block::Air) as i32
                - (self.blocks[i] != Block::Air) as i32) as u16;
            self.blocks[i] = block;
            true
        } else {
            false
        }
    }
    fn region_index(&self, p: Cell) -> usize {
        (((p.y / 8) * self.region_size.z + p.z / 8) * self.region_size.x + p.x / 8) as usize
    }
    pub fn empty_region(&self, p: Cell) -> bool {
        self.regions[self.region_index(p)] == 0
    }
    #[cfg(test)]
    pub fn remove(&mut self, p: Cell) -> Option<Block> {
        let old = self.get(p);
        if old == Block::Air {
            return None;
        }
        self.set(p, Block::Air);
        Some(old)
    }
    pub fn solid_count(&self) -> usize {
        self.blocks.iter().filter(|&&b| b != Block::Air).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn occupancy_tracks_replacement_removal_and_partial_regions() {
        let mut world = World::new(Cell::new(17, 9, 10));
        let a = Cell::new(16, 8, 9);
        let b = Cell::new(16, 8, 8);
        assert!(world.empty_region(a));
        world.set(a, Block::Stone);
        world.set(a, Block::Water);
        world.set(b, Block::Wood);
        world.remove(a);
        assert!(!world.empty_region(a));
        world.remove(b);
        assert!(world.empty_region(a));
        assert!(world.empty_region(Cell::new(0, 0, 0)));
    }
}
