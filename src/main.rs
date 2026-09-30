#[cfg(windows)]
mod app;
mod camera;
mod environment;
mod lighting;
mod material;
mod math;
mod optics;
mod platform;
mod postprocess;
mod profiling;
mod ray;
mod render;
mod shading;
mod texture;
mod world;

use camera::Camera;
use lighting::Lighting;
use render::{Frame, Renderer};
use std::{env, path::PathBuf, time::Instant};
use texture::Textures;
use world::generation::{self, GenerationConfig};

struct Options {
    snapshot: Option<PathBuf>,
    assets: PathBuf,
    seed: u32,
    width: usize,
    samples: u32,
    benchmark: bool,
    orbit_test: bool,
    profile: bool,
    smoke_test: bool,
    view: u8,
    angle: Option<f32>,
}
fn options() -> Result<Options, String> {
    let mut options = Options {
        snapshot: None,
        assets: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/textures"),
        seed: 42,
        width: 640,
        samples: 1,
        benchmark: false,
        orbit_test: false,
        profile: false,
        smoke_test: false,
        view: 1,
        angle: None,
    };
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--snapshot" => {
                options.snapshot = Some(args.next().ok_or("Falta ruta para --snapshot")?.into())
            }
            "--assets" => options.assets = args.next().ok_or("Falta ruta para --assets")?.into(),
            "--seed" => {
                options.seed = args
                    .next()
                    .ok_or("Falta numero para --seed")?
                    .parse()
                    .map_err(|_| "Semilla invalida")?
            }
            "--width" => {
                options.width = args
                    .next()
                    .ok_or("Falta numero para --width")?
                    .parse()
                    .map_err(|_| "Resolucion invalida")?
            }
            "--samples" => {
                options.samples = args
                    .next()
                    .ok_or("Falta cantidad de muestras")?
                    .parse()
                    .map_err(|_| "Muestras invalidas")?
            }
            "--profile" => options.profile = true,
            "--orbit-test" => options.orbit_test = true,
            "--benchmark" => options.benchmark = true,
            "--angle" => {
                let value = args
                    .next()
                    .ok_or("Falta angulo")?
                    .parse::<f32>()
                    .map_err(|_| "Angulo invalido")?;
                if !value.is_finite() {
                    return Err("Angulo no finito".into());
                }
                options.angle = Some(value);
            }
            "--view" => {
                options.view = match args
                    .next()
                    .ok_or("Falta vista: overview, azalea, lagoon")?
                    .as_str()
                {
                    "overview" => 1,
                    "azalea" | "minerals" => 2,

                    "lagoon" => 3,
                    _ => return Err("Vista invalida: overview, azalea, lagoon".into()),
                }
            }
            "--smoke-test" => options.smoke_test = true,
            "--help" | "-h" => {
                println!(
                    "cargo run --release -- [--seed 42] [--width 640] [--assets ruta]\n  --view overview|azalea|lagoon\n  --snapshot captura.bmp   Render sin ventana (opcional --samples 16)\n  --profile               Compara agua, sombras y postprocesado\n  --benchmark             Mide 30 frames sin ventana\n  --orbit-test            Prueba 1440 frames de rotacion\n\nClic: mouse orbital. Q/E: girar. Flechas arriba/abajo: altura\n1-3: vistas. Espacio: giro automatico. --angle grados: azimut inicial\nW/S: acercar/alejar, R: restaurar camara, Esc: liberar mouse\nCerrar ventana: salir. Camara exclusivamente orbital; escena fija."
                );
                std::process::exit(0);
            }
            _ => return Err(format!("Opcion desconocida: {arg}. Usa --help.")),
        }
    }
    if !(1..=64).contains(&options.samples) {
        return Err("--samples debe estar entre 1 y 64".into());
    }
    if !(160..=1920).contains(&options.width) {
        return Err("--width debe estar entre 160 y 1920".into());
    }
    Ok(options)
}
fn run() -> Result<(), String> {
    let options = options()?;
    let textures = Textures::load(&options.assets)?;
    if options.profile {
        profiling::run(&textures, options.width, options.seed);
        return Ok(());
    }
    if options.snapshot.is_some() || options.benchmark || options.orbit_test {
        let world = generation::generate(&GenerationConfig { seed: options.seed });
        let mut renderer = Renderer::new(options.width, options.width * 9 / 16);
        let mut camera = Camera::preset(options.view);
        if let Some(angle) = options.angle {
            camera.set_angle(angle);
        }
        let lighting = Lighting::from_world(&world);
        if options.orbit_test {
            for frame in 0..1440 {
                camera.set_angle(frame as f32 * 0.5);
                renderer.draw(
                    &world,
                    &textures,
                    &lighting,
                    &camera,
                    Frame {
                        time: frame as f32 / 30.0,
                        sample: 0,
                        ..Frame::default()
                    },
                );
                if frame % 90 == 0 {
                    println!("Orbit frame {frame}/1440");
                }
            }
            println!("Orbit test passed");
            return Ok(());
        }
        renderer.draw(&world, &textures, &lighting, &camera, Frame::default());
        let start = Instant::now();
        let frames = if options.benchmark { 30 } else { 1 };
        for _ in 0..frames {
            renderer.draw(&world, &textures, &lighting, &camera, Frame::default());
        }
        println!(
            "{} bloques | {}x{} | {:.1} ms/frame | {:.1} FPS (render CPU, sin ventana)",
            world.solid_count(),
            renderer.width,
            renderer.height,
            start.elapsed().as_secs_f64() * 1000.0 / frames as f64,
            frames as f64 / start.elapsed().as_secs_f64()
        );
        if let Some(path) = options.snapshot {
            for sample in 1..options.samples {
                renderer.draw(
                    &world,
                    &textures,
                    &lighting,
                    &camera,
                    Frame {
                        time: 0.0,
                        sample,
                        ..Frame::default()
                    },
                );
            }
            if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            renderer.save_bmp(&path).map_err(|e| e.to_string())?;
            println!("Captura: {}", path.display());
        }
        return Ok(());
    }
    #[cfg(windows)]
    {
        if options.smoke_test {
            std::fs::create_dir_all("captures").map_err(|e| e.to_string())?;
        }
        app::run(
            textures,
            options.seed,
            options.width,
            options.smoke_test,
            options.view,
            options.angle,
        )
    }
    #[cfg(not(windows))]
    {
        Err(
            "La ventana de esta version requiere Windows. Puedes usar --snapshot o --benchmark."
                .into(),
        )
    }
}
fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}
