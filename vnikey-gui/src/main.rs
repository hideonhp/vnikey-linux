use eframe::egui;
use vnikey_config::Config;

struct AppState {
    config: Config,
    save_status: String,
}

impl AppState {
    fn new() -> Self {
        Self {
            config: Config::load(),
            save_status: String::new(),
        }
    }

    fn toggle_input_method(&mut self, method: &str) {
        self.config.input_method = method.to_string();
    }

    fn set_spell_check(&mut self, enabled: bool) {
        self.config.spell_check = enabled;
    }

    fn set_toggle_modifier(&mut self, modifier: &str) {
        self.config.toggle_modifier = modifier.to_string();
    }

    #[allow(dead_code)]
    fn set_toggle_key(&mut self, key: &str) {
        self.config.toggle_key = key.to_string();
    }

    fn set_start_enabled(&mut self, enabled: bool) {
        self.config.start_enabled = enabled;
    }

    fn save_config(&mut self) {
        if let Err(e) = self.config.save() {
            self.save_status = format!("Lỗi khi lưu: {}", e);
            return;
        }

        self.save_status = "Đã lưu thành công!".to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_toggles() {
        // This is a headless MVVM test for the GUI state.
        let mut state = AppState::new();

        // Check initial state defaults based on Config::default()

        // 1. Toggle input method
        state.toggle_input_method("vni");
        assert_eq!(state.config.input_method, "vni");

        state.toggle_input_method("telex");
        assert_eq!(state.config.input_method, "telex");

        // 2. Toggle spell check
        state.set_spell_check(false);
        assert_eq!(state.config.spell_check, false);

        state.set_spell_check(true);
        assert_eq!(state.config.spell_check, true);

        // 3. Toggle modifier and key
        state.set_toggle_modifier("Shift");
        assert_eq!(state.config.toggle_modifier, "Shift");
        state.set_toggle_key("z");
        assert_eq!(state.config.toggle_key, "z");

        // 4. Start enabled
        state.set_start_enabled(false);
        assert_eq!(state.config.start_enabled, false);
    }
}

struct VniKeyGui {
    state: AppState,
    active_tab: usize,
}

impl Default for VniKeyGui {
    fn default() -> Self {
        Self {
            state: AppState::new(),
            active_tab: 0,
        }
    }
}

impl eframe::App for VniKeyGui {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Cấu hình VNIKey");
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.active_tab, 0, "Chung (General)");
                ui.selectable_value(&mut self.active_tab, 1, "Phím tắt (Shortcuts)");
            });
            ui.separator();

            if self.active_tab == 0 {
                ui.label("Kiểu gõ:");
                let mut current_method = self.state.config.input_method.to_lowercase();

                ui.horizontal(|ui| {
                    if ui
                        .radio_value(&mut current_method, "telex".to_string(), "TELEX")
                        .clicked()
                    {
                        self.state.toggle_input_method("telex");
                    }
                    if ui
                        .radio_value(&mut current_method, "vni".to_string(), "VNI")
                        .clicked()
                    {
                        self.state.toggle_input_method("vni");
                    }
                });

                ui.add_space(10.0);

                let mut spell_check = self.state.config.spell_check;
                if ui
                    .checkbox(
                        &mut spell_check,
                        "Bật kiểm tra chính tả (Smart Spell Check)",
                    )
                    .changed()
                {
                    self.state.set_spell_check(spell_check);
                }

                ui.add_space(10.0);

                let mut start_enabled = self.state.config.start_enabled;
                if ui
                    .checkbox(
                        &mut start_enabled,
                        "Bật tiếng Việt theo mặc định (Start Enabled)",
                    )
                    .changed()
                {
                    self.state.set_start_enabled(start_enabled);
                }
            } else {
                ui.horizontal(|ui| {
                    ui.label("Phím Modifier (Control, Alt, Super, Shift):");
                    let mut current_mod = self.state.config.toggle_modifier.clone();
                    egui::ComboBox::from_id_salt("mod_combo")
                        .selected_text(&current_mod)
                        .show_ui(ui, |ui: &mut egui::Ui| {
                            let modifiers = ["Control", "Alt", "Super", "Shift", "None"];
                            for m in modifiers {
                                if ui
                                    .selectable_value(&mut current_mod, m.to_string(), m)
                                    .clicked()
                                {
                                    self.state.set_toggle_modifier(if m == "None" {
                                        ""
                                    } else {
                                        m
                                    });
                                }
                            }
                        });
                });

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label("Phím kích hoạt (ví dụ: space, z, f1):");
                    if ui
                        .text_edit_singleline(&mut self.state.config.toggle_key)
                        .changed()
                    {
                        self.state.config.toggle_key.make_ascii_lowercase();
                    }
                });
            }

            ui.add_space(20.0);

            if ui.button("Lưu").clicked() {
                self.state.save_config();
            }

            if !self.state.save_status.is_empty() {
                ui.add_space(10.0);
                ui.label(&self.state.save_status);
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 250.0])
            .with_min_inner_size([300.0, 200.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Cấu hình VNIKey",
        options,
        Box::new(|_cc| Ok(Box::new(VniKeyGui::default()))),
    )
}
