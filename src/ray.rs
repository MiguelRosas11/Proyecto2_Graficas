use crate::{
    material::Block,
    math::Vec3,
    texture::Textures,
    world::{Cell, World},
};

#[derive(Clone, Copy, Debug)]
pub struct Hit {
    pub cell: Cell,
    pub block: Block,
    pub distance: f32,
    pub normal: Vec3,
    pub point: Vec3,
}

/// Amanatides-Woo traversal: one visit per voxel, without triangle meshes.
/// Slab clipping allows the camera to orbit outside the finite diorama.
#[cfg(test)]
pub fn cast(world: &World, origin: Vec3, direction: Vec3, max_distance: f32) -> Option<Hit> {
    traverse(world, origin, direction, max_distance, false, None)
}

/// Shadow rays skip water volumes, so transparent water does not cast a solid shadow.
#[cfg(test)]
pub fn cast_opaque(world: &World, origin: Vec3, direction: Vec3, max_distance: f32) -> Option<Hit> {
    traverse(world, origin, direction, max_distance, true, None)
}
pub fn cast_visible(
    world: &World,
    textures: &Textures,
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
    shadow: bool,
) -> Option<Hit> {
    traverse(
        world,
        origin,
        direction,
        max_distance,
        shadow,
        Some(textures),
    )
}

pub fn in_water(world: &World, p: Vec3) -> bool {
    world.get(Cell::new(
        p.x.floor() as i32,
        p.y.floor() as i32,
        p.z.floor() as i32,
    )) == Block::Water
}

