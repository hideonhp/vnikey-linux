# 🚀 VNIKey - Bộ gõ Tiếng Việt Siêu Tốc & Thông Minh cho Linux

![Language: Rust](https://img.shields.io/badge/Language-Rust-orange.svg)
![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)
![Build: passing](https://img.shields.io/badge/Build-passing-brightgreen.svg)
![Release: v0.1.0 Beta](https://img.shields.io/badge/Release-v0.1.0_Beta-red.svg?style=for-the-badge)

## 🎉 TIN NÓNG: ĐÃ PHÁT HÀNH PHIÊN BẢN BETA v0.1.0! 🎉
Sau bao ngày tháng mong chờ, **VNIKey v0.1.0 (Beta)** đã chính thức ra mắt! 
Chúng tôi mang đến một trải nghiệm gõ tiếng Việt hoàn toàn mới, tối ưu hóa đến từng byte nhớ, loại bỏ hoàn toàn độ trễ (zero latency). Tải ngay bản Release mới nhất tại mục [Releases](https://github.com/hideonhp/vnikey-linux/releases) và trải nghiệm sự khác biệt!

---

## 🌟 Giới thiệu

Chào mừng đến với **VNIKey**, bộ gõ tiếng Việt thế hệ mới (IME) được thiết kế đặc biệt cho môi trường desktop Linux. Được viết 100% bằng **Rust**, VNIKey đảm bảo an toàn bộ nhớ tuyệt đối, zero-cost abstraction, và tốc độ gõ phím nhanh như chớp.

Thay vì chạy qua các framework bộ gõ cồng kềnh, nặng nề (như IBus hay Fcitx5), VNIKey **giao tiếp trực tiếp** ở tầng thấp nhất với các giao thức hiển thị (Wayland/X11), mang lại trải nghiệm mượt mà không đối thủ.

## ✨ Tính năng Nổi bật

*   **🧠 Nhận diện Thông minh (Giải quyết triệt để lỗi "Vampire"):**
    Khác với các bộ gõ cơ học "ngu ngốc" truyền thống, VNIKey hiểu luật chính tả tiếng Việt. Khi bạn gõ từ tiếng Anh (ví dụ: gõ `v a m p i r e` trong Telex bị biến thành `vampỉe`), engine thông minh của VNIKey sẽ phát hiện cụm phụ âm `mp` không tồn tại trong tiếng Việt và **tự động hoàn tác** về tiếng Anh nguyên bản cho bạn. Không còn nỗi lo gõ code hay chat tiếng Anh bị lỗi dấu!
*   **🖥️ Hỗ trợ Song song Wayland & X11:**
    Chạy mượt mà trên cả **Wayland** hiện đại (thông qua `zwp_input_method_v2` và `virtual_keyboard_v1`) và hệ thống **X11** truyền thống mà không cần lớp abstraction trung gian nào.
*   **⌨️ Hỗ trợ chuẩn gõ:**
    Hỗ trợ đầy đủ và chuẩn xác cả hai kiểu gõ phổ biến nhất: **Telex** và **VNI**.
*   **⚡ Chế độ Passthrough:**
    Bật/tắt tiếng Việt - tiếng Anh trong tích tắc bằng phím tắt (Ctrl+Space / Shift+Space).

## 🏗️ Kiến trúc Hệ thống

Dự án được chia thành các crate chuyên biệt, tối ưu hóa tối đa:
- **`vnikey-core`**: Trái tim của bộ gõ. Xử lý logic Telex/VNI, đặt dấu, và kiểm tra chính tả hoàn toàn không cấp phát bộ nhớ động (zero-allocation).
- **`vnikey-config`**: Quản lý cấu hình và phím tắt.
- **`vnikey-wayland`**: Daemon native giao tiếp trực tiếp với Wayland compositor.
- **`vnikey-x11`**: Daemon xử lý key event thông qua X11 protocol.
- **`vnikey-gui` & `vnikey-tray`**: Giao diện cài đặt và khay hệ thống hiển thị trạng thái V/E.

## 🛠️ Hướng dẫn Cài đặt & Sử dụng

## GNOME Wayland (IBus)

VNIKey hỗ trợ GNOME Wayland thông qua `vnikey-ibus` — một IBus engine viết bằng Rust thuần.

### Yêu cầu
- IBus đã được cài: `sudo dnf install ibus` (Fedora) hoặc `sudo apt install ibus` (Ubuntu)
- GNOME Wayland (mặc định trên Fedora 39+, Ubuntu 24.04+)

### Cài đặt
```bash
sudo ./install.sh
ibus restart
```

Sau đó vào **GNOME Settings → Keyboard → Input Sources**, thêm "Vietnamese (VNIKey)".

### Cách dùng
- **Gõ tiếng Việt**: Chuyển sang input source "Vietnamese (VNIKey)" bằng `Super + Space`
- **Quay về tiếng Anh**: Bấm `Super + Space` để chuyển về "English (US)"
- **Đổi phím tắt**: Vào GNOME Settings → Keyboard → Special Character Entry

> Đây là cách hoạt động native nhất với GNOME. VNIKey tận dụng hoàn toàn
> hệ thống quản lý input method của IBus, bao gồm per-window state và
> tray icon ngôn ngữ được tích hợp sẵn trong GNOME top bar.

### Đổi kiểu gõ (Telex/VNI)
Sửa file config tại `~/.config/vnikey/config.toml` — engine tự hot-reload, không cần restart.

### Dành cho Người dùng (Khuyên dùng)
1. Tải file `vnikey-linux-amd64.tar.gz` mới nhất tại mục [Releases](https://github.com/hideonhp/vnikey-linux/releases).
2. Giải nén file.
3. Mở Terminal tại thư mục vừa giải nén và chạy script cài đặt tự động (Lưu ý: script này đang được hoàn thiện, nếu không có sẵn bạn có thể chép thủ công các file vào `~/.local/bin/`):
   ```bash
   chmod +x install.sh
   ./install.sh
   ```
4. Bộ gõ sẽ tự động chạy. Để mở Cài đặt (chọn Telex/Vni, đổi phím tắt), bạn chạy lệnh `vnikey-gui` hoặc click vào icon VNIKey dưới khay hệ thống.

### Dành cho Lập trình viên (Build từ Source)
Cài đặt thư viện C cần thiết (trên Ubuntu/Debian):
```bash
sudo apt-get install libwayland-dev libxkbcommon-dev libxkbcommon-x11-dev libxcb-shape0-dev libxcb-xfixes0-dev libdbus-1-dev libxtst-dev
```
Build với Cargo:
```bash
cargo build --release
```

---
**VNIKey** - Tự hào được tạo ra bởi và dành cho những người đam mê Linux mãnh liệt. Chúc bạn có trải nghiệm gõ phím tuyệt vời nhất!
