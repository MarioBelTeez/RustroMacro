use crate::capture::{find_roblox_window, get_client_rect_screen};
use crate::input::{key_down, key_up, Key};
use std::mem::size_of;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits,
    ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, SRCCOPY,
};

const STRIP_H: i32 = 30;
const STRIP_Y_OFFSET: i32 = 48;

#[derive(Debug, Clone, Copy)]
pub struct SpeedProfile {
    pub base_movespeed: f32,
    pub hasty_guard: f32,
    pub gifted_hasty: f32,
}

impl SpeedProfile {
    /// Port of NatroMacro's `base_movespeed` / `hasty_guard` / `gifted_hasty` decomposition.
    pub fn from_configured(movespeed: f32) -> Self {
        let ms_milli = (movespeed * 1000.0).round() as i32;
        let both = (ms_milli % 1265) == 0 || (((movespeed + 0.005) * 1000.0).round() as i32 % 1265) == 0;
        let hasty_guard = both || (ms_milli % 1100) == 0;
        let gifted_hasty = both || (ms_milli % 1150) == 0;
        let mult = if both {
            1.265
        } else if hasty_guard {
            1.1
        } else if gifted_hasty {
            1.15
        } else {
            1.0
        };
        let base_movespeed = (movespeed / mult).round().max(10.0);
        Self {
            base_movespeed,
            hasty_guard: if hasty_guard { 1.1 } else { 1.0 },
            gifted_hasty: if gifted_hasty { 1.15 } else { 1.0 },
        }
    }
}

fn capture_buff_strip() -> Option<(i32, Vec<u8>)> {
    let hwnd = find_roblox_window()?;
    let rect = get_client_rect_screen(hwnd)?;
    if rect.width <= 0 || rect.height < STRIP_Y_OFFSET + STRIP_H {
        return None;
    }

    unsafe {
        let hdc_screen = GetDC(HWND::default());
        if hdc_screen.is_invalid() {
            return None;
        }
        let hdc_mem = CreateCompatibleDC(hdc_screen);
        let hbm = CreateCompatibleBitmap(hdc_screen, rect.width, STRIP_H);
        let old = SelectObject(hdc_mem, hbm);

        let ok = BitBlt(
            hdc_mem,
            0,
            0,
            rect.width,
            STRIP_H,
            hdc_screen,
            rect.x,
            rect.y + STRIP_Y_OFFSET,
            SRCCOPY,
        )
        .is_ok();

        let mut buf = vec![0u8; (rect.width * STRIP_H * 4) as usize];
        if ok {
            let mut bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: rect.width,
                    biHeight: -STRIP_H,
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
                STRIP_H as u32,
                Some(buf.as_mut_ptr() as *mut _),
                &mut bmi,
                DIB_RGB_COLORS,
            );
        }

        SelectObject(hdc_mem, old);
        let _ = DeleteObject(hbm);
        let _ = DeleteDC(hdc_mem);
        ReleaseDC(HWND::default(), hdc_screen);

        if ok {
            Some((rect.width, buf))
        } else {
            None
        }
    }
}

#[inline]
fn px_at(buf: &[u8], w: i32, x: i32, y: i32) -> (u8, u8, u8) {
    let idx = ((y * w + x) * 4) as usize;
    // BGRA -> RGB
    (buf[idx + 2], buf[idx + 1], buf[idx])
}

#[inline]
fn color_near(c: (u8, u8, u8), target: (u8, u8, u8), tol: u8) -> bool {
    c.0.abs_diff(target.0) <= tol
        && c.1.abs_diff(target.1) <= tol
        && c.2.abs_diff(target.2) <= tol
}