fn traverse(
    world: &World,
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
    opaque_only: bool,
    textures: Option<&Textures>,
) -> Option<Hit> {
    traverse_impl(
        world,
        origin,
        direction,
        max_distance,
        opaque_only,
        textures,
        true,
    )
}
fn traverse_impl(
    world: &World,
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
    opaque_only: bool,
    textures: Option<&Textures>,
    skip_empty: bool,
) -> Option<Hit> {
    if direction.dot(direction) < 1e-12 {
        return None;
    }
    let bounds = [
        world.size.x as f32,
        world.size.y as f32,
        world.size.z as f32,
    ];
    let mut enter = 0.0f32;
    let mut exit = max_distance;
    let mut normal = Vec3::default();
    for (axis, &bound) in bounds.iter().enumerate() {
        let o = origin.component(axis);
        let d = direction.component(axis);
        if d.abs() < 1e-8 {
            if o < 0.0 || o >= bound {
                return None;
            }
            continue;
        }
        let a = -o / d;
        let b = (bound - o) / d;
        let near = a.min(b);
        let far = a.max(b);
        if near > enter {
            enter = near;
            normal = axis_normal(axis, -d.signum());
        }
        exit = exit.min(far);
        if enter > exit {
            return None;
        }
    }
    if exit < 0.0 {
        return None;
    }
    let p = origin + direction * (enter + 0.0001);
    let mut cell = [p.x.floor() as i32, p.y.floor() as i32, p.z.floor() as i32];
    let mut step = [0i32; 3];
    let mut delta = [f32::INFINITY; 3];
    let mut next = [f32::INFINITY; 3];
    for axis in 0..3 {
        let d = direction.component(axis);
        if d.abs() < 1e-8 {
            continue;
        }
        step[axis] = if d > 0.0 { 1 } else { -1 };
        delta[axis] = 1.0 / d.abs();
        let edge = cell[axis] as f32 + if d > 0.0 { 1.0 } else { 0.0 };
        next[axis] = (edge - origin.component(axis)) / d;
    }
    let mut distance = enter;
    let submerged = !opaque_only && in_water(world, origin);
    let mut previous = Cell::new(cell[0], cell[1], cell[2]);
    loop {
        let c = Cell::new(cell[0], cell[1], cell[2]);
        let block = world.get(c);
        if submerged && block == Block::Air {
            return Some(Hit {
                cell: previous,
                block: Block::Water,
                distance,
                normal: -normal,
                point: origin + direction * distance,
            });
        }
        if cell
            .iter()
            .enumerate()
            .any(|(a, &v)| v < 0 || v >= bounds[a] as i32)
        {
            return None;
        }
        if block != Block::Air && !(block == Block::Water && (submerged || opaque_only)) {
            // Camera inside a solid voxel: choose a stable face for preview shading.
            if normal == Vec3::default() {
                normal = axis_normal(1, 1.0);
            }
            let hit = Hit {
                cell: c,
                block,
                distance,
                normal,
                point: origin + direction * distance,
            };
            let visible = |h: &Hit| {
                if !block.cutout() {
                    return true;
                }
                let (u, v) = face_uv(h);
                textures.is_none_or(|tex| tex.opaque(block.texture(h.normal), u, v))
            };
            if block.plant() {
                let base = Vec3::new(c.x as f32, c.y as f32, c.z as f32);
                let mut candidates = [
                    box_intersection(
                        base + Vec3::new(0.48, 0.0, 0.0),
                        base + Vec3::new(0.52, 1.0, 1.0),
                        origin,
                        direction,
                    ),
                    box_intersection(
                        base + Vec3::new(0.0, 0.0, 0.48),
                        base + Vec3::new(1.0, 1.0, 0.52),
                        origin,
                        direction,
                    ),
                ];
                candidates.sort_by(|a, b| {
                    a.map_or(f32::INFINITY, |v| v.0)
                        .total_cmp(&b.map_or(f32::INFINITY, |v| v.0))
                });
                for (t, n) in candidates.into_iter().flatten() {
                    let h = Hit {
                        distance: t,
                        normal: n,
                        point: origin + direction * t,
                        ..hit
                    };
                    if t <= exit && visible(&h) {
                        return Some(h);
                    }
                }
            } else if block == Block::Torch {
                if let Some((t, n)) = thin_intersection(c, origin, direction, block)
                    && t <= exit
                {
                    return Some(Hit {
                        distance: t,
                        normal: n,
                        point: origin + direction * t,
                        ..hit
                    });
                }
            } else if visible(&hit) {
                return Some(hit);
            }
        }
        previous = c;
        // Empty 8^3 regions can be crossed without checking their individual cells.
        // Water exits have already been handled above, before this acceleration.
        if skip_empty && world.empty_region(c) {
            let mut crossing = [f32::INFINITY; 3];
            for a in 0..3 {
                if step[a] != 0 {
                    let edge = (cell[a] / 8 * 8 + if step[a] > 0 { 8 } else { 0 }) as f32;
                    crossing[a] = (edge - origin.component(a)) / direction.component(a);
                }
            }
            let axis = if crossing[0] <= crossing[1] && crossing[0] <= crossing[2] {
                0
            } else if crossing[1] <= crossing[2] {
                1
            } else {
                2
            };
            distance = crossing[axis];
            if distance > exit || !distance.is_finite() {
                return None;
            }
            normal = axis_normal(axis, -step[axis] as f32);
            let p = origin + direction * (distance + 0.0001);
            for a in 0..3 {
                cell[a] = if crossing[a] <= distance + 1e-6 {
                    cell[a] / 8 * 8 + if step[a] > 0 { 8 } else { -1 }
                } else {
                    // Rounding near a parallel grid plane must never move a ray backward.
                    let candidate = p.component(a).floor() as i32;
                    if step[a] < 0 {
                        candidate.min(cell[a])
                    } else {
                        candidate.max(cell[a])
                    }
                };
                if step[a] != 0 {
                    let edge = cell[a] as f32 + if step[a] > 0 { 1.0 } else { 0.0 };
                    next[a] = (edge - origin.component(a)) / direction.component(a);
                }
            }
            continue;
        }
        let axis = if next[0] <= next[1] && next[0] <= next[2] {
            0
        } else if next[1] <= next[2] {
            1
        } else {
            2
        };
        distance = next[axis];
        if distance > exit || !distance.is_finite() {
            return None;
        }
        normal = axis_normal(axis, -step[axis] as f32);
        // Simultaneous crossings avoid hitting zero-width voxels along shared edges.
        for a in 0..3 {
            if next[a] <= distance + 1e-6 {
                cell[a] += step[a];
                next[a] += delta[a];
            }
        }
    }
}

