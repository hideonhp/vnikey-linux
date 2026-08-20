# 🚀 vnikey - The Smart, Blazing Fast Vietnamese IME for Linux

![Language: Rust](https://img.shields.io/badge/Language-Rust-orange.svg)
![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)
![Build: passing](https://img.shields.io/badge/Build-passing-brightgreen.svg)

## 🌟 Introduction

Welcome to **vnikey**, a next-generation, bare-metal Vietnamese Input Method Engine (IME) designed specifically for the Linux desktop. Written purely in **Rust**, vnikey guarantees zero-cost abstractions, uncompromising memory safety, and lightning-fast typing with absolutely zero latency.

We bypass bloated, legacy IME frameworks (like IBus and Fcitx5) by communicating directly with the display server protocols, ensuring a seamless and hyper-optimized typing experience.

## ✨ Key Features

*   **🧠 Smart Phonetic Validation (The "Vampire" Problem Solved):**
    Unlike dumb mechanical IMEs, vnikey understands Vietnamese syllable rules and will automatically revert to raw keystrokes for foreign words. For example, in standard Telex, typing `v a m p i r e` stupidly converts `i + r` to `ỉ` (vampỉe). vnikey's smart phonetic engine detects the invalid Vietnamese consonant cluster `mp` and automatically restores the raw English keystrokes.
*   **🖥️ Dual Display Protocol Support:**
    Seamlessly supports both modern **Wayland** (via `zwp_input_method_v2` and `virtual_keyboard_v1`) and legacy **X11** natively without extra abstraction layers.
*   **⌨️ Input Methods:**
    Full, robust support for both **Telex** and **VNI** typing modes.
*   **⚡ Passthrough Mode:**
    Easy Shift/Ctrl+Space toggle for raw English input (no interference) when you need it.

## 🏗️ Architecture Overview

To ensure maintainability and modularity, the project is structured as a Cargo workspace with heavily specialized, pure-Rust crates:

*   **`vnikey-core`**: The standalone, OS-agnostic logic engine and state machine. It handles all phonetic validation, tone placement, and keystroke processing with strict zero-allocation constraints.
*   **`vnikey-config`**: The central configuration crate for managing settings and loading user preferences.
*   **`vnikey-wayland`**: A native Wayland daemon communicating directly with the compositor for flawless integration.
*   **`vnikey-x11`**: A parallel daemon injecting key events via X11 protocols for legacy systems or traditional window managers.
*   **`vnikey-tray`**: A lightweight system tray DBus indicator to visually display the current IME state.

## 🛠️ Installation & Usage

### 1. Install System Dependencies

Before building, ensure you have the required C-dependencies installed on your system. On Debian/Ubuntu-based systems, you can install them via:

```bash
sudo apt-get install libwayland-dev libxkbcommon-dev libxkbcommon-x11-dev libxcb-shape0-dev libxcb-xfixes0-dev libdbus-1-dev libxtst-dev
```

### 2. Build the Project

Clone the repository and build it using Cargo:

```bash
cargo build --release
```

### 3. Cài đặt và Tự khởi động (Autostart)

Sau khi build xong (bằng lệnh `cargo build --release`), bạn có thể cài đặt thủ công để VNIKey khởi động cùng hệ thống:

1. Copy các file thực thi và launcher vào `~/.local/bin/`:
   ```bash
   mkdir -p ~/.local/bin
   cp target/release/vnikey-wayland target/release/vnikey-x11 target/release/vnikey-gui ~/.local/bin/
   cp vnikey.sh ~/.local/bin/
   chmod +x ~/.local/bin/vnikey.sh
   ```

2. Cấu hình Autostart:
   Copy file `.desktop` vào thư mục autostart của bạn:
   ```bash
   mkdir -p ~/.config/autostart
   cp vnikey-autostart.desktop ~/.config/autostart/
   ```

VNIKey sẽ tự động nhận diện Wayland hoặc X11 ở lần khởi động máy tiếp theo. Để cấu hình bộ gõ, chạy lệnh `vnikey-gui` từ terminal.

---

**vnikey** is built by and for hardcore Linux enthusiasts. Enjoy the blazing-fast typing experience!