fn has_melody_at(buf: &[u8], w: i32, x_start: i32, x_end: i32, y_end: i32) -> bool {
    let x0 = x_start.max(0);
    let x1 = x_end.min(w - 3);
    let y1 = y_end.min(STRIP_H - 2);
    for y in 0..=y1 {
        for x in x0..=x1 {
            let mut ok = true;
            for dy in 0..2 {
                for dx in 0..3 {
                    if !color_near(px_at(buf, w, x + dx, y + dy), (0x24, 0x24, 0x24), 12) {
                        ok = false;
                        break;
                    }
                }
                if !ok {
                    break;
                }
            }
            if ok {
                return true;
            }
        }
    }
    false
}

const DIGIT_0: &[&str] = &[
    "..####..", ".#....#.", ".......#", "#......#", "#......#", "#......#", "#......#", ".......#",
    ".#....#.", "..####..",
];
const DIGIT_2: &[&str] = &[
    ".###", "#...", "....", "....", "....", "...#", "..#.", ".#..", "#...", "####", "####",
];
const DIGIT_3: &[&str] = &[
    ".###", "....", "....", "....", "...#", "....", "....", "....", "#...", ".###",
];
const DIGIT_4: &[&str] = &["...#", "....", "..#.", ".#..", "##..", "####"];
const DIGIT_5: &[&str] = &[
    "####", "####", "#...", "#...", "####", "....", "....", "....", "....", "....", "####",
];
const DIGIT_6: &[&str] = &[
    "...#", "..#.", ".#..", ".#..", ".###", "##..", "#...", ".#..", "..##",
];
const DIGIT_7: &[&str] = &[
    "####", "####", "....", "....", "....", "....", "....", "....", "....", "...#", "...#", "...#",
];
const DIGIT_8: &[&str] = &[
    "..##", ".#..", ".#..", ".###", "..##", ".#..", "....", "#...", ".#..", "..##",
];
const DIGIT_9: &[&str] = &[
    "..##", ".#..", "#...", ".#..", "..##", "....", "....", "....", "....", "...#",
];

fn match_digit_template(
    buf: &[u8],
    w: i32,
    tpl: &[&str],
    rx0: i32,
    ry0: i32,
    rx1: i32,
    ry1: i32,
) -> bool {
    let th = tpl.len() as i32;
    let tw = tpl[0].len() as i32;
    let x_start = rx0.max(0);
    let y_start = ry0.max(0);
    let x_end = (rx1 - tw).min(w - tw);
    let y_end = (ry1 - th).min(STRIP_H - th);
    if x_start > x_end || y_start > y_end {
        return false;
    }

    for y in y_start..=y_end {
        for x in x_start..=x_end {
            let mut ok = true;
            'check: for (dy, row) in tpl.iter().enumerate() {
                for (dx, ch) in row.bytes().enumerate() {
                    if ch == b'#' {
                        let c = px_at(buf, w, x + dx as i32, y + dy as i32);
                        // White digit pixel (~#F0F0F0..#FFFFFF)
                        if c.0 < 210 || c.1 < 210 || c.2 < 210 {
                            ok = false;
                            break 'check;
                        }
                    }
                }
            }
            if ok {
                return true;
            }
        }
    }
    false
}

fn detect_haste_stacks(buf: &[u8], w: i32, x1: i32, y1: i32) -> u32 {
    let rx0 = x1 + 2 * y1 - 44;
    let ry0 = (y1 - 18).max(0);
    let rx1 = x1 + 2 * y1 - 14;
    let ry1 = y1 - 1;

    let checks: [(u32, &[&str]); 9] = [
        (9, DIGIT_9),
        (8, DIGIT_8),
        (7, DIGIT_7),
        (6, DIGIT_6),
        (5, DIGIT_5),
        (4, DIGIT_4),
        (3, DIGIT_3),
        (2, DIGIT_2),
        (10, DIGIT_0),
    ];
    for (stacks, tpl) in checks {
        if match_digit_template(buf, w, tpl, rx0, ry0, rx1, ry1) {
            return stacks;
        }
    }
    1
}

