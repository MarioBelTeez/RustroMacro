use crate::config::Field;
use crate::input::{hold_keys, interruptible_sleep, tap_key, Key};
use std::sync::atomic::AtomicBool;

fn scaled_ms(base_ms: u64, move_speed: f32) -> u64 {
    let speed = move_speed.max(10.0);
    ((base_ms as f32) * (28.0 / speed)).round().max(40.0) as u64
}

/// Reset character to return reliably to the claimed hive slot.
pub fn reset_to_hive(stop_flag: &AtomicBool) -> bool {
    if !tap_key(Key::Escape, 80, stop_flag) {
        return false;
    }
    if !interruptible_sleep(180, stop_flag) {
        return false;
    }
    if !tap_key(Key::R, 80, stop_flag) {
        return false;
    }
    if !interruptible_sleep(180, stop_flag) {
        return false;
    }
    if !tap_key(Key::Return, 80, stop_flag) {
        return false;
    }
    interruptible_sleep(6200, stop_flag)
}

/// Walk from hive slot (1..=6, left-to-right facing hives) toward the Red Cannon and launch.
pub fn travel_hive_to_cannon(hive_slot: u8, move_speed: f32, stop_flag: &AtomicBool) -> bool {
    let slot = hive_slot.clamp(1, 6);
    // Step away from hive pad
    if !hold_keys(&[Key::W], scaled_ms(750, move_speed), stop_flag) {
        return false;
    }
    // Strafe right toward Red Cannon ramp; higher hive slots sit closer
    let strafe_ms = match slot {
        1 => 3100,
        2 => 2600,
        3 => 2100,
        4 => 1600,
        5 => 1100,
        _ => 650,
    };
    if !hold_keys(&[Key::D], scaled_ms(strafe_ms, move_speed), stop_flag) {
        return false;
    }
    // Hop onto Red Cannon and fire
    if !hold_keys(&[Key::W, Key::Space], scaled_ms(500, move_speed), stop_flag) {
        return false;
    }
    if !tap_key(Key::E, 100, stop_flag) {
        return false;
    }
    true
}

/// Navigate from Red Cannon launch to the selected starter field.
pub fn travel_to_field(field: Field, hive_slot: u8, move_speed: f32, stop_flag: &AtomicBool) -> bool {
    match field {
        Field::Sunflower | Field::Dandelion | Field::Clover | Field::BlueFlower => {
            // Walk directly from hive area without cannon for nearby starter fields
            if !hold_keys(&[Key::W], scaled_ms(1400, move_speed), stop_flag) {
                return false;
            }
            let dir = match field {
                Field::Sunflower => Key::A,
                Field::Dandelion => Key::W,
                Field::Clover => Key::D,
                Field::BlueFlower => Key::D,
                _ => Key::W,
            };
            let dur = match field {
                Field::Sunflower => 1800,
                Field::Dandelion => 1200,
                Field::Clover => 2100,
                Field::BlueFlower => 1200,
                _ => 1000,
            };
            hold_keys(&[dir], scaled_ms(dur, move_speed), stop_flag)
        }
        Field::PineTree | Field::Pumpkin | Field::Pepper | Field::Strawberry => {
            if !travel_hive_to_cannon(hive_slot, move_speed, stop_flag) {
                return false;
            }
            // Wait for cannon arc & glide timing
            match field {
                Field::PineTree => {
                    if !interruptible_sleep(1150, stop_flag) {
                        return false;
                    }
                    if !tap_key(Key::Space, 90, stop_flag) {
                        return false;
                    }
                    if !hold_keys(&[Key::W, Key::A], scaled_ms(2200, move_speed), stop_flag) {
                        return false;
                    }
                    tap_key(Key::Space, 90, stop_flag)
                }
                Field::Pumpkin => {
                    if !interruptible_sleep(1050, stop_flag) {
                        return false;
                    }
                    if !tap_key(Key::Space, 90, stop_flag) {
                        return false;
                    }
                    if !hold_keys(&[Key::W], scaled_ms(1900, move_speed), stop_flag) {
                        return false;
                    }
                    tap_key(Key::Space, 90, stop_flag)
                }
                Field::Pepper => {
                    if !interruptible_sleep(1300, stop_flag) {
                        return false;
                    }
                    if !tap_key(Key::Space, 90, stop_flag) {
                        return false;
                    }
                    if !hold_keys(&[Key::D, Key::W], scaled_ms(2400, move_speed), stop_flag) {
                        return false;
                    }
                    tap_key(Key::Space, 90, stop_flag)
                }
                Field::Strawberry => {
                    if !interruptible_sleep(750, stop_flag) {
                        return false;
                    }
                    if !tap_key(Key::Space, 90, stop_flag) {
                        return false;
                    }
                    if !hold_keys(&[Key::A], scaled_ms(1100, move_speed), stop_flag) {
                        return false;
                    }
                    tap_key(Key::Space, 90, stop_flag)
                }
                _ => true,
            }
        }
    }
}

/// Press E at hive to convert pollen, then wait up to 45s.
pub fn convert_at_hive(stop_flag: &AtomicBool) -> bool {
    if !reset_to_hive(stop_flag) {
        return false;
    }
    if !tap_key(Key::E, 100, stop_flag) {
        return false;
    }
    interruptible_sleep(25_000, stop_flag)
}
