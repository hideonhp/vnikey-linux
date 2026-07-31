# 🚀 vnikey-linux

**The Zero-Latency, Native Wayland & X11 Vietnamese Input Method written in pure Rust.**

![Rust](https://img.shields.io/badge/Language-Rust-orange.svg)
![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)
![Build: Passing](https://img.shields.io/badge/Build-Passing-brightgreen.svg)

## ⚠️ The Manifesto (Why does this exist?)
The Linux desktop ecosystem is currently plagued by bloated, legacy IME frameworks (like IBus and Fcitx5) that introduce unnecessary layers, complex configurations, and occasional input latency. 

`vnikey-linux` is our unapologetic answer to this. We are bypassing the middlemen. We are building a next-generation, bare-metal input method daemon that talks directly to the display server protocols. Minimal dependencies, zero garbage collection, zero latency. 

If you want a plug-and-play, hyper-optimized Vietnamese typing experience built from scratch, you're in the right place.

## 🔥 Engineering Marvels (The Core)
At the heart of this project is `vnikey-core`, a flawless engine built with strict memory and performance constraints:
*   **Zero-Allocation Hot Path:** Keystrokes are processed using strictly stack-allocated arrays (`[char; 16]`). No `String`, no `Vec`, no heap allocations during active typing.
*   **O(1) Phonotactic Validator:** A dictionary-free, left-to-right lexer that instantly validates Vietnamese syllable structures. Say goodbye to broken inputs when typing English or code (e.g., typing `code1` will naturally output `code1`, not `codé`).
*   **Smart Tone Placement:** Flawless algorithmic tone targeting (Old Style) that perfectly handles edge cases like `qu`, `gi`, and complex triphthongs (e.g., `nguyễn`, `hoàng`, `thủy`).
*   **Dual Engine:** A seamless, memory-safe state machine supporting both **Telex** and **VNI** natively, with smart modifier cancellation.

## 🏗️ Architecture (The "Tam Tài" Model)
To dominate the fragmented Linux landscape, the project is decoupled into three heavily specialized, pure-Rust crates:
1.  **`vnikey-core`**: The standalone, OS-agnostic logic engine and state machine.
2.  **`vnikey-wayland`**: A native Wayland daemon communicating directly via `zwp_input_method_v2` and `virtual_keyboard_v1` using `wayland-rs`.
3.  **`vnikey-x11`**: A parallel daemon injecting key events directly via XTest/XSendEvent using the `x11rb` crate for legacy systems, NVIDIA users, and traditional WMs (i3wm, bspwm).

## 🗺️ Roadmap
- [x] **Phase 1: `vnikey-core`** - The Zero-allocation Engine (Telex & VNI fully tested).
- [ ] **Phase 2a: `vnikey-wayland`** - Direct Wayland integration.
- [ ] **Phase 2b: `vnikey-x11`** - Pure Rust X11 daemon fallback.
- [ ] **Phase 3: GUI** - Lightweight configuration applet.

## 🤝 Contributing
This project is built by Vibe Coders for hardcore Linux enthusiasts. We welcome contributors who share our vision of a zero-latency Linux typing experience. Bring your best Rust game.
