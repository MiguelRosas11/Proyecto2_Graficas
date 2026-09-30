use crate::{
    material::{TEXTURE_NAMES, TextureId},
    math::Vec3,
};
use std::{fs, path::Path};

struct Texture {
    width: usize,
    height: usize,
    pixels: Vec<Vec3>,
    alpha: Vec<u8>,
}
pub struct Textures {
    images: Vec<Texture>,
}
impl Textures {
    pub fn load(directory: &Path) -> Result<Self, String> {
        let images = TEXTURE_NAMES
            .iter()
            .map(|name| {
                let path = directory.join(format!("{name}.ppm"));
                let data = fs::read(&path).map_err(|e| {
                    format!(
                        "{}: {e}. Ejecuta scripts/import-textures.ps1 primero.",
                        path.display()
                    )
                })?;
                let mut texture =
                    parse_ppm(&data).map_err(|e| format!("{}: {e}", path.display()))?;
                if let Ok(alpha) = fs::read(directory.join(format!("{name}.alpha"))) {
                    if alpha.len() != texture.width * texture.height {
                        return Err(format!("Mascara alfa invalida: {name}"));
                    }
                    texture.alpha = alpha;
                }
                Ok(texture)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { images })
    }
    pub fn sample(&self, id: TextureId, u: f32, v: f32) -> Vec3 {
        self.sample_animated(id, u, v, 0.0)
    }
    pub fn opaque(&self, id: TextureId, u: f32, v: f32) -> bool {
        let t = &self.images[id as usize];
        let x = ((u.rem_euclid(1.0) * t.width as f32) as usize).min(t.width - 1);
        let y = ((v.rem_euclid(1.0) * t.height as f32) as usize).min(t.height - 1);
        t.alpha[y * t.width + x] > 127
    }
    pub fn sample_animated(&self, id: TextureId, u: f32, v: f32, time: f32) -> Vec3 {
        let texture = &self.images[id as usize];
        let animated = matches!(id, TextureId::Water | TextureId::Lava);
        let tile_height = if animated {
            texture.width.min(texture.height)
        } else {
            texture.height
        };
        let frame = if animated {
            (time * 7.0) as usize % (texture.height / tile_height)
        } else {
            0
        };
        let x = ((u.rem_euclid(1.0) * texture.width as f32) as usize).min(texture.width - 1);
        let y = ((v.rem_euclid(1.0) * tile_height as f32) as usize).min(tile_height - 1)
            + frame * tile_height;
        texture.pixels[y * texture.width + x]
    }
}

fn parse_ppm(data: &[u8]) -> Result<Texture, String> {
    let mut position = 0;
    fn token<'a>(data: &'a [u8], position: &mut usize) -> Result<&'a str, String> {
        loop {
            while *position < data.len() && data[*position].is_ascii_whitespace() {
                *position += 1;
            }
            if data.get(*position) == Some(&b'#') {
                while *position < data.len() && data[*position] != b'\n' {
                    *position += 1;
                }
            } else {
                break;
            }
        }
        let start = *position;
        while *position < data.len() && !data[*position].is_ascii_whitespace() {
            *position += 1;
        }
        std::str::from_utf8(&data[start..*position]).map_err(|_| "Encabezado PPM invalido".into())
    }
    if token(data, &mut position)? != "P6" {
        return Err("Se esperaba PPM P6".into());
    }
    let w = token(data, &mut position)?
        .parse::<usize>()
        .map_err(|_| "Ancho invalido")?;
    let h = token(data, &mut position)?
        .parse::<usize>()
        .map_err(|_| "Alto invalido")?;
    if token(data, &mut position)? != "255" || w == 0 || h == 0 || w > 4096 || h > 4096 {
        return Err("Dimensiones o profundidad invalidas".into());
    }
    if !data.get(position).is_some_and(u8::is_ascii_whitespace) {
        return Err("Falta separador PPM".into());
    }
    if data.get(position..position + 2) == Some(b"\r\n") {
        position += 2;
    } else {
        position += 1;
    }
    let bytes = data.get(position..).ok_or("Faltan pixeles")?;
    if bytes.len() != w * h * 3 {
        return Err("Cantidad de pixeles invalida".into());
    }
    let pixels = bytes
        .chunks_exact(3)
        .map(|p| Vec3::new(p[0] as f32, p[1] as f32, p[2] as f32) / 255.0)
        .collect();
    Ok(Texture {
        width: w,
        height: h,
        pixels,
        alpha: vec![255; w * h],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_whitespace_valued_first_pixel() {
        let mut bytes = b"P6\n# comment\n1 1\n255\n".to_vec();
        bytes.extend([10, 32, 13]);
        let t = parse_ppm(&bytes).unwrap();
        assert_eq!(t.pixels[0], Vec3::new(10.0, 32.0, 13.0) / 255.0);
        assert!(parse_ppm(b"P6\n5000 5000\n255\n").is_err());
    }
}
