use crate::math::Vec3;

/// A cubemap with six black 1x1 faces. It emits no illumination.
pub struct Skybox {
    faces: [Vec3; 6],
}
impl Skybox {
    pub const fn black() -> Self {
        Self {
            faces: [Vec3::new(0.0, 0.0, 0.0); 6],
        }
    }
    pub fn sample(&self, d: Vec3) -> Vec3 {
        let a = [d.x.abs(), d.y.abs(), d.z.abs()];
        let axis = if a[0] >= a[1] && a[0] >= a[2] {
            0
        } else if a[1] >= a[2] {
            1
        } else {
            2
        };
        self.faces[axis * 2 + (d.component(axis) < 0.0) as usize]
    }
}
pub const SKYBOX: Skybox = Skybox::black();
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_faces_are_black() {
        for d in [
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        ] {
            assert_eq!(SKYBOX.sample(d), Vec3::default());
            assert_eq!(SKYBOX.sample(-d), Vec3::default());
        }
    }
}
