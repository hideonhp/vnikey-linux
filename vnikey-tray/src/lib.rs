use ksni::blocking::{Handle, TrayMethods};
use ksni::{MenuItem, ToolTip, Tray};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU8, Ordering},
};

pub struct VnikeyTray {
    pub is_vietnamese_enabled: Arc<AtomicBool>,
    pub input_method: Arc<AtomicU8>,
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
        self.is_vietnamese_enabled.store(!current, Ordering::SeqCst);
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
                    this.is_vietnamese_enabled.store(!current, Ordering::SeqCst);
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

pub fn spawn_tray(
    is_vietnamese_enabled: Arc<AtomicBool>,
    input_method: Arc<AtomicU8>,
) -> Option<Handle<VnikeyTray>> {
    let tray = VnikeyTray {
        is_vietnamese_enabled,
        input_method,
    };
    match tray.spawn() {
        Ok(handle) => Some(handle),
        Err(e) => {
            eprintln!("Warning: Failed to spawn system tray icon: {}", e);
            None
        }
    }
}
