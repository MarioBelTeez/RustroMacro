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

/// Port of NatroMacro's `nm_backpackPercent()` at (windowWidth/2 + 62, 6).
pub fn backpack_percent(win: WindowRect) -> u8 {
    let sample_x = win.x + (win.width / 2) + 62;
    let sample_y = win.y + 6;
    let Some(px) = get_pixel(sample_x, sample_y) else {
        return 0;
    };
    let gb = ((px.g as u16) << 8) | (px.b as u16);

    if px.r <= 0x69 {
        if px.r <= 0x4B {
            if px.r <= 0x42 {
                if px.r > 0x41 && gb <= 0xFF80 && gb > 0xFC85 { 5 } else { 0 }
            } else if px.r <= 0x47 {
                if px.r <= 0x44 && gb <= 0xFE85 && gb > 0xF984 {
                    10
                } else if px.r > 0x44 && gb <= 0xFB84 && gb > 0xF582 {
                    15
                } else {
                    0
                }
            } else if gb <= 0xF782 && gb > 0xF080 {
                20
            } else {
                0
            }
        } else if px.r <= 0x5B {
            if px.r <= 0x4F && gb <= 0xF280 && gb > 0xEA7D {
                25
            } else if px.r <= 0x55 && gb <= 0xEC7D && gb > 0xE37A {
                30
            } else if px.r > 0x55 && gb <= 0xE57A && gb > 0xDA76 {
                35
            } else {
                0
            }
        } else if px.r <= 0x62 && gb <= 0xDC76 && gb > 0xD072 {
            40
        } else if px.r > 0x62 && gb <= 0xD272 && gb > 0xC66D {
            45
        } else {
            0
        }
    } else if px.r <= 0x9C {
        if px.r <= 0x85 {
            if px.r <= 0x7B {
                if px.r <= 0x72 && gb <= 0xC86D && gb > 0xBA68 {
                    50
                } else if px.r > 0x72 && gb <= 0xBC68 && gb > 0xAD62 {
                    55
                } else {
                    0
                }
            } else if gb <= 0xAF62 && gb > 0x9E5C {
                60
            } else {
                0
            }
        } else if px.r <= 0x90 && gb <= 0xA05C && gb > 0x8F55 {
            65
        } else if px.r > 0x90 && gb <= 0x9155 && gb > 0x7E4E {
            70
        } else {
            0
        }
    } else if px.r <= 0xC4 {
        if px.r <= 0xA9 && gb <= 0x804E && gb > 0x6C46 {
            75
        } else if px.r <= 0xB6 && gb <= 0x6E46 && gb > 0x5A3F {
            80
        } else if px.r > 0xB6 && gb <= 0x5D3F && gb > 0x4637 {
            85
        } else {
            0
        }
    } else if px.r <= 0xD3 && gb <= 0x4A37 && gb > 0x322E {
        90
    } else if (px.r == 0xF7 && px.g == 0x00 && px.b == 0x17)
        || (px.r >= 0xE0 && gb <= 0x2427 && gb > 0x1000)
    {
        100
    } else if gb <= 0x342E {
        95
    } else {
        0
    }
}

pub fn is_backpack_full(win: WindowRect) -> bool {
    backpack_percent(win) >= 95
}
