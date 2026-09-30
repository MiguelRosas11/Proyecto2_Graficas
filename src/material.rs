use crate::math::Vec3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Block {
    Air,
    Grass,
    Dirt,
    Stone,
    Wood,
    Leaves,
    Sand,
    Water,
    Lava,
    Torch,
    Deepslate,
    Moss,
    Diamond,
    Redstone,
    Amethyst,
    Vine,
    Gold,
    Emerald,
    Lapis,
    Iron,
    Azalea,
    Berries,
    Fern,
    Clay,
}

#[derive(Clone, Copy)]
#[repr(usize)]
pub enum TextureId {
    GrassTop,
    GrassSide,
    Dirt,
    Stone,
    WoodSide,
    WoodTop,
    Leaves,
    Sand,
    Water,
    Lava,
    Torch,
    Deepslate,
    Moss,
    Diamond,
    Redstone,
    Amethyst,
    Vine,
    Gold,
    Emerald,
    Lapis,
    Iron,
    Azalea,
    Berries,
    Fern,
    Clay,
}

pub const TEXTURE_NAMES: [&str; 25] = [
    "grass_top",
    "grass_side",
    "dirt",
    "stone",
    "oak_log",
    "oak_log_top",
    "oak_leaves",
    "sand",
    "water",
    "lava",
    "torch",
    "deepslate",
    "moss",
    "diamond",
    "redstone",
    "amethyst",
    "vine",
    "gold",
    "emerald",
    "lapis",
    "iron",
    "azalea",
    "berries",
    "fern",
    "clay",
];

/// Optical properties are independent of geometry and texture loading.
pub struct Material {
    pub albedo: f32,
    pub specular: f32,
    pub shininess: f32,
    pub transparency: f32,
    pub reflectivity: f32,
    pub refractive_index: f32,
    pub emission: f32,
}

impl Block {
    pub fn material(self) -> Material {
        let (albedo, specular, shininess) = match self {
            Self::Grass => (0.90, 0.025, 12.0),
            Self::Dirt => (0.78, 0.01, 6.0),
            Self::Stone => (0.83, 0.10, 28.0),
            Self::Wood => (0.82, 0.045, 18.0),
            Self::Leaves => (0.88, 0.07, 22.0),
            Self::Sand => (0.95, 0.035, 10.0),
            Self::Water => (0.35, 0.9, 140.0),
            Self::Lava => (0.85, 0.08, 10.0),
            Self::Torch => (0.85, 0.04, 12.0),
            Self::Air => (0.0, 0.0, 1.0),
            Self::Deepslate => (0.80, 0.12, 32.0),
            Self::Moss | Self::Vine => (0.88, 0.01, 8.0),
            Self::Diamond => (0.82, 0.48, 90.0),
            Self::Redstone => (0.82, 0.22, 40.0),
            Self::Amethyst => (0.85, 0.45, 80.0),
            Self::Gold | Self::Emerald | Self::Lapis | Self::Iron => (0.85, 0.38, 65.0),
            Self::Azalea | Self::Fern | Self::Berries => (0.92, 0.03, 12.0),
            Self::Clay => (0.9, 0.015, 8.0),
        };
        Material {
            albedo,
            specular,
            shininess,
            transparency: if self == Self::Water { 0.96 } else { 0.0 },
            reflectivity: if self == Self::Water { 0.02 } else { 0.0 },
            refractive_index: if self == Self::Water { 1.333 } else { 1.0 },
            emission: match self {
                Self::Lava => 2.8,
                Self::Torch => 4.5,
                Self::Diamond => 3.0,
                Self::Redstone => 2.0,
                Self::Amethyst => 0.8,
                Self::Gold | Self::Emerald | Self::Lapis | Self::Iron => 2.4,
                Self::Berries => 4.0,
                _ => 0.0,
            },
        }
    }

