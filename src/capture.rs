use std::mem::size_of;
use windows::core::w;
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::Graphics::Gdi::{
    BitBlt, ClientToScreen, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject,
    GetDC, GetDIBits, ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    SRCCOPY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowW, GetClientRect, GetForegroundWindow, SetForegroundWindow,
};

#[derive(Debug, Clone, Copy)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct WindowRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

pub fn find_roblox_window() -> Option<HWND> {
    let hwnd = unsafe { FindWindowW(None, w!("Roblox")) }.ok()?;
    if hwnd.0.is_null() {
        None
    } else {
        Some(hwnd)
    }
}

pub fn is_roblox_focused(hwnd: HWND) -> bool {
    unsafe { GetForegroundWindow() == hwnd }
}

pub fn focus_roblox(hwnd: HWND) -> bool {
    unsafe { SetForegroundWindow(hwnd).as_bool() }
}

pub fn get_client_rect_screen(hwnd: HWND) -> Option<WindowRect> {
    let mut rect = RECT::default();
    unsafe { GetClientRect(hwnd, &mut rect) }.ok()?;
    let mut top_left = POINT {
        x: rect.left,
        y: rect.top,
    };
    if unsafe { !ClientToScreen(hwnd, &mut top_left).as_bool() } {
        return None;
    }
    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;
    if width <= 0 || height <= 0 {
        return None;
    }
    Some(WindowRect {
        x: top_left.x,
        y: top_left.y,
        width,
        height,
    })
}

/// Captures a single pixel from the screen at (screen_x, screen_y).
pub fn get_pixel(screen_x: i32, screen_y: i32) -> Option<Rgb> {
    unsafe {
        let hdc_screen = GetDC(HWND::default());
        if hdc_screen.is_invalid() {
            return None;
        }
        let hdc_mem = CreateCompatibleDC(hdc_screen);
        let hbm = CreateCompatibleBitmap(hdc_screen, 1, 1);
        let old = SelectObject(hdc_mem, hbm);

        let ok = BitBlt(hdc_mem, 0, 0, 1, 1, hdc_screen, screen_x, screen_y, SRCCOPY).is_ok();
        let mut pixel = [0u8; 4];
        if ok {
            let mut bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: 1,
                    biHeight: -1,
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                },
                ..Default::default()
            };
            GetDIBits(
                hdc_mem,
                hbm,
                0,
                1,
                Some(pixel.as_mut_ptr() as *mut _),
                &mut bmi,
                DIB_RGB_COLORS,
            );
        }

        SelectObject(hdc_mem, old);
        let _ = DeleteObject(hbm);
        let _ = DeleteDC(hdc_mem);
        ReleaseDC(HWND::default(), hdc_screen);

        if ok {
            Some(Rgb {
                r: pixel[2],
                g: pixel[1],
                b: pixel[0],
            })
        } else {
            None
        }
    }
}

/// Checks the right side of the top-center BSS backpack bar for the red "full" fill color.
pub fn is_backpack_full(win: WindowRect) -> bool {
    // BSS backpack bar sits near top-center (~50% width, ~6% height).
    // Sample near the right edge of the pollen bar where red appears at ~98-100%.
    let sample_x = win.x + (win.width as f32 * 0.56) as i32;
    let sample_y = win.y + (win.height as f32 * 0.06) as i32;
    if let Some(px) = get_pixel(sample_x, sample_y) {
        // Reddish bar when full (high R, low G/B)
        px.r > 195 && px.g < 85 && px.b < 85
    } else {
        false
    }
}
