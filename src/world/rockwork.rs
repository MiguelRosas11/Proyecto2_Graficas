use super::{Cell, World};
use crate::material::Block;

pub fn hash(x: i32, y: i32, z: i32, seed: u32) -> u32 {
    let n = (x as u32).wrapping_mul(374761393)
        ^ (z as u32).wrapping_mul(668265263)
        ^ (y as u32).wrapping_mul(1274126177)
        ^ seed;
    let n = (n ^ (n >> 13)).wrapping_mul(1274126177);
    n ^ (n >> 16)
}
fn wave(x: f32, z: f32) -> f32 {
    (x * 0.16 + z * 0.06).sin() * 1.3 + (z * 0.21 - x * 0.05).cos() * 0.9
}

pub fn cavern(world: &mut World, seed: u32) {
    // Only the lit foreground exists: two-cell soil shell, water and its bed.
    // There is no enclosing wall, distant floor, roof, or underground solid fill.
    for z in 30..81 {
        for x in 29..83 {
            let dx = x as f32 - 53.0;
            let dz = z as f32 - 54.0;
            let r = (dx * dx + dz * dz).sqrt();
            let edge = 23.0 + 2.0 * (dz * 0.16).sin() + 1.5 * (dx * 0.25).cos();
            if r > edge {
                continue;
            }
            let ground = (8.5 + wave(x as f32, z as f32)).round() as i32;
            for y in ground - 1..=ground {
                world.set(
                    Cell::new(x, y, z),
                    if y == ground {
                        if (x + z) % 13 == 0 {
                            Block::Grass
                        } else {
                            Block::Moss
                        }
                    } else {
                        Block::Dirt
                    },
                );
            }
            let px = x as f32 - 53.0;
            let pz = z as f32 - 53.0;
            let pond = (px * px / 1.3 + pz * pz).sqrt();
            let edge = 15.0 + 1.7 * (pz * 0.19).sin() + 1.3 * (px * 0.25).cos();
            let island = ((x as f32 - 51.0).powi(2) + (z as f32 - 53.0).powi(2)).sqrt();
            if pond < edge && island > 4.4 + (pz * 0.8).sin() * 0.6 {
                let bed = if pond < edge - 3.0 { 5 } else { 7 };
                for y in 4..=12 {
                    world.set(
                        Cell::new(x, y, z),
                        if y == bed {
                            if (x + z) % 7 == 0 {
                                Block::Sand
                            } else {
                                Block::Clay
                            }
                        } else if y > bed && y <= 9 {
                            Block::Water
                        } else {
                            Block::Air
                        },
                    );
                }
            } else if island < 4.9 {
                let top = if island < 3.4 { 10 } else { 9 };
                for y in 7..=12 {
                    world.set(
                        Cell::new(x, y, z),
                        if y > top {
                            Block::Air
                        } else if y == top {
                            Block::Moss
                        } else {
                            Block::Dirt
                        },
                    );
                }
            }
        }
    }
    for (x, z, rx, rz, height) in [(37, 55, 4, 5, 14), (63, 39, 5, 4, 16), (60, 70, 4, 4, 13)] {
        boulder(world, x, z, rx, rz, height);
    }
    scatter_ores(world, seed);
}
fn boulder(world: &mut World, cx: i32, cz: i32, rx: i32, rz: i32, height: i32) {
    for y in 8..height {
        for z in cz - rz..=cz + rz {
            for x in cx - rx..=cx + rx {
                let d =
                    ((x - cx) as f32 / rx as f32).powi(2) + ((z - cz) as f32 / rz as f32).powi(2);
                let taper = 1.0 - (y - 8) as f32 / (height - 8) as f32 * 0.45;
                // Shell only: hidden rock cores are unnecessary for a fixed exterior view.
                if d < taper + 0.07 * (x as f32 + y as f32 * 0.6).sin()
                    && (d > taper - 0.48 || y >= height - 2)
                {
                    world.set(
                        Cell::new(x, y, z),
                        if y < 12 {
                            Block::Deepslate
                        } else {
                            Block::Stone
                        },
                    );
                }
            }
        }
    }
    for z in cz - rz..=cz + rz {
        for x in cx - rx..=cx + rx {
            for y in (10..height).rev() {
                let p = Cell::new(x, y, z);
                if world.get(p) != Block::Air {
                    world.set(p, Block::Moss);
                    break;
                }
            }
        }
    }
}
fn scatter_ores(world: &mut World, seed: u32) {
    let ores = [
        Block::Diamond,
        Block::Gold,
        Block::Emerald,
        Block::Redstone,
        Block::Lapis,
        Block::Iron,
    ];
    for y in 10..23 {
        for z in 30..80 {
            for x in 29..82 {
                let p = Cell::new(x, y, z);
                if !matches!(world.get(p), Block::Stone | Block::Deepslate) {
                    continue;
                }
                let h = hash(x, y, z, seed);
                if !(h as usize).is_multiple_of(31) {
                    continue;
                }
                if ![(1, 0, 0), (-1, 0, 0), (0, 0, 1), (0, 0, -1)]
                    .iter()
                    .any(|&(dx, dy, dz)| world.get(Cell::new(x + dx, y + dy, z + dz)) == Block::Air)
                {
                    continue;
                }
                let mut near = false;
                for dz in -2..=2 {
                    for dx in -2..=2 {
                        for dy in -2..=2 {
                            if world
                                .get(Cell::new(x + dx, y + dy, z + dz))
                                .mineral_color()
                                .is_some()
                            {
                                near = true;
                            }
                        }
                    }
                }
                if !near {
                    world.set(p, ores[((h >> 10) as usize) % 6]);
                }
            }
        }
    }
}
