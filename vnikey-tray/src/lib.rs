use ksni::blocking::{Handle, TrayMethods};
use ksni::{MenuItem, ToolTip, Tray};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU8, Ordering},
};

/// Callback invoked whenever the VI/EN state is toggled via the tray (click or menu).
/// The argument is the **new** state: `true` = Vietnamese, `false` = English.
pub type ToggleCallback = Arc<dyn Fn(bool) + Send + Sync>;

/// Callback invoked when the user switches the input method from the tray menu.
/// The argument is the new value: `0` = Telex, `1` = VNI.
pub type InputMethodCallback = Arc<dyn Fn(u8) + Send + Sync>;

pub struct VnikeyTray {
    pub is_vietnamese_enabled: Arc<AtomicBool>,
    pub input_method: Arc<AtomicU8>,
    /// Called after every VI/EN toggle so that the frontend can emit the
    /// `StateChanged` DBus signal and update the GNOME panel indicator.
    pub on_toggle: Option<ToggleCallback>,
    /// Called after every input-method change so the frontend can persist
    /// the new value to `config.toml`.
    pub on_input_method_change: Option<InputMethodCallback>,
}

impl Tray for VnikeyTray {
    fn id(&self) -> String {
        "vnikey".into()
    }

    fn icon_name(&self) -> String {
        if self.is_vietnamese_enabled.load(Ordering::SeqCst) {
            "input-keyboard-symbolic".into()
        } else {
            "input-keyboard".into()
        }
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        let current = self.is_vietnamese_enabled.load(Ordering::SeqCst);
        let new_state = !current;
        self.is_vietnamese_enabled
            .store(new_state, Ordering::SeqCst);
        if let Some(cb) = &self.on_toggle {
            cb(new_state);
        }
    }

    fn title(&self) -> String {
        if self.is_vietnamese_enabled.load(Ordering::SeqCst) {
            "[V] Vietnamese".into()
        } else {
            "[E] English".into()
        }
    }

    fn tool_tip(&self) -> ToolTip {
        let text = if self.is_vietnamese_enabled.load(Ordering::SeqCst) {
            "[V] Vietnamese"
        } else {
            "[E] English"
        };
        ToolTip {
            title: text.into(),
            description: "vnikey input method".into(),
            icon_name: "input-keyboard".into(),
            icon_pixmap: Vec::new(),
        }
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let is_vi = self.is_vietnamese_enabled.load(Ordering::SeqCst);
        let current_method = self.input_method.load(Ordering::Relaxed);

        vec![
            MenuItem::Standard(ksni::menu::StandardItem {
                label: if is_vi {
                    "✓ Tiếng Việt".into()
                } else {
                    "  Tiếng Anh".into()
                },
                activate: Box::new(|this: &mut Self| {
                    let current = this.is_vietnamese_enabled.load(Ordering::SeqCst);
                    let new_state = !current;
                    this.is_vietnamese_enabled
                        .store(new_state, Ordering::SeqCst);
                    if let Some(cb) = &this.on_toggle {
                        cb(new_state);
                    }
                }),
                ..Default::default()
            }),
            MenuItem::Separator,
            MenuItem::Standard(ksni::menu::StandardItem {
                label: if current_method == 0 {
                    "✓ Telex".into()
                } else {
                    "  Telex".into()
                },
                activate: Box::new(|this: &mut Self| {
                    this.input_method.store(0, Ordering::Relaxed);
                    if let Some(cb) = &this.on_input_method_change {
                        cb(0);
                    }
                }),
                ..Default::default()
            }),
            MenuItem::Standard(ksni::menu::StandardItem {
                label: if current_method == 1 {
                    "✓ VNI".into()
                } else {
                    "  VNI".into()
                },
                activate: Box::new(|this: &mut Self| {
                    this.input_method.store(1, Ordering::Relaxed);
                    if let Some(cb) = &this.on_input_method_change {
                        cb(1);
                    }
                }),
                ..Default::default()
            }),
            MenuItem::Separator,
            MenuItem::Standard(ksni::menu::StandardItem {
                label: "Thoát".into(),
                icon_name: "application-exit".into(),
                activate: Box::new(|_| {
                    std::process::exit(0);
                }),
                ..Default::default()
            }),
        ]
    }
}

/// Spawn the system tray icon.
///
/// * `is_vietnamese_enabled` — shared atomic flag for the current VI/EN state.
/// * `input_method` — shared atomic for the current input method (0 = Telex, 1 = VNI).
/// * `on_toggle` — optional callback invoked after every VI/EN toggle from the tray.
///   The frontend should use this to emit the `StateChanged` DBus signal so that the
///   GNOME panel indicator stays in sync.
/// * `on_input_method_change` — optional callback invoked when the user picks a different
///   input method (Telex/VNI) from the tray menu.  The frontend should persist the new
///   value to `config.toml` so it survives a daemon restart.
pub fn spawn_tray(
    is_vietnamese_enabled: Arc<AtomicBool>,
    input_method: Arc<AtomicU8>,
    on_toggle: Option<ToggleCallback>,
    on_input_method_change: Option<InputMethodCallback>,
) -> Option<Handle<VnikeyTray>> {
    let tray = VnikeyTray {
        is_vietnamese_enabled,
        input_method,
        on_toggle,
        on_input_method_change,
    };
    match tray.spawn() {
        Ok(handle) => Some(handle),
        Err(e) => {
            eprintln!("Warning: Failed to spawn system tray icon: {}", e);
            None
        }
    }
}
