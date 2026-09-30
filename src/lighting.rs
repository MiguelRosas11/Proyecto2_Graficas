use crate::{
    material::Block,
    math::Vec3,
    ray,
    texture::Textures,
    world::{Cell, World},
};
use std::sync::OnceLock;

pub struct PointLight {
    pub position: Vec3,
    pub color: Vec3,
    pub power: f32,
    pub range: f32,
    pub radius: f32,
    pub cell: Cell,
}
pub struct Lighting {
    pub shadows: bool,
    pub lights: Vec<PointLight>,
    bins: Vec<Vec<usize>>,
    dimensions: [usize; 3],
    face_ids: Vec<u32>,
    diffuse_cache: Vec<OnceLock<Vec3>>,
}
#[derive(Default)]
pub struct SurfaceLight {
    pub diffuse: Vec3,
    pub specular: Vec3,
}
pub struct Surface {
    pub diffuse: bool,
    pub point: Vec3,
    pub normal: Vec3,
    pub view: Vec3,
    pub shininess: f32,
    pub specular: f32,
}
impl Lighting {
    pub fn from_world(world: &World) -> Self {
        let mut lights = Vec::new();
        for y in 0..world.size.y {
            for z in 0..world.size.z {
                for x in 0..world.size.x {
                    let cell = Cell::new(x, y, z);
                    let block = world.get(cell);
                    let center = Vec3::new(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5);
                    let (position, color, power, range, radius) = match block {
                        Block::Torch => (
                            center + Vec3::new(0.0, 0.45, 0.0),
                            Vec3::new(1.0, 0.55, 0.20),
                            15.0,
                            13.0,
                            0.18,
                        ),
                        Block::Berries if world.get(Cell::new(x, y + 2, z)) == Block::Berries => {
                            continue;
                        }
                        Block::Berries => (center, Vec3::new(1.0, 0.79, 0.46), 18.0, 13.0, 0.28),
                        Block::Lava
                            if x % 2 == 1
                                && z % 2 == 0
                                && world.get(Cell::new(x, y + 1, z)) == Block::Air =>
                        {
                            (
                                center + Vec3::new(0.0, 0.7, 0.0),
                                Vec3::new(1.0, 0.32, 0.055),
                                45.0,
                                15.0,
                                0.55,
                            )
                        }
                        b if b.mineral_color().is_some() => {
                            let mut outward = None;
                            for (dx, dy, dz) in [
                                (0, 0, -1),
                                (1, 0, 0),
                                (0, 0, 1),
                                (-1, 0, 0),
                                (0, 1, 0),
                                (0, -1, 0),
                            ] {
                                if world.get(Cell::new(x + dx, y + dy, z + dz)) == Block::Air {
                                    outward = Some(Vec3::new(dx as f32, dy as f32, dz as f32));
                                    break;
                                }
                            }
                            let Some(outward) = outward else {
                                continue;
                            };
                            (
                                center + outward * 0.65,
                                b.mineral_color().unwrap(),
                                8.0,
                                9.0,
                                0.15,
                            )
                        }
                        _ => continue,
                    };
                    lights.push(PointLight {
                        position,
                        color,
                        power,
                        range,
                        radius,
                        cell,
                    });
                }
            }
        }
        let dimensions = [
            (world.size.x as usize).div_ceil(8),
            (world.size.y as usize).div_ceil(8),
            (world.size.z as usize).div_ceil(8),
        ];
        let mut bins = vec![Vec::new(); dimensions.iter().product()];
        for (index, light) in lights.iter().enumerate() {
            let p = light.position;
            let r = light.range;
            let low = [
                ((p.x - r) / 8.0).floor() as i32,
                ((p.y - r) / 8.0).floor() as i32,
                ((p.z - r) / 8.0).floor() as i32,
            ];
            let high = [
                ((p.x + r) / 8.0).floor() as i32,
                ((p.y + r) / 8.0).floor() as i32,
                ((p.z + r) / 8.0).floor() as i32,
            ];
            for z in low[2].max(0)..=high[2].min(dimensions[2] as i32 - 1) {
                for y in low[1].max(0)..=high[1].min(dimensions[1] as i32 - 1) {
                    for x in low[0].max(0)..=high[0].min(dimensions[0] as i32 - 1) {
                        bins[(z as usize * dimensions[1] + y as usize) * dimensions[0]
                            + x as usize]
                            .push(index);
                    }
                }
            }
        }
        let mut face_ids = vec![u32::MAX; (world.size.x * world.size.y * world.size.z) as usize];
        let mut count = 0;
        for y in 0..world.size.y {
            for z in 0..world.size.z {
                for x in 0..world.size.x {
                    let b = world.get(Cell::new(x, y, z));
                    if b != Block::Air && b != Block::Water && !b.plant() && b != Block::Torch {
                        face_ids[((y * world.size.z + z) * world.size.x + x) as usize] = count;
                        count += 1;
                    }
                }
            }
        }
        Self {
            shadows: true,
            lights,
            bins,
            dimensions,
            face_ids,
            diffuse_cache: (0..count as usize * 6 * 16)
                .map(|_| OnceLock::new())
                .collect(),
        }
    }

