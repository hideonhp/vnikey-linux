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
        self.config.toggle_modifier = modifier.to_lowercase();
    }

    fn set_start_enabled(&mut self, enabled: bool) {
        self.config.start_enabled = enabled;
    }

    fn set_vim_mode(&mut self, enabled: bool) {
        self.config.vim_mode = enabled;
    }

    fn set_per_window_state(&mut self, enabled: bool) {
        self.config.per_window_state = enabled;
    }

    fn set_notification_enabled(&mut self, enabled: bool) {
        self.config.notification_enabled = enabled;
    }

    fn save_config(&mut self) {
        if let Err(e) = self.config.save() {
            self.save_status = format!("Lỗi khi lưu: {}", e);
            return;
        }

        self.save_status = "Đã lưu! Daemon sẽ tự động reload trong vài giây.".to_string();
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
        assert_eq!(state.config.toggle_modifier, "shift");
        state.config.toggle_key = "z".to_string();
        assert_eq!(state.config.toggle_key, "z");

        // 4. Start enabled
        state.set_start_enabled(false);
        assert_eq!(state.config.start_enabled, false);
    }
}

struct VniKeyGui {
    state: AppState,
    active_tab: usize,
    test_text: String,
}

impl Default for VniKeyGui {
    fn default() -> Self {
        Self {
            state: AppState::new(),
            active_tab: 0,
            test_text: String::new(),
        }
    }
}

impl eframe::App for VniKeyGui {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading(egui::RichText::new("⌨ Cấu hình VNIKey").size(24.0).strong());
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.active_tab, 0, "Chung (General)");
                ui.selectable_value(&mut self.active_tab, 1, "Phím tắt (Shortcuts)");
                ui.selectable_value(&mut self.active_tab, 2, "Giới thiệu (About)");
            });
            ui.separator();

            if self.active_tab == 0 {
                ui.label(egui::RichText::new("Kiểu gõ:").strong());
                ui.horizontal(|ui| {
                    if ui
                        .radio(
                            self.state.config.input_method.eq_ignore_ascii_case("telex"),
                            "TELEX",
                        )
                        .clicked()
                    {
                        self.state.toggle_input_method("telex");
                    }
                    if ui
                        .radio(
                            self.state.config.input_method.eq_ignore_ascii_case("vni"),
                            "VNI",
                        )
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

                ui.add_space(10.0);

                let mut vim_mode = self.state.config.vim_mode;
                if ui
                    .checkbox(
                        &mut vim_mode,
                        "Vim Mode (Tự động tắt tiếng Việt khi bấm ESC)",
                    )
                    .changed()
                {
                    self.state.set_vim_mode(vim_mode);
                }

                ui.add_space(10.0);

                let mut per_window_state = self.state.config.per_window_state;
                if ui
                    .checkbox(
                        &mut per_window_state,
                        "Lưu trạng thái tiếng Việt theo từng cửa sổ (Per-window state)",
                    )
                    .changed()
                {
                    self.state.set_per_window_state(per_window_state);
                }

                ui.add_space(10.0);

                let mut notification_enabled = self.state.config.notification_enabled;
                if ui
                    .checkbox(
                        &mut notification_enabled,
                        "Hiện thông báo khi chuyển chế độ (Notification)",
                    )
                    .changed()
                {
                    self.state.set_notification_enabled(notification_enabled);
                }
                ui.add_space(10.0);

                ui.label(egui::RichText::new("Clipboard inject delay (X11 only):").strong());
                ui.horizontal(|ui| {
                    let mut timeout = self.state.config.clipboard_timeout_ms as f32;
                    let slider = egui::Slider::new(&mut timeout, 10.0..=200.0)
                        .step_by(5.0)
                        .suffix(" ms")
                        .text("Clipboard timeout");
                    if ui.add(slider).changed() {
                        self.state.config.clipboard_timeout_ms = timeout as u64;
                    }
                });
                ui.label(
                    egui::RichText::new(
                        "Tăng lên nếu chữ bị mất khi gõ nhanh trên X11 (máy cũ, HDD).",
                    )
                    .small()
                    .color(egui::Color32::GRAY),
                );
            } else if self.active_tab == 1 {
                ui.horizontal(|ui| {
                    ui.label("Phím Modifier (Control, Alt, Super, Shift):");
                    let current_mod_text = if self.state.config.toggle_modifier.is_empty() {
                        "None"
                    } else {
                        self.state.config.toggle_modifier.as_str()
                    };
                    egui::ComboBox::from_id_salt("mod_combo")
                        .selected_text(current_mod_text)
                        .show_ui(ui, |ui: &mut egui::Ui| {
                            let modifiers = ["Control", "Alt", "Super", "Shift", "None"];
                            for m in modifiers {
                                let is_selected = if m == "None" {
                                    self.state.config.toggle_modifier.is_empty()
                                } else {
                                    self.state.config.toggle_modifier.eq_ignore_ascii_case(m)
                                };

                                if ui.selectable_label(is_selected, m).clicked() {
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
            } else if self.active_tab == 2 {
                ui.add_space(10.0);
                ui.heading(egui::RichText::new("VNIKey").size(20.0).strong());
                ui.label(format!("Phiên bản: {}", env!("CARGO_PKG_VERSION")));
                ui.add_space(8.0);
                ui.label("Bộ gõ tiếng Việt cho Linux — hỗ trợ Wayland, X11 và IBus.");
                ui.add_space(4.0);
                ui.label("License: MIT");
                ui.add_space(8.0);
                ui.hyperlink_to(
                    "GitHub Repository",
                    "https://github.com/hideonhp/vnikey-linux",
                );
                ui.add_space(4.0);
                ui.hyperlink_to(
                    "Báo lỗi / Issues",
                    "https://github.com/hideonhp/vnikey-linux/issues",
                );
                ui.add_space(4.0);
                ui.hyperlink_to(
                    "Changelog / Releases",
                    "https://github.com/hideonhp/vnikey-linux/releases",
                );
            }

            ui.add_space(20.0);

            if ui.button("Lưu").clicked() {
                self.state.save_config();
            }

            if !self.state.save_status.is_empty() {
                ui.add_space(10.0);
                let status_text = if self.state.save_status.contains("Lỗi") {
                    egui::RichText::new(&self.state.save_status).color(egui::Color32::RED)
                } else {
                    egui::RichText::new(&self.state.save_status).color(egui::Color32::GREEN)
                };
                ui.label(status_text);
            }

            ui.add_space(15.0);
            ui.separator();
            ui.add_space(5.0);

            ui.label(egui::RichText::new("Gõ thử:").strong());
            ui.add(
                egui::TextEdit::multiline(&mut self.test_text)
                    .desired_rows(3)
                    .hint_text("Click vào đây để gõ thử tiếng Việt...")
                    .desired_width(f32::INFINITY),
            );
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([450.0, 350.0])
            .with_min_inner_size([350.0, 300.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Cấu hình VNIKey",
        options,
        Box::new(|_cc| Ok(Box::new(VniKeyGui::default()))),
    )
}
