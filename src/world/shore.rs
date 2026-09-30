use super::{Cell, World};
use crate::material::Block;

// Small planted pockets on existing banks, leaving the water surface unobstructed.
pub fn build(world: &mut World) {
    for (cx, cz) in [
        (43, 37),
        (56, 34),
        (70, 43),
        (72, 57),
        (65, 67),
        (48, 73),
        (35, 64),
    ] {
        for (dx, dz) in [(-1, 0), (0, 0), (1, 0), (0, 1), (1, 1), (-2, -1), (2, -1)] {
            let (x, z) = (cx + dx, cz + dz);
            if let Some(y) = ground(world, x, z) {
                world.set(
                    Cell::new(x, y + 1, z),
                    if dx == 0 { Block::Leaves } else { Block::Fern },
                );
            }
        }
        if let Some(y) = ground(world, cx - 1, cz - 2) {
            world.set(Cell::new(cx - 1, y + 1, cz - 2), Block::Torch);
        }
    }
}
fn ground(world: &World, x: i32, z: i32) -> Option<i32> {
    (6..14).rev().find(|&y| {
        matches!(world.get(Cell::new(x, y, z)), Block::Moss | Block::Grass)
            && world.get(Cell::new(x, y + 1, z)) == Block::Air
    })
}