/// Detect active speed buffs (Haste 1-10x, Coconut Haste, Haste+) from the Roblox buff bar.
/// Returns `(effective_speed, haste_stacks, haste_plus, coconut)`.
pub fn detect_live_movespeed_info(profile: SpeedProfile) -> (f32, u32, bool, bool) {
    let base_total = profile.base_movespeed * profile.hasty_guard * profile.gifted_hasty;
    let Some((w, buf)) = capture_buff_strip() else {
        return (base_total, 0, false, false);
    };

    // 1. Scan for Haste / Coconut Haste (10px wide #F0F0F0 strip at y >= 14, excluding Melody #242424)
    let mut haste_count = 0u32;
    let mut first_haste = None;
    let mut x = 0i32;
    for _ in 0..3 {
        let mut found = None;
        'search: for y in 14..STRIP_H {
            for sx in x..=(w - 10) {
                let mut run = true;
                for dx in 0..10 {
                    if !color_near(px_at(&buf, w, sx + dx, y), (0xF0, 0xF0, 0xF0), 14) {
                        run = false;
                        break;
                    }
                }
                if run {
                    found = Some((sx, y));
                    break 'search;
                }
            }
        }
        let Some((fx, fy)) = found else {
            break;
        };
        let melody_x_end = fx + (2 * fy - 24).max(16);
        if !has_melody_at(&buf, w, fx + 2, melody_x_end, fy) {
            haste_count += 1;
            if first_haste.is_none() {
                first_haste = Some((fx, fy));
            }
        }
        x = fx + (2 * fy - 14).max(12);
        if x >= w - 10 {
            break;
        }
    }

    let coconut = haste_count >= 2;
    let haste_stacks = if let Some((x1, y1)) = first_haste {
        detect_haste_stacks(&buf, w, x1, y1)
    } else {
        0
    };

    // 2. Scan for Haste+ (20px wide #EDDB4C strip at y = 25..27)
    let mut haste_plus = false;
    'hp: for y in 25..27.min(STRIP_H) {
        for sx in 0..=(w - 20) {
            let mut run = true;
            for dx in 0..20 {
                if !color_near(px_at(&buf, w, sx + dx, y), (0xED, 0xDB, 0x4C), 16) {
                    run = false;
                    break;
                }
            }
            if run {
                haste_plus = true;
                break 'hp;
            }
        }
    }

    let coconut_bonus = if coconut { 10.0 } else { 0.0 };
    let haste_mult = 1.0 + (haste_stacks as f32) * 0.1;
    let hp_mult = if haste_plus { 2.0 } else { 1.0 };

    let speed = (profile.base_movespeed + coconut_bonus)
        * profile.hasty_guard
        * profile.gifted_hasty
        * haste_mult
        * hp_mult;
    (speed, haste_stacks, haste_plus, coconut)
}

pub fn detect_live_movespeed(profile: SpeedProfile) -> f32 {
    detect_live_movespeed_info(profile).0
}

/// NatroMacro `Walk(n)` with live movespeed integration (4 studs per tile).
pub fn walk_corrected(
    tiles: f32,
    keys: &[Key],
    move_speed: f32,
    stop_flag: &AtomicBool,
) -> bool {
    let profile = SpeedProfile::from_configured(move_speed);
    let target_studs = tiles * 4.0;
    let mut covered_studs = 0.0f32;

    for &k in keys {
        key_down(k);
    }

    let mut t_prev = Instant::now();
    let mut v_prev = detect_live_movespeed(profile);

    // Initial 125ms step matching Natro's `d := freq // 8`
    let step_dur = Duration::from_millis(25);
    while covered_studs < target_studs {
        if stop_flag.load(Ordering::Relaxed) {
            for &k in keys {
                key_up(k);
            }
            return false;
        }
        thread::sleep(step_dur);
        let t_now = Instant::now();
        let dt = (t_now - t_prev).as_secs_f32();
        let v_now = detect_live_movespeed(profile);
        covered_studs += ((v_prev + v_now) * 0.5) * dt;
        t_prev = t_now;
        v_prev = v_now;
    }

    for &k in keys {
        key_up(k);
    }
    !stop_flag.load(Ordering::Relaxed)
}
