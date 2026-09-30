use crate::{
    camera::Camera,
    lighting::Lighting,
    platform::Window,
    render::{Frame, Renderer},
    texture::Textures,
    world::generation::{self, GenerationConfig},
};
use std::{
    path::Path,
    thread,
    time::{Duration, Instant},
};

const REFINEMENT_SAMPLES: u32 = 64;

pub fn run(
    textures: Textures,
    seed: u32,
    resolution: usize,
    smoke_test: bool,
    view: u8,
    angle: Option<f32>,
) -> Result<(), String> {
    let world = generation::generate(&GenerationConfig { seed });
    let lighting = Lighting::from_world(&world);
    let mut camera = Camera::preset(view);
    if let Some(angle) = angle {
        camera.set_angle(angle);
    }
    let mut window = Window::new()?;
    let mut renderer = Renderer::new(resolution, resolution * 9 / 16);
    let started = Instant::now();
    let mut last = started;
    let mut title_time = started;
    let mut frames = 0;
    let mut total = 0;
    let mut automatic = false;
    let mut changed_at = Instant::now();
    let mut frozen_time = 0.0;
    let mut samples = 0u32;
    let mut previous_camera = (camera.position, camera.forward());
    let mut previous_size = (0, 0);
    while let Some(input) = window.poll() {
        let now = Instant::now();
        let dt = (now - last).as_secs_f32().min(0.08);
        last = now;
        if window.minimized() {
            thread::sleep(Duration::from_millis(40));
            continue;
        }
        if input.reset_camera {
            camera = Camera::default();
            automatic = false;
        }
        if let Some(view) = input.view {
            camera = Camera::preset(view);
            automatic = false;
        }
        if input.toggle_rotation {
            automatic = !automatic;
        }
        camera.orbit(
            input.orbit * dt * 0.7
                + input.mouse_delta.0 * 0.004
                + if automatic { dt * 0.15 } else { 0.0 },
            input.elevation * dt * 0.5 + input.mouse_delta.1 * 0.003,
        );
        camera.zoom(input.zoom * dt * 5.0);
        let (w, h) = window.size();
        if w == 0 || h == 0 {
            thread::sleep(Duration::from_millis(40));
            continue;
        }
        let signature = (camera.position, camera.forward());
        if signature != previous_camera || (w, h) != previous_size {
            changed_at = now;
            samples = 0;
            frozen_time = started.elapsed().as_secs_f32();
        }
        previous_camera = signature;
        previous_size = (w, h);
        let refining = changed_at.elapsed() >= Duration::from_millis(350);
        let width = if refining {
            (resolution * 3).min(1920)
        } else {
            resolution
        };
        let height = (width * h / w).clamp(120, 1080);
        if renderer.width != width || renderer.height != height {
            renderer = Renderer::new(width, height);
            samples = 0;
        }
        if !refining || samples < REFINEMENT_SAMPLES {
            renderer.draw(
                &world,
                &textures,
                &lighting,
                &camera,
                Frame {
                    time: if refining {
                        frozen_time
                    } else {
                        started.elapsed().as_secs_f32()
                    },
                    sample: if refining { samples } else { 0 },
                    ..Frame::default()
                },
            );
            if refining {
                samples += 1;
            }
        }
        window.present(&renderer.pixels, renderer.width, renderer.height);
        frames += 1;
        total += 1;
        if title_time.elapsed() > Duration::from_secs(1) {
            let fps = frames as f32 / title_time.elapsed().as_secs_f32();
            let quality = if refining {
                format!("Refinado {samples}/{REFINEMENT_SAMPLES}")
            } else {
                format!("Movimiento {fps:.0} FPS")
            };
            window.title(&format!(
                "Santuario | {quality} | {}x{} | Q/E orbita | 1-3 vistas | W/S zoom",
                renderer.width, renderer.height
            ));
            frames = 0;
            title_time = now;
        }
        if smoke_test && total >= 8 && samples >= REFINEMENT_SAMPLES {
            renderer
                .save_bmp(Path::new("captures/window-smoke.bmp"))
                .map_err(|e| e.to_string())?;
            break;
        }
        let interval = if refining && samples >= REFINEMENT_SAMPLES {
            1.0 / 30.0
        } else {
            1.0 / 60.0
        };
        if let Some(rest) = Duration::from_secs_f32(interval).checked_sub(now.elapsed()) {
            thread::sleep(rest);
        }
    }
    Ok(())
}
