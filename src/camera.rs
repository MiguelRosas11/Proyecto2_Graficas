use crate::math::Vec3;

pub const MAIN_TARGET: Vec3 = Vec3::new(51.0, 14.0, 53.0);
pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
    target: Vec3,
    azimuth: f32,
    elevation: f32,
    radius: f32,

    detail: bool,
    center_angle: f32,
}
impl Default for Camera {
    fn default() -> Self {
        Self::preset(1)
    }
}
impl Camera {
    pub fn preset(view: u8) -> Self {
        let mut c = Self {
            position: Vec3::default(),
            yaw: 0.0,
            pitch: 0.0,
            fov: 58.0f32.to_radians(),
            target: if view == 2 {
                Vec3::new(51.0, 15.0, 53.0)
            } else if view == 3 {
                Vec3::new(54.0, 11.0, 54.0)
            } else {
                MAIN_TARGET
            },
            azimuth: match view {
                2 => 330.0f32,

                3 => 210.0,
                _ => 325.0,
            }
            .to_radians(),
            elevation: if view == 3 { 0.12 } else { 0.24 },
            radius: if view == 2 || view == 3 { 18.0 } else { 29.0 },

            detail: view == 2 || view == 3,
            center_angle: 0.0,
        };
        c.center_angle = c.azimuth;
        c.update();
        c
    }
    fn update(&mut self) {
        self.position = self.target
            + Vec3::new(
                self.azimuth.sin() * self.elevation.cos(),
                self.elevation.sin(),
                -self.azimuth.cos() * self.elevation.cos(),
            ) * self.radius;
        let d = (self.target - self.position).normalized();
        self.yaw = d.x.atan2(d.z);
        self.pitch = d.y.asin();
    }
    pub fn orbit(&mut self, h: f32, v: f32) {
        if self.detail {
            self.azimuth =
                (self.azimuth + h).clamp(self.center_angle - 0.18, self.center_angle + 0.18);
            self.elevation = (self.elevation + v).clamp(0.10, 0.30);
        } else {
            self.azimuth = (self.azimuth + h).rem_euclid(std::f32::consts::TAU);
            self.elevation = (self.elevation + v).clamp(0.24, 0.50);
        }
        self.update();
    }
    pub fn set_angle(&mut self, degrees: f32) {
        self.azimuth = if self.detail {
            degrees
                .to_radians()
                .clamp(self.center_angle - 0.18, self.center_angle + 0.18)
        } else {
            degrees.to_radians().rem_euclid(std::f32::consts::TAU)
        };
        self.update();
    }
    pub fn zoom(&mut self, steps: f32) {
        self.radius = if self.detail {
            (self.radius - steps).clamp(16.0, 20.0)
        } else {
            (self.radius - steps * 1.5).clamp(26.0, 34.0)
        };
        self.update();
    }
    pub fn forward(&self) -> Vec3 {
        (self.target - self.position).normalized()
    }
    pub fn right(&self) -> Vec3 {
        Vec3::new(self.yaw.cos(), 0.0, -self.yaw.sin())
    }
    pub fn up(&self) -> Vec3 {
        Vec3::new(
            -self.yaw.sin() * self.pitch.sin(),
            self.pitch.cos(),
            -self.yaw.cos() * self.pitch.sin(),
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn main_orbit_stays_inside_perimeter() {
        let mut c = Camera::default();
        c.zoom(-1000.0);
        for _ in 0..16 {
            c.orbit(std::f32::consts::TAU / 16.0, 0.0);
            assert!((c.position - MAIN_TARGET).length() < 39.001);
            assert!(c.position.y > 17.0);
        }
    }
}