    /// Lazy 4x4 lightmaps on occupied voxel faces. Geometry and emitters are static;
    /// view-dependent reflections still trace rays every frame. No ambient fill.
    pub fn diffuse(&self, world: &World, textures: &Textures, hit: &ray::Hit) -> Vec3 {
        let c = hit.cell;
        let id = self.face_ids[((c.y * world.size.z + c.z) * world.size.x + c.x) as usize];
        if id == u32::MAX {
            return self
                .evaluate(
                    world,
                    textures,
                    &Surface {
                        diffuse: true,
                        point: hit.point,
                        normal: hit.normal,
                        view: -hit.normal,
                        shininess: 1.0,
                        specular: 0.0,
                    },
                    0.0,
                )
                .diffuse;
        }
        let n = hit.normal;
        let axis = if n.x.abs() > 0.5 {
            0
        } else if n.y.abs() > 0.5 {
            1
        } else {
            2
        };
        let face = axis * 2 + (n.component(axis) < 0.0) as usize;
        let base = Vec3::new(c.x as f32, c.y as f32, c.z as f32);
        let p = hit.point - base;
        let (u, v) = match axis {
            0 => (p.z, p.y),
            1 => (p.x, p.z),
            _ => (p.x, p.y),
        };
        let u = u.clamp(0.0, 0.9999) * 3.0;
        let v = v.clamp(0.0, 0.9999) * 3.0;
        let ix = u.floor() as usize;
        let iy = v.floor() as usize;
        let sample = |x: usize, y: usize| {
            *self.diffuse_cache[(id as usize * 6 + face) * 16 + y * 4 + x].get_or_init(|| {
                let a = (x as f32 / 3.0).clamp(0.005, 0.995);
                let b = (y as f32 / 3.0).clamp(0.005, 0.995);
                let surface = if n.component(axis) > 0.0 { 1.0 } else { 0.0 };
                let local = match axis {
                    0 => Vec3::new(surface, b, a),
                    1 => Vec3::new(a, surface, b),
                    _ => Vec3::new(a, b, surface),
                };
                self.evaluate(
                    world,
                    textures,
                    &Surface {
                        diffuse: true,
                        point: base + local,
                        normal: n,
                        view: -n,
                        shininess: 1.0,
                        specular: 0.0,
                    },
                    0.0,
                )
                .diffuse
            })
        };
        sample(ix, iy).lerp(sample(ix + 1, iy), u.fract()).lerp(
            sample(ix, iy + 1).lerp(sample(ix + 1, iy + 1), u.fract()),
            v.fract(),
        )
    }
    pub fn evaluate(
        &self,
        world: &World,
        textures: &Textures,
        s: &Surface,
        time: f32,
    ) -> SurfaceLight {
        let mut result = SurfaceLight::default();
        let p = s.point + s.normal * 0.004;
        let coords = [
            (p.x.max(0.0) / 8.0) as usize,
            (p.y.max(0.0) / 8.0) as usize,
            (p.z.max(0.0) / 8.0) as usize,
        ];
        if coords.iter().zip(self.dimensions).any(|(&v, d)| v >= d) {
            return result;
        }
        let index = (coords[2] * self.dimensions[1] + coords[1]) * self.dimensions[0] + coords[0];
        for &id in &self.bins[index] {
            let source = &self.lights[id];

            let delta = source.position - p;
            let d2 = delta.dot(delta);
            if d2 > source.range * source.range {
                continue;
            }
            let distance = d2.sqrt();
            let dir = delta / distance.max(0.001);
            let lambert = if s.diffuse {
                s.normal.dot(dir).max(0.0)
            } else {
                0.0
            };
            let half = (dir - s.view).normalized();
            let highlight = s.normal.dot(half).max(0.0).powf(s.shininess) * s.specular;
            if lambert + highlight < 0.001 {
                continue;
            }
            let mut visibility = 0.0;
            for offset in [
                Vec3::new(-0.6, 0.4, 0.0),
                Vec3::new(0.6, 0.4, 0.0),
                Vec3::new(0.0, -0.4, 0.6),
            ] {
                let vector = source.position + offset * source.radius - p;
                let dist = vector.length();
                let hit = if self.shadows {
                    ray::cast_visible(world, textures, p, vector / dist, dist - 0.02, true)
                } else {
                    None
                };
                if hit.is_none_or(|h| h.cell == source.cell) {
                    visibility += 1.0 / 3.0;
                }
            }
            if visibility == 0.0 {
                continue;
            }
            let fade = (1.0 - d2 / (source.range * source.range)).max(0.0);
            let flicker = if matches!(world.get(source.cell), Block::Torch | Block::Lava) {
                0.98 + 0.02 * (time * 4.0 + id as f32).sin()
            } else {
                1.0
            };
            let energy = source.power * fade * visibility * flicker / (1.0 + d2);
            result.diffuse += source.color * (energy * lambert);
            result.specular += source.color * (energy * highlight);
        }
        result
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_world_has_no_light() {
        let w = World::new(Cell::new(8, 8, 8));
        assert!(Lighting::from_world(&w).lights.is_empty());
    }
    #[test]
    fn only_emissive_blocks_register_lights() {
        let mut w = World::new(Cell::new(8, 8, 8));
        w.set(Cell::new(2, 2, 2), Block::Stone);
        assert!(Lighting::from_world(&w).lights.is_empty());
        w.set(Cell::new(3, 3, 3), Block::Torch);
        assert_eq!(Lighting::from_world(&w).lights.len(), 1);
    }
}
