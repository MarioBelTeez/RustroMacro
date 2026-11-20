use crate::capture::{focus_roblox, find_roblox_window, get_client_rect_screen, is_backpack_full};
use crate::config::MacroConfig;
use crate::input::{
    interruptible_sleep, is_f1_pressed, is_f2_pressed, release_all_movement, set_left_mouse,
};
use crate::paths::{convert_at_hive, reset_to_hive, travel_to_field};
use crate::patterns::run_pattern;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacroStatus {
    Idle,
    Running,
    Paused,
    Converting,
}

impl MacroStatus {
    pub fn label(self) -> &'static str {
        match self {
            MacroStatus::Idle => "IDLE",
            MacroStatus::Running => "GATHERING",
            MacroStatus::Paused => "PAUSED",
            MacroStatus::Converting => "CONVERTING",
        }
    }
}

pub struct SharedState {
    pub status: MacroStatus,
    pub logs: VecDeque<String>,
    pub cycles_completed: u32,
    pub roblox_detected: bool,
    pub live_speed: f32,
    pub live_haste: u32,
}

impl SharedState {
    pub fn new() -> Self {
        let mut logs = VecDeque::with_capacity(120);
        logs.push_back("RustroMacro ready. Press F1 to Start/Pause, F2 to Stop.".to_string());
        Self {
            status: MacroStatus::Idle,
            logs,
            cycles_completed: 0,
            roblox_detected: false,
            live_speed: 28.0,
            live_haste: 0,
        }
    }

    pub fn log(&mut self, msg: impl Into<String>) {
        if self.logs.len() >= 100 {
            self.logs.pop_front();
        }
        self.logs.push_back(msg.into());
    }
}

pub struct MacroController {
    pub shared: Arc<Mutex<SharedState>>,
    stop_flag: Arc<AtomicBool>,
    paused_flag: Arc<AtomicBool>,
    config: Arc<Mutex<MacroConfig>>,
}

impl MacroController {
    pub fn new(initial_config: MacroConfig) -> Self {
        let shared = Arc::new(Mutex::new(SharedState::new()));
        let stop_flag = Arc::new(AtomicBool::new(true));
        let paused_flag = Arc::new(AtomicBool::new(false));
        let config = Arc::new(Mutex::new(initial_config));

        let ctrl = Self {
            shared: Arc::clone(&shared),
            stop_flag: Arc::clone(&stop_flag),
            paused_flag: Arc::clone(&paused_flag),
            config: Arc::clone(&config),
        };

        // Background hotkey + Roblox window monitor thread
        let hk_shared = Arc::clone(&shared);
        let hk_stop = Arc::clone(&stop_flag);
        let hk_paused = Arc::clone(&paused_flag);
        let hk_config = Arc::clone(&config);

        thread::spawn(move || {
            let mut prev_f1 = false;
            let mut prev_f2 = false;
            let mut tick = 0u32;

            loop {
                let f1 = is_f1_pressed();
                let f2 = is_f2_pressed();

                if f2 && !prev_f2 {
                    stop_inner(&hk_shared, &hk_stop, &hk_paused);
                } else if f1 && !prev_f1 {
                    let is_idle = hk_stop.load(Ordering::Relaxed);
                    if is_idle {
                        start_inner(&hk_shared, &hk_stop, &hk_paused, &hk_config);
                    } else {
                        let now_paused = !hk_paused.load(Ordering::Relaxed);
                        hk_paused.store(now_paused, Ordering::Relaxed);
                        if let Ok(mut s) = hk_shared.lock() {
                            s.status = if now_paused {
                                MacroStatus::Paused
                            } else {
                                MacroStatus::Running
                            };
                            s.log(if now_paused {
                                "Macro paused (F1)."
                            } else {
                                "Macro resumed (F1)."
                            });
                        }
                        if now_paused {
                            release_all_movement();
                            set_left_mouse(false);
                        }
                    }
                }

                prev_f1 = f1;
                prev_f2 = f2;

                if tick % 10 == 0 {
                    let detected = find_roblox_window().is_some();
                    let ms_cfg = hk_config.lock().map(|c| c.move_speed).unwrap_or(28.0);
                    let (spd, haste, _, _) = crate::walk::detect_live_movespeed_info(
                        crate::walk::SpeedProfile::from_configured(ms_cfg),
                    );
                    if let Ok(mut s) = hk_shared.lock() {
                        s.roblox_detected = detected;
                        s.live_speed = spd;
                        s.live_haste = haste;
                    }
                }
                tick = tick.wrapping_add(1);
                thread::sleep(Duration::from_millis(50));
            }
        });

        ctrl
    }

    pub fn update_config(&self, cfg: MacroConfig) {
        if let Ok(mut guard) = self.config.lock() {
            *guard = cfg;
        }
    }

    pub fn start_or_resume(&self) {
        if self.stop_flag.load(Ordering::Relaxed) {
            start_inner(&self.shared, &self.stop_flag, &self.paused_flag, &self.config);
        } else {
            self.paused_flag.store(false, Ordering::Relaxed);
            if let Ok(mut s) = self.shared.lock() {
                s.status = MacroStatus::Running;
                s.log("Macro resumed.");
            }
        }
    }

