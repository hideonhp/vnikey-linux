import sys

# vnikey-tray
with open('vnikey-tray/src/lib.rs', 'r', encoding='utf-8') as f:
    content = f.read()

content = content.replace(
    ") -> Handle<VnikeyTray> {\n    let tray = VnikeyTray {\n        is_vietnamese_enabled,\n        input_method,\n    };\n    tray.spawn().expect(\"Failed to spawn tray\")\n}",
    ") -> Option<Handle<VnikeyTray>> {\n    let tray = VnikeyTray {\n        is_vietnamese_enabled,\n        input_method,\n    };\n    match tray.spawn() {\n        Ok(handle) => Some(handle),\n        Err(e) => {\n            eprintln!(\"Warning: Failed to spawn system tray icon: {}\", e);\n            None\n        }\n    }\n}"
)

with open('vnikey-tray/src/lib.rs', 'w', encoding='utf-8') as f:
    f.write(content)

# vnikey-wayland
with open('vnikey-wayland/src/main.rs', 'r', encoding='utf-8') as f:
    content = f.read()

content = content.replace(
    "tray_handle: ksni::blocking::Handle<vnikey_tray::VnikeyTray>,",
    "tray_handle: Option<ksni::blocking::Handle<vnikey_tray::VnikeyTray>>,"
)
content = content.replace(
    "state.tray_handle.update(|_| {});",
    "if let Some(tray) = &state.tray_handle { tray.update(|_| {}); }"
)
content = content.replace(
    "self.tray_handle.update(|_| {});",
    "if let Some(tray) = &self.tray_handle { tray.update(|_| {}); }"
)

with open('vnikey-wayland/src/main.rs', 'w', encoding='utf-8') as f:
    f.write(content)

# vnikey-x11
with open('vnikey-x11/src/main.rs', 'r', encoding='utf-8') as f:
    content = f.read()

content = content.replace(
    "tray_handle.update(|_| {});",
    "if let Some(tray) = &tray_handle { tray.update(|_| {}); }"
)

with open('vnikey-x11/src/main.rs', 'w', encoding='utf-8') as f:
    f.write(content)

print('Fixed code')
