use super::{Cell, World, garden, rockwork, shore};
pub struct GenerationConfig {
    pub seed: u32,
}
impl Default for GenerationConfig {
    fn default() -> Self {
        Self { seed: 42 }
    }
}
pub fn generate(config: &GenerationConfig) -> World {
    let mut world = World::new(Cell::new(112, 64, 112));
    rockwork::cavern(&mut world, config.seed);
    garden::build(&mut world, config.seed);
    shore::build(&mut world);

    world
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{camera::Camera, material::Block};
    #[test]
    fn orbit_is_clear_at_both_zoom_limits() {
        let w = generate(&GenerationConfig::default());
        for angle in (0..360).step_by(10) {
            for elevation in [-100.0, 0.0, 100.0] {
                for zoom in [-100.0, 100.0] {
                    let mut c = Camera::default();
                    c.set_angle(angle as f32);
                    c.zoom(zoom);
                    c.orbit(0.0, elevation);
                    let p = c.position;
                    assert_eq!(
                        w.get(Cell::new(
                            p.x.floor() as i32,
                            p.y.floor() as i32,
                            p.z.floor() as i32
                        )),
                        Block::Air,
                        "Camera at {p:?}"
                    );
                }
            }
        }
    }
    #[test]
    fn sparse_scene_stays_within_block_budget() {
        let w = generate(&GenerationConfig::default());
        assert!(w.solid_count() < 6_500);
        for z in 0..112 {
            for x in 0..112 {
                assert_eq!(w.get(Cell::new(x, 0, z)), Block::Air);
            }
        }
    }
    #[test]
    fn distant_background_is_empty() {
        let w = generate(&GenerationConfig::default());
        for y in 0..w.size.y {
            for z in 0..w.size.z {
                for x in 0..w.size.x {
                    if !(29..83).contains(&x) || !(30..81).contains(&z) {
                        assert_eq!(w.get(Cell::new(x, y, z)), Block::Air);
                    }
                }
            }
        }
    }
    #[test]
    fn island_has_water_level_rim_and_no_stepping_stones() {
        let w = generate(&GenerationConfig::default());
        assert_eq!(w.get(Cell::new(51, 9, 57)), Block::Moss);
        assert_eq!(w.get(Cell::new(51, 10, 57)), Block::Air);
        assert_eq!(w.get(Cell::new(51, 10, 53)), Block::Moss);
        assert_eq!(w.get(Cell::new(51, 11, 53)), Block::Wood);
        for (x, z) in [(48, 47), (46, 44)] {
            assert_eq!(w.get(Cell::new(x, 9, z)), Block::Water);
            assert_eq!(w.get(Cell::new(x, 10, z)), Block::Air);
        }
    }
    #[test]
    fn detail_views_stay_in_air() {
        let w = generate(&GenerationConfig::default());
        for view in 2..=3 {
            for h in [-100.0, 0.0, 100.0] {
                for v in [-100.0, 0.0, 100.0] {
                    for zoom in [-100.0, 100.0] {
                        let mut c = Camera::preset(view);
                        c.orbit(h, v);
                        c.zoom(zoom);
                        let p = c.position;
                        assert_eq!(
                            w.get(Cell::new(
                                p.x.floor() as i32,
                                p.y.floor() as i32,
                                p.z.floor() as i32
                            )),
                            Block::Air,
                            "View {view} at {p:?}"
                        );
                    }
                }
            }
        }
    }
}