    pub fn pause(&self) {
        if !self.stop_flag.load(Ordering::Relaxed) {
            self.paused_flag.store(true, Ordering::Relaxed);
            release_all_movement();
            set_left_mouse(false);
            if let Ok(mut s) = self.shared.lock() {
                s.status = MacroStatus::Paused;
                s.log("Macro paused.");
            }
        }
    }

    pub fn stop(&self) {
        stop_inner(&self.shared, &self.stop_flag, &self.paused_flag);
    }
}

fn stop_inner(
    shared: &Arc<Mutex<SharedState>>,
    stop_flag: &Arc<AtomicBool>,
    paused_flag: &Arc<AtomicBool>,
) {
    if !stop_flag.swap(true, Ordering::Relaxed) {
        paused_flag.store(false, Ordering::Relaxed);
        release_all_movement();
        set_left_mouse(false);
        if let Ok(mut s) = shared.lock() {
            s.status = MacroStatus::Idle;
            s.log("Macro stopped.");
        }
    }
}

fn start_inner(
    shared: &Arc<Mutex<SharedState>>,
    stop_flag: &Arc<AtomicBool>,
    paused_flag: &Arc<AtomicBool>,
    config_arc: &Arc<Mutex<MacroConfig>>,
) {
    let Some(hwnd) = find_roblox_window() else {
        if let Ok(mut s) = shared.lock() {
            s.log("Roblox window not found. Open Roblox first.");
        }
        return;
    };

    focus_roblox(hwnd);
    stop_flag.store(false, Ordering::Relaxed);
    paused_flag.store(false, Ordering::Relaxed);

    let shared = Arc::clone(shared);
    let stop_flag = Arc::clone(stop_flag);
    let paused_flag = Arc::clone(paused_flag);
    let config_arc = Arc::clone(config_arc);

    thread::spawn(move || {
        if let Ok(mut s) = shared.lock() {
            s.status = MacroStatus::Running;
            s.log("Macro started.");
        }

        // Small grace delay after focusing Roblox
        if !interruptible_sleep(400, &stop_flag) {
            return;
        }

        loop {
            if stop_flag.load(Ordering::Relaxed) {
                break;
            }

            let cfg = config_arc.lock().map(|c| c.clone()).unwrap_or_default();

            if let Ok(mut s) = shared.lock() {
                s.status = MacroStatus::Running;
                s.log(format!(
                    "Resetting to hive slot {} → traveling to {}...",
                    cfg.hive_slot,
                    cfg.field.label()
                ));
            }

            if !reset_to_hive(&stop_flag) {
                break;
            }
            if !travel_to_field(cfg.field, cfg.hive_slot, cfg.move_speed, &stop_flag) {
                break;
            }

            if let Ok(mut s) = shared.lock() {
                s.log(format!(
                    "Gathering with {} pattern ({} min max)...",
                    cfg.pattern.label(),
                    cfg.gather_minutes
                ));
            }

            let gather_start = Instant::now();
            let max_gather = Duration::from_secs(u64::from(cfg.gather_minutes.max(1)) * 60);

            if cfg.auto_click_gather {
                set_left_mouse(true);
            }

            while !stop_flag.load(Ordering::Relaxed) && gather_start.elapsed() < max_gather {
                while paused_flag.load(Ordering::Relaxed) && !stop_flag.load(Ordering::Relaxed) {
                    set_left_mouse(false);
                    release_all_movement();
                    thread::sleep(Duration::from_millis(100));
                }
                if stop_flag.load(Ordering::Relaxed) {
                    break;
                }

                if cfg.auto_click_gather {
                    set_left_mouse(true);
                }

                if !run_pattern(cfg.pattern, cfg.pattern_size, cfg.move_speed, &stop_flag) {
                    break;
                }

                if cfg.convert_at_full {
                    if let Some(hwnd_now) = find_roblox_window() {
                        if let Some(rect) = get_client_rect_screen(hwnd_now) {
                            if is_backpack_full(rect) {
                                if let Ok(mut s) = shared.lock() {
                                    s.log("Backpack full detected — returning to hive.");
                                }
                                break;
                            }
                        }
                    }
                }
            }

            set_left_mouse(false);
            release_all_movement();

            if stop_flag.load(Ordering::Relaxed) {
                break;
            }

            if let Ok(mut s) = shared.lock() {
                s.status = MacroStatus::Converting;
                s.log("Converting pollen at hive...");
            }

            if !convert_at_hive(&stop_flag) {
                break;
            }

            if let Ok(mut s) = shared.lock() {
                s.cycles_completed += 1;
                let count = s.cycles_completed;
                s.log(format!("Cycle #{count} complete."));
            }
        }

        release_all_movement();
        set_left_mouse(false);
        stop_flag.store(true, Ordering::Relaxed);
        if let Ok(mut s) = shared.lock() {
            s.status = MacroStatus::Idle;
        }
    });
}
