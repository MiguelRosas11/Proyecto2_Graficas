use crate::{
    environment,
    lighting::{Lighting, Surface},
    material::Block,
    math::Vec3,
    optics,
    ray::{self, Hit},
    texture::Textures,
    world::World,
};
pub struct Shading<'a> {
    pub world: &'a World,
    pub textures: &'a Textures,
    pub lighting: &'a Lighting,
    pub time: f32,
    pub flat_water: bool,
    pub primary_only: bool,
}
impl Shading<'_> {
    pub fn trace(&self, origin: Vec3, direction: Vec3, depth: u8) -> Vec3 {
        let Some(hit) =
            ray::cast_visible(self.world, self.textures, origin, direction, 180.0, false)
        else {
            return environment::SKYBOX.sample(direction);
        };
        if self.primary_only {
            return Vec3::new(0.3, 0.3, 0.3);
        }
        let mut color = if hit.block == Block::Water {
            self.water(&hit, direction, depth)
        } else {
            self.opaque(&hit, direction)
        };
        if ray::in_water(self.world, origin) {
            color = color.hadamard(optics::water_transmittance(hit.distance));
        }
        color
    }
    fn opaque(&self, hit: &Hit, view: Vec3) -> Vec3 {
        let (u, v) = ray::face_uv(hit);
        let tex = if hit.block == Block::Lava {
            self.textures
                .sample_animated(hit.block.texture(hit.normal), u, v, self.time)
        } else {
            self.textures.sample(hit.block.texture(hit.normal), u, v)
        };
        let base = tex.hadamard(tex);
        let material = hit.block.material();
        let mask = if hit.block == Block::Torch {
            if v < 0.3 || hit.normal.y > 0.5 {
                1.0
            } else {
                0.0
            }
        } else {
            hit.block.emission_mask(tex)
        };
        let emission = base * (material.emission * mask);
        if matches!(hit.block, Block::Lava | Block::Torch) && mask > 0.0 {
            return emission;
        }
        let diffuse = self.lighting.diffuse(self.world, self.textures, hit);
        let specular = if material.specular >= 0.3 {
            self.lighting
                .evaluate(
                    self.world,
                    self.textures,
                    &Surface {
                        diffuse: false,
                        point: hit.point,
                        normal: hit.normal,
                        view,
                        shininess: material.shininess,
                        specular: material.specular,
                    },
                    self.time,
                )
                .specular
        } else {
            Vec3::default()
        };
        base.hadamard(diffuse) * material.albedo + specular + emission
    }
    fn water(&self, hit: &Hit, view: Vec3, depth: u8) -> Vec3 {
        let material = hit.block.material();
        if self.flat_water {
            return Vec3::new(0.02, 0.08, 0.1);
        }
        let entering = view.dot(hit.normal) < 0.0;
        let geometric = if entering { hit.normal } else { -hit.normal };
        let mut normal = geometric;
        if normal.y.abs() > 0.5 {
            let p = hit.point;
            let dx = 0.023 * (p.x * 1.4 + p.z * 0.4 + self.time * 1.1).sin()
                + 0.012 * (p.x * 3.2 - p.z + self.time * 0.8).cos();
            let dz = 0.025 * (p.z * 1.6 - p.x * 0.3 + self.time).cos();
            normal = (normal + Vec3::new(dx, 0.0, dz)).normalized();
            if normal.dot(view) > -0.001 {
                normal = geometric;
            }
        }
        let reflected = optics::reflect(view, normal).normalized();
        let refracted = optics::refract(
            view,
            normal,
            if entering {
                1.0 / material.refractive_index
            } else {
                material.refractive_index
            },
        );
        let f = if refracted.is_none() {
            1.0
        } else {
            optics::fresnel(-view.dot(normal), material.reflectivity)
        };
        // Keep the main reflection; avoid recursive reflection branches inside water.
        let reflection = if depth == 0 {
            self.trace(hit.point + geometric * 0.004, reflected, depth + 1)
        } else {
            environment::SKYBOX.sample(reflected)
        };
        let transmission = if let Some(d) = refracted {
            if depth < 2 {
                self.trace(hit.point - geometric * 0.004, d, depth + 1)
            } else {
                Vec3::default()
            }
        } else {
            Vec3::default()
        };
        let light = self.lighting.evaluate(
            self.world,
            self.textures,
            &Surface {
                diffuse: false,
                point: hit.point,
                normal,
                view,
                shininess: 180.0,
                specular: 0.35,
            },
            self.time,
        );
        reflection * f + transmission * ((1.0 - f) * material.transparency) + light.specular
    }
}
