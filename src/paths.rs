use crate::config::Field;
use crate::input::{hold_keys, interruptible_sleep, key_down, key_up, tap_key, Key};
use std::sync::atomic::AtomicBool;

/// NatroMacro's `Walk(tiles)` formula: `4000 / MoveSpeedNum * tiles` ms.
pub fn walk_ms(tiles: f32, move_speed: f32) -> u64 {
    let speed = move_speed.max(10.0);
    ((4000.0 / speed) * tiles).round().max(20.0) as u64
}

fn nm_walk(tiles: f32, k1: Key, k2: Option<Key>, move_speed: f32, stop_flag: &AtomicBool) -> bool {
    let ms = walk_ms(tiles, move_speed);
    match k2 {
        Some(k2) => hold_keys(&[k1, k2], ms, stop_flag),
        None => hold_keys(&[k1], ms, stop_flag),
    }
}

fn send_repeat(key: Key, count: u8, stop_flag: &AtomicBool) -> bool {
    for _ in 0..count {
        if !tap_key(key, 35, stop_flag) || !interruptible_sleep(45, stop_flag) {
            return false;
        }
    }
    true
}

/// Reset character to return reliably to the claimed hive slot, then zoom out.
pub fn reset_to_hive(stop_flag: &AtomicBool) -> bool {
    if !tap_key(Key::Escape, 80, stop_flag) || !interruptible_sleep(250, stop_flag) {
        return false;
    }
    if !tap_key(Key::R, 80, stop_flag) || !interruptible_sleep(250, stop_flag) {
        return false;
    }
    if !tap_key(Key::Return, 80, stop_flag) {
        return false;
    }
    if !interruptible_sleep(6000, stop_flag) {
        return false;
    }
    send_repeat(Key::O, 5, stop_flag)
}

/// NatroMacro `nm_gotoRamp()`: walk forward 5 tiles, then right `9.2 * HiveSlot - 4` tiles.
pub fn nm_goto_ramp(hive_slot: u8, move_speed: f32, stop_flag: &AtomicBool) -> bool {
    let slot = hive_slot.clamp(1, 6) as f32;
    if !nm_walk(5.0, Key::W, None, move_speed, stop_flag) {
        return false;
    }
    nm_walk(9.2 * slot - 4.0, Key::D, None, move_speed, stop_flag)
}

/// NatroMacro `nm_gotoCannon()`: hop onto the Red Cannon pad from the ramp.
pub fn nm_goto_cannon(move_speed: f32, stop_flag: &AtomicBool) -> bool {
    key_down(Key::Space);
    key_down(Key::D);
    if !interruptible_sleep(100, stop_flag) {
        key_up(Key::Space);
        key_up(Key::D);
        return false;
    }
    key_up(Key::Space);
    if !interruptible_sleep(walk_ms(2.0, move_speed), stop_flag) {
        key_up(Key::D);
        return false;
    }
    key_down(Key::W);
    let ok = interruptible_sleep(walk_ms(1.5, move_speed), stop_flag);
    key_up(Key::W);
    key_up(Key::D);
    ok
}

fn fire_cannon(stop_flag: &AtomicBool) -> bool {
    tap_key(Key::E, 100, stop_flag)
}

