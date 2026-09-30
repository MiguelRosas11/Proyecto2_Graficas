#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::Window;
#[derive(Default)]
pub struct Input {
    pub mouse_delta: (f32, f32),
    pub zoom: f32,
    pub reset_camera: bool,
    pub orbit: f32,
    pub elevation: f32,
    pub view: Option<u8>,
    pub toggle_rotation: bool,
}
