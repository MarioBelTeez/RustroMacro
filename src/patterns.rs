use crate::config::PatternKind;
use crate::input::{hold_keys, interruptible_sleep, Key};
use crate::paths::walk_ms;
use std::sync::atomic::AtomicBool;

fn walk(tiles: f32, k1: Key, k2: Option<Key>, move_speed: f32, stop_flag: &AtomicBool) -> bool {
    let ms = walk_ms(tiles, move_speed);
    match k2 {
        Some(k2) => hold_keys(&[k1, k2], ms, stop_flag),
        None => hold_keys(&[k1], ms, stop_flag),
    }
}

pub fn run_pattern(
    kind: PatternKind,
    size: u8,
    move_speed: f32,
    stop_flag: &AtomicBool,
) -> bool {
    let s = size.clamp(1, 6) as f32;
    match kind {
        PatternKind::Stationary => interruptible_sleep(500, stop_flag),
        PatternKind::Squares => run_squares(s, move_speed, stop_flag),
        PatternKind::Snake => run_snake(s, move_speed, stop_flag),
        PatternKind::CornerXSnake => run_corner_x_snake(s, move_speed, stop_flag),
        PatternKind::ELol => run_e_lol(s, move_speed, stop_flag),
    }
}

/// Ported from NatroMacro `patterns/Squares.ahk`
fn run_squares(size: f32, move_speed: f32, stop_flag: &AtomicBool) -> bool {
    for i in 1..=3 {
        let dist = 5.0 * size + i as f32;
        for k in [Key::W, Key::D, Key::S, Key::A] {
            if !walk(dist, k, None, move_speed, stop_flag) {
                return false;
            }
        }
    }
    true
}

/// Ported from NatroMacro `patterns/Snake.ahk`
fn run_snake(size: f32, move_speed: f32, stop_flag: &AtomicBool) -> bool {
    let len = 11.0 * size;
    for _ in 0..2 {
        if !walk(len, Key::D, None, move_speed, stop_flag)
            || !walk(1.0, Key::W, None, move_speed, stop_flag)
            || !walk(len, Key::A, None, move_speed, stop_flag)
            || !walk(1.0, Key::W, None, move_speed, stop_flag)
        {
            return false;
        }
    }
    for _ in 0..2 {
        if !walk(len, Key::D, None, move_speed, stop_flag)
            || !walk(1.0, Key::S, None, move_speed, stop_flag)
            || !walk(len, Key::A, None, move_speed, stop_flag)
            || !walk(1.0, Key::S, None, move_speed, stop_flag)
        {
            return false;
        }
    }
    true
}

/// Ported from NatroMacro `patterns/CornerXSnake.ahk`
fn run_corner_x_snake(size: f32, move_speed: f32, stop_flag: &AtomicBool) -> bool {
    let diag8 = ((8.0 * size).powi(2) * 2.0).sqrt();
    let diag4 = ((4.0 * size).powi(2) * 2.0).sqrt();
    let reps = 2.0;

    walk(4.0 * size, Key::D, None, move_speed, stop_flag)
        && walk(2.0 * size, Key::W, None, move_speed, stop_flag)
        && walk(8.0 * size, Key::A, None, move_speed, stop_flag)
        && walk(2.0 * size, Key::W, None, move_speed, stop_flag)
        && walk(8.0 * size, Key::D, None, move_speed, stop_flag)
        && walk(diag8, Key::A, Some(Key::S), move_speed, stop_flag)
        && walk(8.0 * size, Key::D, None, move_speed, stop_flag)
        && walk(2.0 * size, Key::W, None, move_speed, stop_flag)
        && walk(8.0 * size, Key::A, None, move_speed, stop_flag)
        && walk(6.7 * size + 10.0, Key::W, None, move_speed, stop_flag)
        && walk(6.0 + reps, Key::A, None, move_speed, stop_flag)
        && walk(3.0, Key::W, None, move_speed, stop_flag)
        && walk(2.0 + reps, Key::D, None, move_speed, stop_flag)
        && walk(5.0, Key::S, None, move_speed, stop_flag)
        && walk(8.0 * size, Key::D, None, move_speed, stop_flag)
        && walk(2.0 * size, Key::S, None, move_speed, stop_flag)
        && walk(8.0 * size, Key::A, None, move_speed, stop_flag)
        && walk(2.0 * size, Key::S, None, move_speed, stop_flag)
        && walk(8.0 * size, Key::D, None, move_speed, stop_flag)
        && walk(2.0 * size, Key::S, None, move_speed, stop_flag)
        && walk(8.0 * size, Key::A, None, move_speed, stop_flag)
        && walk(3.0 * size, Key::S, None, move_speed, stop_flag)
        && walk(8.0 * size, Key::D, None, move_speed, stop_flag)
        && walk(diag4, Key::W, Some(Key::A), move_speed, stop_flag)
}

/// Ported from NatroMacro `patterns/e_lol.ahk`
fn run_e_lol(size: f32, move_speed: f32, stop_flag: &AtomicBool) -> bool {
    let spacing = 274.0 * 9.0 / 2000.0;
    let reps = 2;

    if !walk(spacing * (reps as f32 * 2.0 + 1.0), Key::D, None, move_speed, stop_flag)
        || !walk(5.0 * size, Key::S, None, move_speed, stop_flag)
    {
        return false;
    }
    for _ in 0..reps {
        if !walk(spacing, Key::A, None, move_speed, stop_flag)
            || !walk(5.0 * size, Key::W, None, move_speed, stop_flag)
            || !walk(spacing, Key::A, None, move_speed, stop_flag)
            || !walk(1094.0 * 9.0 / 2000.0 * size, Key::S, None, move_speed, stop_flag)
        {
            return false;
        }
    }
    if !walk(spacing * (reps as f32 * 2.0 + 0.5), Key::D, None, move_speed, stop_flag)
        || !walk(5.0 * size, Key::W, None, move_speed, stop_flag)
    {
        return false;
    }
    for _ in 0..reps {
        if !walk(spacing, Key::A, None, move_speed, stop_flag)
            || !walk(1094.0 * 9.0 / 2000.0 * size, Key::S, None, move_speed, stop_flag)
            || !walk(spacing * 1.5, Key::A, None, move_speed, stop_flag)
            || !walk(5.0 * size, Key::W, None, move_speed, stop_flag)
        {
            return false;
        }
    }
    true
}
