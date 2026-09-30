//! Minimal Win32 boundary. The renderer and world never call operating-system APIs.
use super::Input;
use std::{
    ffi::c_void,
    mem::zeroed,
    ptr::{null, null_mut},
};
type Handle = *mut c_void;
#[repr(C)]
struct Point {
    x: i32,
    y: i32,
}
#[repr(C)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}
#[repr(C)]
struct Message {
    window: Handle,
    id: u32,
    wparam: usize,
    lparam: isize,
    time: u32,
    point: Point,
    private: u32,
}
#[repr(C)]
struct WindowClass {
    style: u32,
    procedure: Option<unsafe extern "system" fn(Handle, u32, usize, isize) -> isize>,
    class_extra: i32,
    window_extra: i32,
    instance: Handle,
    icon: Handle,
    cursor: Handle,
    background: Handle,
    menu: *const u16,
    name: *const u16,
}
#[repr(C)]
struct BitmapHeader {
    size: u32,
    width: i32,
    height: i32,
    planes: u16,
    bits: u16,
    compression: u32,
    image_size: u32,
    x_density: i32,
    y_density: i32,
    colors: u32,
    important: u32,
}
#[repr(C)]
struct BitmapInfo {
    header: BitmapHeader,
    colors: [u32; 1],
}

#[link(name = "user32")]
unsafe extern "system" {
    fn RegisterClassW(class: *const WindowClass) -> u16;
    fn CreateWindowExW(
        ex: u32,
        class: *const u16,
        title: *const u16,
        style: u32,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        parent: Handle,
        menu: Handle,
        instance: Handle,
        param: *mut c_void,
    ) -> Handle;
    fn DefWindowProcW(window: Handle, id: u32, wparam: usize, lparam: isize) -> isize;
    fn DestroyWindow(window: Handle) -> i32;
    fn PostQuitMessage(code: i32);
    fn ShowWindow(window: Handle, command: i32) -> i32;
    fn PeekMessageW(message: *mut Message, window: Handle, min: u32, max: u32, remove: u32) -> i32;
    fn TranslateMessage(message: *const Message) -> i32;
    fn DispatchMessageW(message: *const Message) -> isize;
    fn GetDC(window: Handle) -> Handle;
    fn ReleaseDC(window: Handle, dc: Handle) -> i32;
    fn GetClientRect(window: Handle, rect: *mut Rect) -> i32;
    fn ClientToScreen(window: Handle, point: *mut Point) -> i32;
    fn GetCursorPos(point: *mut Point) -> i32;
    fn SetCursorPos(x: i32, y: i32) -> i32;
    fn GetAsyncKeyState(key: i32) -> i16;
    fn GetForegroundWindow() -> Handle;
    fn ShowCursor(show: i32) -> i32;
    fn ClipCursor(rect: *const Rect) -> i32;
    fn LoadCursorW(instance: Handle, name: *const u16) -> Handle;
    fn SetWindowTextW(window: Handle, text: *const u16) -> i32;
    fn SetProcessDPIAware() -> i32;
    fn IsWindow(window: Handle) -> i32;
    fn IsIconic(window: Handle) -> i32;
}
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(name: *const u16) -> Handle;
    fn GetLastError() -> u32;
}
#[link(name = "gdi32")]
unsafe extern "system" {
    fn StretchDIBits(
        dc: Handle,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        sx: i32,
        sy: i32,
        sw: i32,
        sh: i32,
        pixels: *const c_void,
        info: *const BitmapInfo,
        usage: u32,
        operation: u32,
    ) -> i32;
    fn SetStretchBltMode(dc: Handle, mode: i32) -> i32;
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}
unsafe extern "system" fn procedure(
    window: Handle,
    id: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    // All messages not owned by this boundary are delegated to Windows.
    unsafe {
        match id {
            0x0010 => {
                DestroyWindow(window);
                0
            }
            0x0002 => {
                PostQuitMessage(0);
                0
            }
            0x0014 => 1, // The next full framebuffer replaces the background.
            _ => DefWindowProcW(window, id, wparam, lparam),
        }
    }
}

