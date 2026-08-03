use ksni::blocking::{Handle, TrayMethods};
use ksni::{MenuItem, ToolTip, Tray};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

pub struct VnikeyTray {
    pub is_vietnamese_enabled: Arc<AtomicBool>,
}

impl Tray for VnikeyTray {
    fn id(&self) -> String {
        "vnikey".into()
    }

    fn icon_name(&self) -> String {
        "input-keyboard".into()
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
        vec![MenuItem::Standard(ksni::menu::StandardItem {
            label: "Quit".into(),
            icon_name: "application-exit".into(),
            activate: Box::new(|_| {
                std::process::exit(0);
            }),
            ..Default::default()
        })]
    }
}

pub fn spawn_tray(is_vietnamese_enabled: Arc<AtomicBool>) -> Handle<VnikeyTray> {
    let tray = VnikeyTray {
        is_vietnamese_enabled,
    };
    tray.spawn().expect("Failed to spawn tray")
}
