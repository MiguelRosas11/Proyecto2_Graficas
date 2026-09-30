use crate::{
    camera::Camera,
    lighting::Lighting,
    render::{Frame, Renderer},
    texture::Textures,
    world::generation::{GenerationConfig, generate},
};

// Controlled ablations: same camera, scene, resolution and frozen animation time.
// Differences include interactions and must not be summed as independent costs.
pub fn run(textures: &Textures, width: usize, seed: u32) {
    let world = generate(&GenerationConfig { seed });
    for view in 1..=3 {
        let camera = Camera::preset(view);
        for mode in 0..4 {
            let mut lighting = Lighting::from_world(&world);
            lighting.shadows = mode != 2;
            let mut renderer = Renderer::new(width, width * 9 / 16);
            let frame = Frame {
                flat_water: mode == 1,
                primary_only: mode == 3,
                ..Frame::default()
            };
            for _ in 0..3 {
                renderer.draw(&world, textures, &lighting, &camera, frame);
            }
            let mut trace = Vec::new();
            let mut post = Vec::new();
            for _ in 0..60 {
                renderer.draw(&world, textures, &lighting, &camera, frame);
                trace.push(renderer.trace_ms);
                post.push(renderer.post_ms);
            }
            trace.sort_by(f64::total_cmp);
            post.sort_by(f64::total_cmp);
            println!(
                "view={view} mode={} trace={:.2}ms post={:.2}ms median_total~={:.2}ms lights={}",
                ["full", "flat-water", "no-shadows", "primary-only"][mode],
                trace[30],
                post[30],
                trace[30] + post[30],
                lighting.lights.len()
            );
        }
    }
}
