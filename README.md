# 🚀 VNIKey - Bộ gõ Tiếng Việt cho Linux

![Language: Rust](https://img.shields.io/badge/Language-Rust-orange.svg)
![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)
![Build: passing](https://img.shields.io/github/actions/workflow/status/hideonhp/vnikey-linux/ci.yml?branch=main&label=Build)
![Release: v0.2.0](https://img.shields.io/badge/Release-v0.2.0-brightgreen.svg?style=for-the-badge)

## 🎉 Phiên bản v0.2.0 đã ra mắt!

Bản release đầu tiên đủ tính năng để sử dụng hàng ngày. Tải ngay tại mục [Releases](https://github.com/hideonhp/vnikey-linux/releases).

---

## 🌟 Giới thiệu

**VNIKey** là bộ gõ tiếng Việt thế hệ mới (IME) được thiết kế đặc biệt cho môi trường desktop Linux. Được viết 100% bằng **Rust**, VNIKey đảm bảo an toàn bộ nhớ, zero-cost abstraction, và tốc độ gõ phím nhanh như chớp.

Thay vì chạy qua các framework bộ gõ cồng kềnh (IBus/Fcitx5), VNIKey **giao tiếp trực tiếp** ở tầng thấp nhất với Wayland/X11, mang lại trải nghiệm mượt mà không đối thủ. Với GNOME Wayland, VNIKey cũng cung cấp IBus engine native để tương thích hoàn toàn.

## ✨ Tính năng

*   **🧠 Nhận diện thông minh (Spell-check):** Engine hiểu luật chính tả tiếng Việt — khi gõ từ tiếng Anh, VNIKey phát hiện cụm phụ âm không hợp lệ và tự động hoàn tác về raw text. Gõ code, chat tiếng Anh không còn bị lỗi dấu!
*   **🖥️ Hỗ trợ song song Wayland & X11:** Native Wayland qua `zwp_input_method_v2`, X11 qua XTest. IBus engine cho GNOME Wayland.
*   **⌨️ Ba kiểu gõ:** **Telex**, **VNI**, và **VIQR** (Vietnamese Quoted Readable — lý tưởng cho terminal/SSH).
*   **⚡ Phím tắt cycle kiểu gõ:** Đổi nhanh Telex → VNI → VIQR không cần mở GUI.
*   **📝 Gõ tắt (Abbreviation):** File `~/.config/vnikey/abbr.toml` — gõ `vn` → expand thành `Việt Nam`.
*   **🔁 Surrounding Text:** Backspace sau khi commit từ để recompose và sửa lại.
*   **🪟 Per-window state:** Nhớ VI/EN riêng cho từng cửa sổ.
*   **🔔 Notification:** Thông báo desktop khi toggle, hiển thị kiểu gõ đang dùng.
*   **🔄 Hot-reload config:** Thay đổi config, engine tự reload ngay, không cần restart.

## 🏗️ Kiến trúc

| Crate | Mô tả |
|-------|-------|
| `vnikey-core` | Engine xử lý Telex/VNI/VIQR, spell-check, zero-allocation |
| `vnikey-config` | Quản lý cấu hình TOML, hot-reload |
| `vnikey-wayland` | Daemon native Wayland (`zwp_input_method_v2`) |
| `vnikey-x11` | Daemon X11 (XTest + clipboard inject) |
| `vnikey-ibus` | IBus engine cho GNOME Wayland |
| `vnikey-gui` | GUI settings (egui) |
| `vnikey-tray` | System tray icon |
| `vnikey-gnome-extension` | GNOME Shell extension (panel V/E indicator) |

## 🛠️ Cài đặt

### Yêu cầu

- **GNOME Wayland (IBus)**: `sudo dnf install ibus` (Fedora) hoặc `sudo apt install ibus` (Ubuntu)
- **Wayland native**: Compositor hỗ trợ `zwp_input_method_v2` (Sway, KWin 5.26+)
- **X11**: Không cần thêm gì

### Từ binary release (khuyến nghị)

```bash
# Tải bản mới nhất tại https://github.com/hideonhp/vnikey-linux/releases
tar xzf vnikey-linux-amd64.tar.gz
cd vnikey-linux
./install.sh
```

**GNOME Wayland (IBus):** Sau khi install, restart IBus:
```bash
ibus restart
```
Vào **GNOME Settings → Keyboard → Input Sources**, thêm "Vietnamese (VNIKey)".

### Build từ source

Cài dependencies (Ubuntu/Debian):
```bash
sudo apt-get install libwayland-dev libxkbcommon-dev libxkbcommon-x11-dev \
  libxcb-shape0-dev libxcb-xfixes0-dev libdbus-1-dev libxtst-dev
```

Build:
```bash
cargo build --release
```

## ⚙️ Cấu hình

File cấu hình: `~/.config/vnikey/config.toml` (tự tạo với giá trị mặc định khi chạy lần đầu).

```toml
input_method = "telex"        # "telex" | "vni" | "viqr"
toggle_modifier = "control"   # Phím toggle VI/EN
toggle_key = "space"
cycle_modifier = "control"    # Phím cycle kiểu gõ
cycle_key = "period"
start_enabled = true
spell_check = true
vim_mode = false
per_window_state = false
notification_enabled = true
clipboard_timeout_ms = 20     # X11 only
```

**Gõ tắt** (`~/.config/vnikey/abbr.toml`):
```toml
vn = "Việt Nam"
hn = "Hà Nội"
tp = "Thành phố Hồ Chí Minh"
```

## 📖 Kiểu gõ VIQR

VIQR dùng ký tự ASCII thuần — lý tưởng cho terminal, SSH, hoặc bàn phím không có phím đặc biệt:

| Ký tự | Tác dụng | Ví dụ |
|-------|----------|-------|
| `'` | Sắc | `a'` → á |
| `` ` `` | Huyền | `` a` `` → à |
| `?` | Hỏi | `a?` → ả |
| `~` | Ngã | `a~` → ã |
| `.` | Nặng | `a.` → ạ |
| `^` | Circumflex | `a^` → â |
| `(` | Breve | `a(` → ă |
| `+` | Horn | `o+` → ơ, `u+` → ư |
| `-` | Stroke | `d-` → đ |

Gõ ký tự modifier hai lần để cancel: `a^^ ` → `a^`.

## 🛠️ Troubleshooting

### VNIKey không gõ được tiếng Việt sau khi cài

**Kiểm tra binary có trong PATH không:**
```bash
which vnikey-wayland   # hoặc vnikey-x11, vnikey-ibus
```
Nếu không tìm thấy: thêm `export PATH="$HOME/.local/bin:$PATH"` vào `~/.bashrc` hoặc `~/.zshrc`.

**Kiểm tra version:**
```bash
vnikey-wayland --version   # → vnikey-wayland 0.2.0
```

---

### IBus không nhận engine VNIKey

```bash
ibus restart
```
Sau đó vào **GNOME Settings → Keyboard → Input Sources**, xóa và thêm lại "Vietnamese (VNIKey)".

Nếu vẫn không thấy:
```bash
ls /usr/share/ibus/component/ | grep vnikey
```

---

### `vnikey-wayland` crash ngay lập tức

Compositor của bạn có thể không hỗ trợ `zwp_input_method_v2`. Xem log:
```bash
journalctl --user -u vnikey-wayland -f
# hoặc chạy thủ công:
vnikey-wayland
```
**Giải pháp**: Dùng `vnikey-ibus` (hoạt động trên GNOME Wayland qua IBus).

---

### Daemon tự tắt / crash

Nếu cài qua systemd service, daemon tự restart sau 2 giây. Xem log:
```bash
journalctl --user -u vnikey-wayland -n 50
```

---

### Không thấy thông báo khi toggle VI/EN

Cài `libnotify`:
```bash
sudo apt install libnotify-bin      # Ubuntu/Debian
sudo dnf install libnotify          # Fedora
```
Hoặc tắt trong **vnikey-gui → tab Chung → bỏ tick "Hiện thông báo"**.

---

## 📋 Changelog

### v0.2.0 (2026-09-07)
- ✅ Kiểu gõ VIQR hoàn chỉnh với integration tests
- ✅ Phím tắt cycle kiểu gõ (Telex → VNI → VIQR)
- ✅ Gõ tắt (Abbreviation/Macro) từ `abbr.toml`
- ✅ GNOME Shell 45–51 support
- ✅ systemd user services
- ✅ Man page `vnikey.1`
- ✅ `--version` flag trên tất cả binary
- ✅ Desktop notification hiển thị kiểu gõ đang dùng
- ✅ GUI: tab About, notification toggle, clipboard timeout slider
- ✅ GNOME Extension `prefs.js`
- ✅ install.sh: kiểm tra dependency trước khi cài

### v0.1.0 (2026-08)
- ✅ Telex & VNI engine, spell-check
- ✅ Wayland native, X11, IBus frontends
- ✅ Surrounding Text (backspace recompose)
- ✅ Per-window state, vim mode
- ✅ System tray, GUI settings, hot-reload

---

**VNIKey** — Tự hào được tạo ra bằng Rust, dành cho cộng đồng Linux Việt Nam. Free & Open Source mãi mãi. 🦀