    pub fn texture(self, normal: Vec3) -> TextureId {
        match self {
            Self::Grass if normal.y > 0.5 => TextureId::GrassTop,
            Self::Grass if normal.y < -0.5 => TextureId::Dirt,
            Self::Grass => TextureId::GrassSide,
            Self::Dirt | Self::Air => TextureId::Dirt,
            Self::Stone => TextureId::Stone,
            Self::Wood if normal.y.abs() > 0.5 => TextureId::WoodTop,
            Self::Wood => TextureId::WoodSide,
            Self::Leaves => TextureId::Leaves,
            Self::Sand => TextureId::Sand,
            Self::Water => TextureId::Water,
            Self::Lava => TextureId::Lava,
            Self::Torch => TextureId::Torch,
            Self::Deepslate => TextureId::Deepslate,
            Self::Moss => TextureId::Moss,
            Self::Diamond => TextureId::Diamond,
            Self::Redstone => TextureId::Redstone,
            Self::Amethyst => TextureId::Amethyst,
            Self::Vine => TextureId::Vine,
            Self::Gold => TextureId::Gold,
            Self::Emerald => TextureId::Emerald,
            Self::Lapis => TextureId::Lapis,
            Self::Iron => TextureId::Iron,
            Self::Azalea => TextureId::Azalea,
            Self::Berries => TextureId::Berries,
            Self::Fern => TextureId::Fern,
            Self::Clay => TextureId::Clay,
        }
    }

    pub fn mineral_color(self) -> Option<Vec3> {
        match self {
            Self::Diamond => Some(Vec3::new(0.1, 0.85, 1.0)),
            Self::Redstone => Some(Vec3::new(1.0, 0.06, 0.015)),
            Self::Amethyst => Some(Vec3::new(0.55, 0.18, 1.0)),
            Self::Gold => Some(Vec3::new(1.0, 0.66, 0.12)),
            Self::Emerald => Some(Vec3::new(0.12, 1.0, 0.35)),
            Self::Lapis => Some(Vec3::new(0.12, 0.32, 1.0)),
            Self::Iron => Some(Vec3::new(1.0, 0.74, 0.53)),
            _ => None,
        }
    }

    pub fn emission_mask(self, tex: Vec3) -> f32 {
        match self {
            Self::Diamond => ((tex.y + tex.z) * 0.5 - tex.x - 0.08).max(0.0) * 3.0,
            Self::Redstone => (tex.x - tex.y.max(tex.z) - 0.10).max(0.0) * 3.0,
            Self::Amethyst => ((tex.x + tex.z) * 0.5 - tex.y).max(0.0),
            Self::Gold => ((tex.x + tex.y) * 0.5 - tex.z - 0.08).max(0.0) * 3.0,
            Self::Emerald => (tex.y - tex.x.max(tex.z) - 0.04).max(0.0) * 3.0,
            Self::Lapis => (tex.z - tex.x.max(tex.y) - 0.03).max(0.0) * 4.0,
            Self::Iron => (tex.x - tex.z - 0.04).max(0.0) * 6.0,
            Self::Berries => (tex.x - tex.z - 0.15).max(0.0) * 3.0,
            _ => 1.0,
        }
        .clamp(0.0, 1.0)
    }
    pub fn plant(self) -> bool {
        matches!(self, Self::Vine | Self::Berries | Self::Fern)
    }
    pub fn cutout(self) -> bool {
        self.plant() || matches!(self, Self::Leaves | Self::Azalea)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mineral_mask_keeps_neutral_rock_dark() {
        let rock = Vec3::new(0.4, 0.4, 0.4);
        assert_eq!(Block::Diamond.emission_mask(rock), 0.0);
        assert_eq!(Block::Redstone.emission_mask(rock), 0.0);
        assert!(Block::Diamond.emission_mask(Vec3::new(0.1, 0.9, 1.0)) > 0.8);
        assert!(Block::Redstone.emission_mask(Vec3::new(0.9, 0.1, 0.1)) > 0.8);
    }
}