pub struct Window {
    handle: Handle,
    captured: bool,
    running: bool,
}
impl Window {
    pub fn new() -> Result<Self, String> {
        // Pointers refer to live, correctly aligned repr(C) structures and terminated UTF-16.
        unsafe {
            SetProcessDPIAware();
            let name = wide("RustVoxelDiorama");
            let title = wide(
                "Santuario de las profundidades | 1-3 vistas | Q/E orbita | Clic para girar | Esc libera",
            );
            let instance = GetModuleHandleW(null());
            let class = WindowClass {
                style: 3,
                procedure: Some(procedure),
                class_extra: 0,
                window_extra: 0,
                instance,
                icon: null_mut(),
                cursor: LoadCursorW(null_mut(), 32512usize as *const u16),
                background: null_mut(),
                menu: null(),
                name: name.as_ptr(),
            };
            if RegisterClassW(&class) == 0 {
                return Err(format!(
                    "No se pudo registrar la ventana: {}",
                    GetLastError()
                ));
            }
            let handle = CreateWindowExW(
                0,
                name.as_ptr(),
                title.as_ptr(),
                0x00cf0000,
                0x80000000u32 as i32,
                0x80000000u32 as i32,
                1280,
                760,
                null_mut(),
                null_mut(),
                instance,
                null_mut(),
            );
            if handle.is_null() {
                return Err(format!("No se pudo abrir la ventana: {}", GetLastError()));
            }
            ShowWindow(handle, 5);
            Ok(Self {
                handle,
                captured: false,
                running: true,
            })
        }
    }
    pub fn size(&self) -> (usize, usize) {
        unsafe {
            let mut r: Rect = zeroed();
            GetClientRect(self.handle, &mut r);
            (
                (r.right - r.left).max(0) as usize,
                (r.bottom - r.top).max(0) as usize,
            )
        }
    }
    pub fn minimized(&self) -> bool {
        unsafe { IsIconic(self.handle) != 0 }
    }
    fn center_and_bounds(&self) -> (Point, Rect) {
        let (w, h) = self.size();
        unsafe {
            let mut origin = Point { x: 0, y: 0 };
            ClientToScreen(self.handle, &mut origin);
            (
                Point {
                    x: origin.x + w as i32 / 2,
                    y: origin.y + h as i32 / 2,
                },
                Rect {
                    left: origin.x,
                    top: origin.y,
                    right: origin.x + w as i32,
                    bottom: origin.y + h as i32,
                },
            )
        }
    }
    fn capture(&mut self, enabled: bool) {
        if enabled == self.captured {
            return;
        }
        unsafe {
            if enabled {
                let (center, bounds) = self.center_and_bounds();
                ClipCursor(&bounds);
                SetCursorPos(center.x, center.y);
                ShowCursor(0);
            } else {
                ClipCursor(null());
                ShowCursor(1);
            }
        }
        self.captured = enabled;
    }
    pub fn poll(&mut self) -> Option<Input> {
        let mut input = Input::default();
        unsafe {
            let mut message: Message = zeroed();
            while PeekMessageW(&mut message, null_mut(), 0, 0, 1) != 0 {
                match message.id {
                    0x0012 => self.running = false,
                    0x0201 => {
                        if !self.captured && GetForegroundWindow() == self.handle {
                            self.capture(true);
                        }
                    }
                    0x0100 => {
                        if message.wparam == 27 {
                            self.capture(false);
                        }
                        if message.wparam == b'R' as usize && message.lparam & (1 << 30) == 0 {
                            input.reset_camera = true;
                        }
                        if message.lparam & (1 << 30) == 0 {
                            if (49..=51).contains(&message.wparam) {
                                input.view = Some((message.wparam - 48) as u8);
                            }
                            if message.wparam == 32 {
                                input.toggle_rotation = true;
                            }
                        }
                    }
                    _ => {}
                }
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
            if !self.running {
                self.capture(false);
                return None;
            }
            if GetForegroundWindow() != self.handle {
                self.capture(false);
                return Some(Input::default());
            }
            input.zoom = if GetAsyncKeyState(87) < 0 { 1.0 } else { 0.0 }
                - if GetAsyncKeyState(83) < 0 { 1.0 } else { 0.0 };
            input.orbit = if GetAsyncKeyState(69) < 0 { 1.0 } else { 0.0 }
                - if GetAsyncKeyState(81) < 0 { 1.0 } else { 0.0 };
            if self.captured {
                let (center, bounds) = self.center_and_bounds();
                let mut p: Point = zeroed();
                GetCursorPos(&mut p);
                input.mouse_delta = ((p.x - center.x) as f32, (p.y - center.y) as f32);
                ClipCursor(&bounds);
                SetCursorPos(center.x, center.y);
            }
        }
        unsafe {
            input.elevation = if GetAsyncKeyState(38) < 0 { 1.0 } else { 0.0 }
                - if GetAsyncKeyState(40) < 0 { 1.0 } else { 0.0 };
        }
        Some(input)
    }
    pub fn present(&self, pixels: &[u32], width: usize, height: usize) {
        assert_eq!(pixels.len(), width * height);
        let (w, h) = self.size();
        if w == 0 || h == 0 {
            return;
        }
        let info = BitmapInfo {
            header: BitmapHeader {
                size: 40,
                width: width as i32,
                height: -(height as i32),
                planes: 1,
                bits: 32,
                compression: 0,
                image_size: 0,
                x_density: 0,
                y_density: 0,
                colors: 0,
                important: 0,
            },
            colors: [0],
        };
        unsafe {
            let dc = GetDC(self.handle);
            if !dc.is_null() {
                SetStretchBltMode(dc, 3);
                StretchDIBits(
                    dc,
                    0,
                    0,
                    w as i32,
                    h as i32,
                    0,
                    0,
                    width as i32,
                    height as i32,
                    pixels.as_ptr().cast(),
                    &info,
                    0,
                    0x00cc0020,
                );
                ReleaseDC(self.handle, dc);
            }
        }
    }
    pub fn title(&self, text: &str) {
        let text = wide(text);
        unsafe {
            SetWindowTextW(self.handle, text.as_ptr());
        }
    }
}
impl Drop for Window {
    fn drop(&mut self) {
        self.capture(false);
        unsafe {
            if IsWindow(self.handle) != 0 {
                DestroyWindow(self.handle);
            }
        }
    }
}
