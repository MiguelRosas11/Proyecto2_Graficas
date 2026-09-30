use crate::math::Vec3;

pub fn reflect(direction: Vec3, normal: Vec3) -> Vec3 {
    direction - normal * (2.0 * direction.dot(normal))
}

/// Snell's law. The normal must point against the incident ray.
pub fn refract(direction: Vec3, normal: Vec3, eta: f32) -> Option<Vec3> {
    let cosine = (-direction.dot(normal)).clamp(0.0, 1.0);
    let k = 1.0 - eta * eta * (1.0 - cosine * cosine);
    if k < 0.0 {
        None
    } else {
        Some((direction * eta + normal * (eta * cosine - k.sqrt())).normalized())
    }
}

pub fn fresnel(cosine: f32, base: f32) -> f32 {
    base + (1.0 - base) * (1.0 - cosine.clamp(0.0, 1.0)).powi(5)
}

pub fn water_transmittance(distance: f32) -> Vec3 {
    Vec3::new(
        (-distance * 0.48).exp(),
        (-distance * 0.10).exp(),
        (-distance * 0.045).exp(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn snell_bends_toward_normal_and_handles_total_internal_reflection() {
        let normal = Vec3::new(0.0, 1.0, 0.0);
        let incident = Vec3::new(0.7, -0.7, 0.0).normalized();
        let transmitted = refract(incident, normal, 1.0 / 1.333).unwrap();
        assert!(transmitted.x < incident.x);
        assert!((transmitted.length() - 1.0).abs() < 1e-5);
        assert!(refract(Vec3::new(0.99, -0.1, 0.0).normalized(), normal, 1.333).is_none());
        assert!(reflect(incident, normal).y > 0.0);
        assert!(fresnel(0.0, 0.02) > fresnel(1.0, 0.02));
    }
}
