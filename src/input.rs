use std::mem::size_of;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, MapVirtualKeyW, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE,
    KEYBDINPUT, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, MAPVK_VK_TO_VSC,
    MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MOVE, MOUSEINPUT, VIRTUAL_KEY, VK_A,
    VK_D, VK_E, VK_ESCAPE, VK_F1, VK_F2, VK_LEFT, VK_OEM_COMMA, VK_OEM_PERIOD, VK_R, VK_RETURN,
    VK_RIGHT, VK_S, VK_SPACE, VK_W, VK_O, VK_I,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    W,
    A,
    S,
    D,
    E,
    R,
    O,
    I,
    Space,
    Escape,
    Return,
    Comma,
    Period,
    Left,
    Right,
}

impl Key {
    fn vk(self) -> VIRTUAL_KEY {
        match self {
            Key::W => VK_W,
            Key::A => VK_A,
            Key::S => VK_S,
            Key::D => VK_D,
            Key::E => VK_E,
            Key::R => VK_R,
            Key::O => VK_O,
            Key::I => VK_I,
            Key::Space => VK_SPACE,
            Key::Escape => VK_ESCAPE,
            Key::Return => VK_RETURN,
            Key::Comma => VK_OEM_COMMA,
            Key::Period => VK_OEM_PERIOD,
            Key::Left => VK_LEFT,
            Key::Right => VK_RIGHT,
        }
    }

    fn is_extended(self) -> bool {
        matches!(self, Key::Left | Key::Right)
    }
}

fn send_key_event(key: Key, up: bool) {
    let vk = key.vk();
    let scan = unsafe { MapVirtualKeyW(vk.0 as u32, MAPVK_VK_TO_VSC) } as u16;
    let mut flags = KEYEVENTF_SCANCODE;
    if key.is_extended() {
        flags |= KEYEVENTF_EXTENDEDKEY;
    }
    if up {
        flags |= KEYEVENTF_KEYUP;
    }

    let input = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: scan,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    unsafe {
        SendInput(&[input], size_of::<INPUT>() as i32);
    }
}

pub fn key_down(key: Key) {
    send_key_event(key, false);
}

pub fn key_up(key: Key) {
    send_key_event(key, true);
}

pub fn release_all_movement() {
    for k in [Key::W, Key::A, Key::S, Key::D, Key::Space] {
        key_up(k);
    }
}

/// Sleep in small chunks so Stop/Pause interrupts respond immediately.
pub fn interruptible_sleep(ms: u64, stop_flag: &AtomicBool) -> bool {
    let step = 20;
    let mut elapsed = 0;
    while elapsed < ms {
        if stop_flag.load(Ordering::Relaxed) {
            return false;
        }
        let chunk = (ms - elapsed).min(step);
        thread::sleep(Duration::from_millis(chunk));
        elapsed += chunk;
    }
    !stop_flag.load(Ordering::Relaxed)
}

pub fn tap_key(key: Key, hold_ms: u64, stop_flag: &AtomicBool) -> bool {
    key_down(key);
    let ok = interruptible_sleep(hold_ms, stop_flag);
    key_up(key);
    ok
}

pub fn hold_keys(keys: &[Key], hold_ms: u64, stop_flag: &AtomicBool) -> bool {
    for &k in keys {
        key_down(k);
    }
    let ok = interruptible_sleep(hold_ms, stop_flag);
    for &k in keys {
        key_up(k);
    }
    ok
}

pub fn set_left_mouse(down: bool) {
    let flags = if down {
        MOUSEEVENTF_LEFTDOWN
    } else {
        MOUSEEVENTF_LEFTUP
    };
    let input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    unsafe {
        SendInput(&[input], size_of::<INPUT>() as i32);
    }
}

pub fn move_mouse_rel(dx: i32, dy: i32) {
    let input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx,
                dy,
                mouseData: 0,
                dwFlags: MOUSEEVENTF_MOVE,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    unsafe {
        SendInput(&[input], size_of::<INPUT>() as i32);
    }
}

pub fn is_f1_pressed() -> bool {
    unsafe { (GetAsyncKeyState(VK_F1.0 as i32) as u16 & 0x8000) != 0 }
}

pub fn is_f2_pressed() -> bool {
    unsafe { (GetAsyncKeyState(VK_F2.0 as i32) as u16 & 0x8000) != 0 }
}