/// Ported directly from NatroMacro `paths/gtf-*.ahk`.
pub fn travel_to_field(field: Field, hive_slot: u8, move_speed: f32, stop_flag: &AtomicBool) -> bool {
    if !nm_goto_ramp(hive_slot, move_speed, stop_flag) {
        return false;
    }

    match field {
        Field::Sunflower => {
            // gtf-sunflower.ahk
            nm_walk(9.0, Key::S, None, move_speed, stop_flag)
                && nm_walk(6.75, Key::S, Some(Key::D), move_speed, stop_flag)
                && send_repeat(Key::Period, 2, stop_flag)
                && nm_walk(29.0, Key::D, None, move_speed, stop_flag)
        }
        Field::Dandelion => {
            // gtf-dandelion.ahk
            nm_walk(39.0, Key::S, Some(Key::A), move_speed, stop_flag)
                && nm_walk(14.0, Key::A, None, move_speed, stop_flag)
                && send_repeat(Key::Comma, 2, stop_flag)
        }
        Field::PineTree => {
            // gtf-pinetree.ahk (Cannon)
            if !nm_goto_cannon(move_speed, stop_flag) || !fire_cannon(stop_flag) {
                return false;
            }
            key_down(Key::D);
            key_down(Key::S);
            if !interruptible_sleep(925, stop_flag) {
                key_up(Key::D);
                key_up(Key::S);
                return false;
            }
            if !send_repeat(Key::Space, 2, stop_flag) || !interruptible_sleep(4500, stop_flag) {
                key_up(Key::D);
                key_up(Key::S);
                return false;
            }
            key_up(Key::S);
            if !interruptible_sleep(500, stop_flag) {
                key_up(Key::D);
                return false;
            }
            key_up(Key::D);
            tap_key(Key::Space, 60, stop_flag)
                && send_repeat(Key::Comma, 4, stop_flag)
                && interruptible_sleep(2000, stop_flag)
        }
        Field::Pumpkin => {
            // gtf-pumpkin.ahk (Cannon)
            if !nm_goto_cannon(move_speed, stop_flag) || !fire_cannon(stop_flag) {
                return false;
            }
            key_down(Key::D);
            key_down(Key::S);
            if !interruptible_sleep(950, stop_flag) {
                key_up(Key::D);
                key_up(Key::S);
                return false;
            }
            if !send_repeat(Key::Space, 2, stop_flag) || !interruptible_sleep(2700, stop_flag) {
                key_up(Key::D);
                key_up(Key::S);
                return false;
            }
            key_up(Key::D);
            if !interruptible_sleep(500, stop_flag) {
                key_up(Key::S);
                return false;
            }
            key_up(Key::S);
            if !interruptible_sleep(600, stop_flag) {
                return false;
            }
            tap_key(Key::Space, 60, stop_flag)
                && send_repeat(Key::Comma, 4, stop_flag)
                && interruptible_sleep(1500, stop_flag)
        }
        Field::Clover => {
            // gtf-clover.ahk (Cannon)
            if !nm_goto_cannon(move_speed, stop_flag) {
                return false;
            }
            if !send_repeat(Key::Comma, 1, stop_flag) {
                return false;
            }
            key_down(Key::E);
            key_down(Key::W);
            if !interruptible_sleep(100, stop_flag) {
                key_up(Key::E);
                key_up(Key::W);
                return false;
            }
            key_up(Key::E);
            if !interruptible_sleep(1100, stop_flag) {
                key_up(Key::W);
                return false;
            }
            let ok = send_repeat(Key::Space, 2, stop_flag);
            key_up(Key::W);
            ok && interruptible_sleep(3767, stop_flag)
                && tap_key(Key::Space, 60, stop_flag)
                && send_repeat(Key::Period, 1, stop_flag)
                && interruptible_sleep(1500, stop_flag)
        }
        Field::BlueFlower => {
            // gtf-blueflower.ahk (Cannon)
            if !nm_goto_cannon(move_speed, stop_flag) || !fire_cannon(stop_flag) {
                return false;
            }
            key_down(Key::A);
            if !interruptible_sleep(675, stop_flag) {
                key_up(Key::A);
                return false;
            }
            if !send_repeat(Key::Space, 2, stop_flag) || !interruptible_sleep(2000, stop_flag) {
                key_up(Key::A);
                return false;
            }
            key_up(Key::A);
            interruptible_sleep(1250, stop_flag)
                && tap_key(Key::Space, 60, stop_flag)
                && send_repeat(Key::Comma, 2, stop_flag)
                && interruptible_sleep(1000, stop_flag)
        }
        Field::Strawberry => {
            // gtf-strawberry.ahk (Cannon)
            if !nm_goto_cannon(move_speed, stop_flag) || !fire_cannon(stop_flag) {
                return false;
            }
            key_down(Key::D);
            key_down(Key::S);
            if !interruptible_sleep(700, stop_flag) {
                key_up(Key::D);
                key_up(Key::S);
                return false;
            }
            let ok = send_repeat(Key::Space, 2, stop_flag) && interruptible_sleep(1700, stop_flag);
            key_up(Key::D);
            key_up(Key::S);
            ok && tap_key(Key::Space, 60, stop_flag)
                && send_repeat(Key::Period, 2, stop_flag)
                && interruptible_sleep(2000, stop_flag)
        }
        Field::Pepper => {
            // gtf-pepper.ahk (Walk from ramp)
            key_down(Key::Space);
            key_down(Key::D);
            if !interruptible_sleep(100, stop_flag) {
                key_up(Key::Space);
                key_up(Key::D);
                return false;
            }
            key_up(Key::Space);
            if !interruptible_sleep(walk_ms(2.0, move_speed), stop_flag) {
                key_up(Key::D);
                return false;
            }
            key_down(Key::W);
            if !interruptible_sleep(walk_ms(1.8, move_speed), stop_flag) {
                key_up(Key::W);
                key_up(Key::D);
                return false;
            }
            key_up(Key::W);
            if !interruptible_sleep(walk_ms(30.0, move_speed), stop_flag) {
                key_up(Key::D);
                return false;
            }
            key_up(Key::D);
            if !tap_key(Key::Space, 300, stop_flag) {
                return false;
            }
            if !nm_walk(4.0, Key::D, None, move_speed, stop_flag)
                || !nm_walk(5.0, Key::W, None, move_speed, stop_flag)
                || !nm_walk(3.0, Key::D, None, move_speed, stop_flag)
                || !tap_key(Key::Space, 300, stop_flag)
                || !nm_walk(6.0, Key::W, None, move_speed, stop_flag)
                || !nm_walk(2.0, Key::A, Some(Key::W), move_speed, stop_flag)
                || !nm_walk(8.0, Key::W, None, move_speed, stop_flag)
            {
                return false;
            }
            key_down(Key::W);
            key_down(Key::D);
            if !interruptible_sleep(walk_ms(11.0, move_speed), stop_flag) {
                key_up(Key::W);
                key_up(Key::D);
                return false;
            }
            key_up(Key::D);
            if !tap_key(Key::Space, 200, stop_flag)
                || !interruptible_sleep(1100, stop_flag)
                || !tap_key(Key::Space, 200, stop_flag)
                || !interruptible_sleep(walk_ms(18.0, move_speed), stop_flag)
                || !tap_key(Key::Space, 200, stop_flag)
                || !interruptible_sleep(walk_ms(20.0, move_speed), stop_flag)
            {
                key_up(Key::W);
                return false;
            }
            key_down(Key::D);
            if !interruptible_sleep(walk_ms(9.0, move_speed), stop_flag)
                || !tap_key(Key::Space, 300, stop_flag)
                || !interruptible_sleep(walk_ms(1.0, move_speed), stop_flag)
            {
                key_up(Key::W);
                key_up(Key::D);
                return false;
            }
            key_up(Key::W);
            if !interruptible_sleep(walk_ms(33.0, move_speed), stop_flag)
                || !tap_key(Key::Space, 300, stop_flag)
                || !interruptible_sleep(walk_ms(4.0, move_speed), stop_flag)
            {
                key_up(Key::D);
                return false;
            }
            key_up(Key::D);
            send_repeat(Key::Period, 2, stop_flag)
                && nm_walk(9.0, Key::W, None, move_speed, stop_flag)
                && nm_walk(1.5, Key::D, None, move_speed, stop_flag)
        }
    }
}

/// Reset to hive, press E to convert pollen, and wait up to 45s.
pub fn convert_at_hive(stop_flag: &AtomicBool) -> bool {
    if !reset_to_hive(stop_flag) {
        return false;
    }
    if !tap_key(Key::E, 100, stop_flag) {
        return false;
    }
    interruptible_sleep(25_000, stop_flag)
}
