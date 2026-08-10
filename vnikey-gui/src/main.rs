use eframe::egui;
use std::process::Command;
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

    fn save_and_restart(&mut self) {
        if let Err(e) = self.config.save() {
            self.save_status = format!("Lỗi khi lưu: {}", e);
            return;
        }

        // Kill existing daemons
        let _ = Command::new("killall")
            .args(["vnikey-wayland", "vnikey-x11", "vnikey-tray"])
            .output();

        self.save_status = "Đã lưu thành công! Vui lòng khởi động lại bộ gõ.".to_string();
    }
}

struct VniKeyGui {
    state: AppState,
}

impl Default for VniKeyGui {
    fn default() -> Self {
        Self {
            state: AppState::new(),
        }
    }
}

impl eframe::App for VniKeyGui {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // eframe 0.35 gives us a bare ui, but it doesn't have background by default so we wrap in central panel
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Cấu hình VNIKey");
            ui.add_space(10.0);

            ui.label("Kiểu gõ:");
            let mut current_method = self.state.config.input_method.to_lowercase();

            ui.horizontal(|ui| {
                if ui.radio_value(&mut current_method, "telex".to_string(), "TELEX").clicked() {
                    self.state.toggle_input_method("telex");
                }
                if ui.radio_value(&mut current_method, "vni".to_string(), "VNI").clicked() {
                    self.state.toggle_input_method("vni");
                }
            });

            ui.add_space(10.0);

            let mut spell_check = self.state.config.spell_check;
            if ui.checkbox(&mut spell_check, "Bật kiểm tra chính tả (Smart Spell Check)").changed() {
                self.state.set_spell_check(spell_check);
            }

            ui.add_space(20.0);

            if ui.button("Lưu & Khởi động lại").clicked() {
                self.state.save_and_restart();
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
