#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod capture;
mod config;
mod input;
mod paths;
mod patterns;
mod runner;
mod walk;

use config::{Field, MacroConfig, PatternKind};
use eframe::egui;
use runner::{MacroController, MacroStatus};

struct RustroApp {
    config: MacroConfig,
    controller: MacroController,
    save_feedback: Option<String>,
}

impl RustroApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        let config = MacroConfig::load();
        let controller = MacroController::new(config.clone());
        Self {
            config,
            controller,
            save_feedback: None,
        }
    }
}

impl eframe::App for RustroApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(std::time::Duration::from_millis(120));

        let (status, cycles, roblox_ok, live_speed, live_haste, logs) = {
            let s = self.controller.shared.lock().unwrap();
            (
                s.status,
                s.cycles_completed,
                s.roblox_detected,
                s.live_speed,
                s.live_haste,
                s.logs.iter().cloned().collect::<Vec<_>>(),
            )
        };

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new("RUSTROMACRO")
                        .strong()
                        .color(egui::Color32::from_rgb(235, 87, 87)),
                );
                ui.label(egui::RichText::new("v0.1.0").weak());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let (badge_text, badge_color) = match status {
                        MacroStatus::Idle => ("IDLE", egui::Color32::GRAY),
                        MacroStatus::Running => ("GATHERING", egui::Color32::from_rgb(76, 175, 80)),
                        MacroStatus::Paused => ("PAUSED", egui::Color32::from_rgb(255, 193, 7)),
                        MacroStatus::Converting => {
                            ("CONVERTING", egui::Color32::from_rgb(66, 165, 245))
                        }
                    };
                    ui.label(egui::RichText::new(badge_text).strong().color(badge_color));
                    ui.separator();
                    let rbx_color = if roblox_ok {
                        egui::Color32::from_rgb(76, 175, 80)
                    } else {
                        egui::Color32::from_rgb(235, 87, 87)
                    };
                    ui.label(
                        egui::RichText::new(if roblox_ok {
                            "Roblox: Found"
                        } else {
                            "Roblox: Not Found"
                        })
                        .color(rbx_color),
                    );
                });
            });

            ui.separator();

            // Controls
            ui.horizontal(|ui| {
                if status == MacroStatus::Idle || status == MacroStatus::Paused {
                    if ui.button("▶ Start / Resume (F1)").clicked() {
                        self.controller.update_config(self.config.clone());
                        self.controller.start_or_resume();
                    }
                } else if ui.button("⏸ Pause (F1)").clicked() {
                    self.controller.pause();
                }

                if ui
                    .add_enabled(status != MacroStatus::Idle, egui::Button::new("⏹ Stop (F2)"))
                    .clicked()
                {
                    self.controller.stop();
                }

                if ui.button("💾 Save Config").clicked() {
                    self.controller.update_config(self.config.clone());
                    self.save_feedback = match self.config.save() {
                        Ok(()) => Some("Saved rustro_config.toml".to_string()),
                        Err(e) => Some(format!("Save error: {e}")),
                    };
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("Cycles: {cycles}  |  Live Speed: {live_speed:.1} (Haste {live_haste}x)"));
                });
            });

            if let Some(msg) = &self.save_feedback {
                ui.label(egui::RichText::new(msg).small().weak());
            }

            ui.add_space(6.0);

            // Settings Grid
            egui::Grid::new("settings_grid")
                .num_columns(2)
                .spacing([16.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Field");
                    egui::ComboBox::from_id_salt("field_combo")
                        .selected_text(self.config.field.label())
                        .show_ui(ui, |ui| {
                            for &f in Field::ALL {
                                ui.selectable_value(&mut self.config.field, f, f.label());
                            }
                        });
                    ui.end_row();

                    ui.label("Pattern");
                    egui::ComboBox::from_id_salt("pattern_combo")
                        .selected_text(self.config.pattern.label())
                        .show_ui(ui, |ui| {
                            for &p in PatternKind::ALL {
                                ui.selectable_value(&mut self.config.pattern, p, p.label());
                            }
                        });
                    ui.end_row();

                    ui.label("Hive Slot (1–6)");
                    ui.add(egui::Slider::new(&mut self.config.hive_slot, 1..=6));
                    ui.end_row();

                    ui.label("Gather Max (min)");
                    ui.add(egui::Slider::new(&mut self.config.gather_minutes, 1..=45));
                    ui.end_row();

                    ui.label("Move Speed");
                    ui.add(
                        egui::DragValue::new(&mut self.config.move_speed)
                            .speed(0.05)
                            .range(10.0..=60.0)
                            .max_decimals(2),
                    );
                    ui.end_row();

                    ui.label("Pattern Size");
                    ui.add(egui::Slider::new(&mut self.config.pattern_size, 1..=6));
                    ui.end_row();
                });

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.config.convert_at_full, "Convert when backpack full");
                ui.checkbox(&mut self.config.auto_click_gather, "Hold click to gather");
            });

            self.controller.update_config(self.config.clone());

            ui.add_space(6.0);
            ui.separator();
            ui.label(egui::RichText::new("Activity Log").strong());

            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for line in &logs {
                        ui.label(egui::RichText::new(line).monospace().size(11.5));
                    }
                });
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([460.0, 520.0])
            .with_min_inner_size([400.0, 440.0]),
        ..Default::default()
    };
    eframe::run_native(
        "RustroMacro",
        options,
        Box::new(|cc| Ok(Box::new(RustroApp::new(cc)))),
    )
}