fn thin_intersection(
    cell: Cell,
    origin: Vec3,
    direction: Vec3,
    block: Block,
) -> Option<(f32, Vec3)> {
    let min = Vec3::new(cell.x as f32 + 0.40, cell.y as f32, cell.z as f32 + 0.40);
    let max = min + Vec3::new(0.20, if block == Block::Torch { 0.85 } else { 1.0 }, 0.20);
    box_intersection(min, max, origin, direction)
}
fn box_intersection(min: Vec3, max: Vec3, origin: Vec3, direction: Vec3) -> Option<(f32, Vec3)> {
    let mut near = f32::NEG_INFINITY;
    let mut far = f32::INFINITY;
    let mut normal = Vec3::default();
    let mut far_normal = normal;
    for a in 0..3 {
        let d = direction.component(a);
        let o = origin.component(a);
        if d.abs() < 1e-8 {
            if o < min.component(a) || o > max.component(a) {
                return None;
            }
            continue;
        }
        let t1 = (min.component(a) - o) / d;
        let t2 = (max.component(a) - o) / d;
        if t1.min(t2) > near {
            near = t1.min(t2);
            normal = axis_normal(a, -d.signum());
        }
        if t1.max(t2) < far {
            far = t1.max(t2);
            far_normal = axis_normal(a, d.signum());
        }
        if near > far {
            return None;
        }
    }
    if far < 0.0 {
        None
    } else if near < 0.0 {
        Some((far, far_normal))
    } else {
        Some((near, normal))
    }
}
fn axis_normal(axis: usize, sign: f32) -> Vec3 {
    match axis {
        0 => Vec3::new(sign, 0.0, 0.0),
        1 => Vec3::new(0.0, sign, 0.0),
        _ => Vec3::new(0.0, 0.0, sign),
    }
}

