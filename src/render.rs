use crate::{
    camera::Camera, lighting::Lighting, math::Vec3, postprocess, shading::Shading,
    texture::Textures, world::World,
};
use std::{
    fs::File,
    io::{self, Write},
    path::Path,
    thread,
};

pub struct Renderer {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>,
    workers: usize,
    pub trace_ms: f64,
    pub post_ms: f64,
    hdr: Vec<Vec3>,
    scratch: Vec<Vec3>,
    bloom: Vec<Vec3>,
}
#[derive(Default, Clone, Copy)]
pub struct Frame {
    pub time: f32,
    pub sample: u32,
    pub flat_water: bool,
    pub primary_only: bool,
}
impl Renderer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; width * height],
            hdr: vec![Vec3::default(); width * height],
            scratch: vec![Vec3::default(); width * height],
            bloom: vec![Vec3::default(); width * height],
            trace_ms: 0.0,
            post_ms: 0.0,
            workers: thread::available_parallelism()
                .map_or(1, usize::from)
                .min(12),
        }
    }
    pub fn draw(
        &mut self,
        world: &World,
        textures: &Textures,
        lighting: &Lighting,
        camera: &Camera,
        frame: Frame,
    ) {
        let started = std::time::Instant::now();
        let width = self.width;
        let height = self.height;
        let forward = camera.forward();
        let right = camera.right();
        let up = camera.up();
        let scale = (camera.fov * 0.5).tan();
        let aspect = width as f32 / height as f32;
        let shading = Shading {
            world,
            textures,
            lighting,
            time: frame.time,
            flat_water: frame.flat_water,
            primary_only: frame.primary_only,
        };
        let jitter = sample_offset(frame.sample);
        let rows = 4;
        let jobs = std::sync::Mutex::new(self.hdr.chunks_mut(rows * width).enumerate());
        thread::scope(|scope| {
            for _ in 0..self.workers {
                let jobs = &jobs;
                let shading = &shading;
                scope.spawn(move || {
                    loop {
                        let job = jobs.lock().unwrap().next();
                        let Some((chunk_id, chunk)) = job else {
                            break;
                        };
                        for (i, pixel) in chunk.iter_mut().enumerate() {
                            let x = i % width;
                            let y = chunk_id * rows + i / width;
                            let sx =
                                (2.0 * (x as f32 + jitter.0) / width as f32 - 1.0) * aspect * scale;
                            let sy = (1.0 - 2.0 * (y as f32 + jitter.1) / height as f32) * scale;
                            let dir = (forward + right * sx + up * sy).normalized();
                            let value = shading.trace(camera.position, dir, 0);
                            *pixel = if frame.sample == 0 {
                                value
                            } else {
                                pixel.lerp(value, 1.0 / (frame.sample + 1) as f32)
                            };
                        }
                    }
                });
            }
        });
        self.trace_ms = started.elapsed().as_secs_f64() * 1000.0;
        let post_started = std::time::Instant::now();
        postprocess::finish(
            &self.hdr,
            &mut self.scratch,
            &mut self.bloom,
            &mut self.pixels,
            width,
            height,
        );
        self.post_ms = post_started.elapsed().as_secs_f64() * 1000.0;
    }
    pub fn save_bmp(&self, path: &Path) -> io::Result<()> {
        let mut f = File::create(path)?;
        let size = (54 + self.width * self.height * 4) as u32;
        f.write_all(b"BM")?;
        f.write_all(&size.to_le_bytes())?;
        f.write_all(&[0; 4])?;
        f.write_all(&54u32.to_le_bytes())?;
        f.write_all(&40u32.to_le_bytes())?;
        f.write_all(&(self.width as i32).to_le_bytes())?;
        f.write_all(&(-(self.height as i32)).to_le_bytes())?;
        f.write_all(&1u16.to_le_bytes())?;
        f.write_all(&32u16.to_le_bytes())?;
        f.write_all(&[0; 24])?;
        for pixel in &self.pixels {
            f.write_all(&pixel.to_le_bytes())?;
        }
        Ok(())
    }
}

// Deterministic subpixel samples; accumulation happens in linear HDR space.
fn sample_offset(sample: u32) -> (f32, f32) {
    if sample == 0 {
        return (0.5, 0.5);
    }
    fn radical(mut n: u32, base: u32) -> f32 {
        let mut value = 0.0;
        let mut weight = 1.0;
        while n > 0 {
            weight /= base as f32;
            value += (n % base) as f32 * weight;
            n /= base;
        }
        value
    }
    (radical(sample, 2), radical(sample, 3))
}
