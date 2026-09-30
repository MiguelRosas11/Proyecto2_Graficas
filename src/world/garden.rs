use super::{Cell, World, rockwork::hash};
use crate::material::Block;

pub fn build(world: &mut World, seed: u32) {
    // Vanilla-like compact azalea: bent oak trunk and an irregular low canopy.
    for y in 11..17 {
        world.set(Cell::new(51 + (y >= 15) as i32, y, 53), Block::Wood);
    }
    world.set(Cell::new(50, 15, 53), Block::Wood);
    world.set(Cell::new(53, 16, 53), Block::Wood);
    for (cx, cy, cz, r) in [
        (51i32, 17i32, 53i32, 3i32),
        (53, 17, 54, 3),
        (49, 16, 53, 2),
        (51, 18, 51, 2),
    ] {
        for y in cy - 1..=cy + 1 {
            for z in cz - r..=cz + r {
                for x in cx - r..=cx + r {
                    if (x - cx).abs() + (z - cz).abs() + 2 * (y - cy).abs() > r + 2 {
                        continue;
                    }
                    let p = Cell::new(x, y, z);
                    if world.get(p) == Block::Air {
                        world.set(
                            p,
                            if hash(x, y, z, seed).is_multiple_of(4) {
                                Block::Leaves
                            } else {
                                Block::Azalea
                            },
                        );
                    }
                }
            }
        }
    }
    // Hang from existing canopy edges and moss shelves; no floating supports.
    for (x, z, length) in [
        (47, 53, 5),
        (49, 50, 5),
        (51, 49, 4),
        (54, 51, 5),
        (56, 54, 6),
        (54, 57, 4),
        (51, 56, 5),
        (49, 55, 4),
        (38, 54, 4),
        (40, 56, 4),
        (63, 39, 4),
        (65, 40, 4),
        (60, 70, 3),
    ] {
        hang_vine(world, x, z, length);
    }
    // Torches sit on the low island, without raised pedestals.
    for (x, z) in [(49, 52), (53, 55)] {
        world.set(Cell::new(x, 11, z), Block::Torch);
    }
    // Ferns and little azalea bushes clustered on moist banks, not in a grid.
    for z in 28..82 {
        for x in 28..83 {
            let h = hash(x, 0, z, seed);
            if !h.is_multiple_of(11) {
                continue;
            }
            for y in (6..15).rev() {
                if matches!(world.get(Cell::new(x, y, z)), Block::Moss | Block::Grass)
                    && world.get(Cell::new(x, y + 1, z)) == Block::Air
                {
                    world.set(
                        Cell::new(x, y + 1, z),
                        if h.is_multiple_of(5) {
                            Block::Azalea
                        } else {
                            Block::Fern
                        },
                    );
                    break;
                }
            }
        }
    }
    // Amethyst is a tiny accent inside the garden, not a wall-sized patch.
    world.set(Cell::new(48, 10, 54), Block::Amethyst);
}

fn hang_vine(world: &mut World, x: i32, z: i32, length: i32) {
    // Find the lowest exposed underside, so vines hang rather than sit on leaves.
    for y in 12..=21 {
        if matches!(
            world.get(Cell::new(x, y, z)),
            Block::Leaves | Block::Azalea | Block::Moss
        ) && world.get(Cell::new(x, y - 1, z)) == Block::Air
        {
            for n in 1..=length {
                let p = Cell::new(x, y - n, z);
                if world.get(p) != Block::Air {
                    break;
                }
                world.set(
                    p,
                    if n % 2 == 1 {
                        Block::Berries
                    } else {
                        Block::Vine
                    },
                );
            }
            break;
        }
    }
}
