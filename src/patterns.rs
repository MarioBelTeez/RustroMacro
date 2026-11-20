use crate::config::PatternKind;
use crate::input::{hold_keys, Key};
use std::sync::atomic::AtomicBool;

/// Scale walk duration (ms) relative to NatroMacro's baseline 28 movespeed.
fn scaled_ms(base_ms: u64, move_speed: f32) -> u64 {
    let speed = move_speed.max(10.0);
    ((base_ms as f32) * (28.0 / speed)).round().max(40.0) as u64
}

pub fn run_pattern(
    kind: PatternKind,
    size: u8,
    move_speed: f32,
    stop_flag: &AtomicBool,
) -> bool {
    let size_mult = (size.clamp(1, 6) as u64) * 180;
    match kind {
        PatternKind::Stationary => crate::input::interruptible_sleep(500, stop_flag),
        PatternKind::Squares => run_squares(size_mult, move_speed, stop_flag),
        PatternKind::Snake => run_snake(size_mult, move_speed, stop_flag),
        PatternKind::CornerXSnake => run_corner_x_snake(size_mult, move_speed, stop_flag),
    }
}

fn run_squares(leg_ms: u64, move_speed: f32, stop_flag: &AtomicBool) -> bool {
    let t = scaled_ms(leg_ms, move_speed);
    for step in [Key::W, Key::D, Key::S, Key::A] {
        if !hold_keys(&[step], t, stop_flag) {
            return false;
        }
    }
    true
}

fn run_snake(leg_ms: u64, move_speed: f32, stop_flag: &AtomicBool) -> bool {
    let long_t = scaled_ms(leg_ms, move_speed);
    let step_t = scaled_ms(140, move_speed);

    for _ in 0..3 {
        if !hold_keys(&[Key::W], long_t, stop_flag) {
            return false;
        }
        if !hold_keys(&[Key::D], step_t, stop_flag) {
            return false;
        }
        if !hold_keys(&[Key::S], long_t, stop_flag) {
            return false;
        }
        if !hold_keys(&[Key::D], step_t, stop_flag) {
            return false;
        }
    }
    for _ in 0..3 {
        if !hold_keys(&[Key::W], long_t, stop_flag) {
            return false;
        }
        if !hold_keys(&[Key::A], step_t, stop_flag) {
            return false;
        }
        if !hold_keys(&[Key::S], long_t, stop_flag) {
            return false;
        }
        if !hold_keys(&[Key::A], step_t, stop_flag) {
            return false;
        }
    }
    true
}

fn run_corner_x_snake(leg_ms: u64, move_speed: f32, stop_flag: &AtomicBool) -> bool {
    let long_t = scaled_ms(leg_ms, move_speed);
    let step_t = scaled_ms(160, move_speed);

    // Drift into corner briefly, then snake across the field
    if !hold_keys(&[Key::S, Key::A], scaled_ms(320, move_speed), stop_flag) {
        return false;
    }
    for _ in 0..2 {
        if !hold_keys(&[Key::W], long_t, stop_flag) {
            return false;
        }
        if !hold_keys(&[Key::D], step_t, stop_flag) {
            return false;
        }
        if !hold_keys(&[Key::S], long_t, stop_flag) {
            return false;
        }
        if !hold_keys(&[Key::D], step_t, stop_flag) {
            return false;
        }
    }
    if !hold_keys(&[Key::A], scaled_ms(320, move_speed), stop_flag) {
        return false;
    }
    true
}