pub fn face_uv(hit: &Hit) -> (f32, f32) {
    if hit.block == Block::Torch {
        let local = hit.point - Vec3::new(hit.cell.x as f32, hit.cell.y as f32, hit.cell.z as f32);
        let u = if hit.normal.x.abs() > 0.5 {
            (local.z - 0.40) / 0.20
        } else {
            (local.x - 0.40) / 0.20
        };
        let v = if hit.normal.y.abs() > 0.5 {
            0.01
        } else {
            1.0 - local.y / 0.85
        };
        return (u.clamp(0.0, 0.999), v.clamp(0.0, 0.999));
    }
    let p = hit.point;
    let (u, v) = if hit.normal.y.abs() > 0.5 {
        (p.x, p.z)
    } else if hit.normal.x.abs() > 0.5 {
        (p.z, -p.y)
    } else {
        (p.x, -p.y)
    };
    (u.rem_euclid(1.0), v.rem_euclid(1.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn grazing_empty_region_boundary_does_not_step_backward() {
        let w = crate::world::generation::generate(&Default::default());
        let origin = Vec3::new(80.03938, 22.319592, 31.960613);
        let direction = Vec3::new(-0.005001619, 0.21340251, 0.97695154);
        let a = traverse_impl(&w, origin, direction, 180.0, false, None, true);
        let b = traverse_impl(&w, origin, direction, 180.0, false, None, false);
        assert_eq!(a.map(|h| h.cell), b.map(|h| h.cell));
        assert!(a.is_none());
    }
    #[test]
    fn empty_region_skips_match_cell_by_cell_traversal() {
        let w = crate::world::generation::generate(&Default::default());
        let mut seed = 7u32;
        let mut rand = || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 8) as f32 / 16777216.0
        };
        for _ in 0..5000 {
            let origin = Vec3::new(
                rand() * 130.0 - 9.0,
                rand() * 75.0 - 5.0,
                rand() * 130.0 - 9.0,
            );
            let direction = Vec3::new(rand() - 0.5, rand() - 0.5, rand() - 0.5).normalized();
            for shadow in [false, true] {
                let a = traverse_impl(&w, origin, direction, 180.0, shadow, None, true);
                let b = traverse_impl(&w, origin, direction, 180.0, shadow, None, false);
                assert_eq!(
                    a.map(|h| (h.cell, h.block)),
                    b.map(|h| (h.cell, h.block)),
                    "Ray {origin:?} {direction:?}"
                );
                if let (Some(a), Some(b)) = (a, b) {
                    assert!((a.distance - b.distance).abs() < 0.002);
                    assert_eq!(a.normal, b.normal);
                }
            }
        }
    }
    #[test]
    fn water_crosses_internal_voxels_and_exits_with_outward_normal() {
        let mut w = World::new(Cell::new(6, 6, 6));
        for z in 1..4 {
            w.set(Cell::new(2, 2, z), Block::Water);
        }
        let h = cast(&w, Vec3::new(2.5, 2.5, 1.5), Vec3::new(0.0, 0.0, 1.0), 10.0).unwrap();
        assert_eq!(h.block, Block::Water);
        assert!((h.distance - 2.5).abs() < 1e-5);
        assert_eq!(h.normal, Vec3::new(0.0, 0.0, 1.0));
        assert!(
            cast_opaque(&w, Vec3::new(2.5, 2.5, 0.0), Vec3::new(0.0, 0.0, 1.0), 10.0).is_none()
        );
        w.set(Cell::new(2, 2, 4), Block::Stone);
        let floor = cast(&w, Vec3::new(2.5, 2.5, 1.5), Vec3::new(0.0, 0.0, 1.0), 10.0).unwrap();
        assert_eq!(floor.block, Block::Stone);
    }

    #[test]
    fn torch_occupies_only_its_narrow_shape() {
        let mut w = World::new(Cell::new(4, 4, 4));
        w.set(Cell::new(1, 1, 1), Block::Torch);
        let dir = Vec3::new(0.0, 0.0, 1.0);
        assert!(cast(&w, Vec3::new(1.1, 1.5, 0.0), dir, 10.0).is_none());
        let hit = cast(&w, Vec3::new(1.5, 1.5, 0.0), dir, 10.0).unwrap();
        assert!((hit.distance - 1.4).abs() < 1e-5);
    }
    fn scene() -> World {
        let mut w = World::new(Cell::new(4, 4, 4));
        w.set(Cell::new(1, 1, 1), Block::Stone);
        w
    }
    #[test]
    fn enters_from_outside_and_returns_face() {
        let h = cast(
            &scene(),
            Vec3::new(1.5, 1.5, -4.0),
            Vec3::new(0.0, 0.0, 1.0),
            20.0,
        )
        .unwrap();
        assert_eq!(h.cell, Cell::new(1, 1, 1));
        assert!((h.distance - 5.0).abs() < 1e-5);
        assert_eq!(h.normal, Vec3::new(0.0, 0.0, -1.0));
    }
    #[test]
    fn distance_and_parallel_misses() {
        assert!(
            cast(
                &scene(),
                Vec3::new(1.5, 1.5, -4.0),
                Vec3::new(0.0, 0.0, 1.0),
                4.9
            )
            .is_none()
        );
        assert!(
            cast(
                &scene(),
                Vec3::new(-1.0, 1.5, -4.0),
                Vec3::new(0.0, 0.0, 1.0),
                20.0
            )
            .is_none()
        );
    }
    #[test]
    fn negative_direction_and_removed_block() {
        let mut w = scene();
        let o = Vec3::new(4.0, 1.5, 1.5);
        let d = Vec3::new(-1.0, 0.0, 0.0);
        let hit = cast(&w, o, d, 10.0).unwrap();
        assert_eq!(hit.normal, Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(w.remove(hit.cell), Some(Block::Stone));
        assert!(cast(&w, o, d, 10.0).is_none());
    }
    #[test]
    fn boundary_ray_and_zero_direction_are_safe() {
        assert!(
            cast(
                &scene(),
                Vec3::new(4.0, 2.0, 2.0),
                Vec3::new(1.0, 0.0, 0.0),
                10.0
            )
            .is_none()
        );
        assert!(cast(&scene(), Vec3::default(), Vec3::default(), 10.0).is_none());
    }
}
