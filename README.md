<p align="center">
  <a href="https://github.com/vndangkhoa/vietc/stargazers"><img src="https://img.shields.io/github/stars/vndangkhoa/vietc?style=for-the-badge&logo=apachespark&color=f59e0b" alt="GitHub Stars"></a>
  <img src="https://img.shields.io/badge/Nền_tảng-Linux_(Wayland_|_X11)-blue?style=for-the-badge&logo=linux&logoColor=white" alt="Platform">
  <img src="https://img.shields.io/badge/Ngôn_ngữ-Rust_1.85+-orange?style=for-the-badge&logo=rust&logoColor=white" alt="Rust">
  <img src="https://img.shields.io/badge/Phiên_bản-0.1.24-purple?style=for-the-badge" alt="Version">
  <img src="https://img.shields.io/badge/Kiểm_thử-151_đạt_(100%25)-brightgreen?style=for-the-badge" alt="Tests">
  <a href="#giấy-phép"><img src="https://img.shields.io/badge/Giấy_phép-MIT-gray?style=for-the-badge" alt="License"></a>
</p>

<h1 align="center">⌨️ Viet+ (VietC)</h1>

<p align="center">
  <strong>Bộ gõ tiếng Việt thế hệ mới, hiệu năng cao cho Linux (Wayland & X11).</strong><br>
  Được phát triển hoàn toàn bằng <b>Rust</b> với cơ chế gõ phím ảo trực tiếp.<br>
  <i>Zero Underline &bull; Không độ trễ &bull; Không tranh chấp Clipboard &bull; Tự động phục hồi tiếng Anh chuẩn xác.</i>
</p>

<p align="center">
  <a href="#-cài-đặt-nhanh-1-lệnh-duy-nhất"><b>Cài đặt nhanh</b></a> •
  <a href="#-so-sánh-với-các-bộ-gõ-khác"><b>So sánh</b></a> •
  <a href="#-tính-năng-nổi-bật"><b>Tính năng</b></a> •
  <a href="#-danh-sách-bản-phân-phối-được-hỗ-trợ--tối-ưu"><b>Bản phân phối</b></a> •
  <a href="#️-phím-tắt--lệnh-điều-khiển"><b>Phím tắt</b></a> •
  <a href="#-star-history"><b>Star History</b></a> •
  <a href="README.en.md"><b>🌐 English Docs</b></a>
</p>

---

## ⚡ Giới thiệu

Người dùng Linux tại Việt Nam thường xuyên phải đối mặt với các vấn đề nhức nhối khi gõ tiếng Việt trên Wayland: **gạch chân (pre-edit underline) gây gián đoạn**, **xung đột clipboard khi copy/paste**, và **từ tiếng Anh bị nhảy dấu lung tung**.

**Viet+ (VietC)** giải quyết triệt để các vấn đề trên nhờ cơ chế gửi ký tự trực tiếp qua bàn phím ảo (`wtype` / `/dev/uinput`):

```
Nguyeenx DDawng Khoa   ➔   Nguyễn Đăng Khoa
Khoong cos gif quis    ➔   Không có gì quí
search for the test    ➔   search for the test  (Tự động bảo toàn thuật ngữ tiếng Anh)
```

---

## 📊 So sánh với các bộ gõ khác

| Tính năng | 🚀 **Viet+ (VietC)** | 🎋 IBus-Bamboo | 🐧 Fcitx5-Unikey |
| :--- | :---: | :---: | :---: |
| **Ngôn ngữ phát triển** | **100% Rust** | Go / C | C++ |
| **Hiện tượng gạch chân (Pre-edit)** | **❌ Hoàn toàn không (Zero Underline)** | ⚠️ Thường gặp trên Wayland | ⚠️ Hay giật khung popup |
| **Tranh chấp Clipboard khi gõ** | **❌ Không dùng Clipboard** | ⚠️ Thỉnh thoảng bị kẹt | ❌ Không dùng Clipboard |
| **Tự động phục hồi tiếng Anh** | **✅ Tự động theo âm vị học + từ điển** | ⚠️ Bán tự động / Phím tắt | ❌ Thủ công |
| **Tối ưu Wayland / Hyprland** | **✅ Direct `wtype` (0ms delay)** | ⚠️ Phụ thuộc module IBus | ⚠️ Cần cài đặt fcitx5-wayland |
| **Bộ nhớ tiêu thụ (RAM)** | **⚡ ~8–12 MB** | ~25–40 MB | ~30–50 MB |
| **Dịch vụ chạy nền (Daemon)** | **systemd user service (Rootless)** | IBus daemon | Fcitx daemon |

---

## 🚀 Cài đặt nhanh (1 lệnh duy nhất)

Cài đặt Viet+ ngay lập tức trên Arch Linux, CachyOS, Fedora, Ubuntu, Debian, Pop!_OS:

```bash
curl -fsSL https://raw.githubusercontent.com/vndangkhoa/vietc/main/install.sh | bash
```

### Chuyển đổi chế độ gõ
- Nhấn **`Ctrl + Space`** (hoặc `Ctrl + Shift`) để xoay vòng: **⚪ ENG (Tiếng Anh) ➔ 🔴 VNI ➔ 🔵 TELEX ➔ ⚪ ENG**
- Hoặc click trực tiếp vào biểu tượng khay hệ thống (**EN / VN / TLX**).
- Điều khiển qua dòng lệnh: `vietcctl status` | `vietcctl cycle` | `vietcctl method telex`

---

## 🐧 Danh sách bản phân phối được hỗ trợ & tối ưu

| Hệ điều hành / Hệ sinh thái | Môi trường Desktop / WM | Máy chủ hiển thị | Cơ chế nhập liệu | Trạng thái |
| :--- | :--- | :--- | :--- | :--- |
| ⚡ **CachyOS** | KDE Plasma 6 / Hyprland | **Wayland** | `wtype` (Bàn phím ảo trực tiếp) | ✅ **Tối ưu 100%** |
| 🏹 **Arch Linux** | Hyprland / Sway / KDE / GNOME | **Wayland / X11** | `wtype` / `/dev/uinput` | ✅ **Kiểm thử 100%** |
| 🚀 **EndeavourOS / Omarchy / Garuda** | Hyprland / KDE Plasma / i3 | **Wayland / X11** | `wtype` / `/dev/uinput` | ✅ **Hỗ trợ hoàn hảo** |
| 🎩 **Fedora 40/41 / Nobara** | GNOME 46/47 / KDE Plasma | **Wayland** | Hybrid IBus + `wtype` | ✅ **Hỗ trợ hoàn hảo** |
| 🌿 **Linux Mint** | Cinnamon / XFCE / MATE | **X11** | `/dev/uinput` Direct | ✅ **Hỗ trợ hoàn hảo** |
| 🪐 **Pop!_OS** | COSMIC Desktop / GNOME | **Wayland / X11** | `wtype` / `/dev/uinput` | ✅ **Hỗ trợ hoàn hảo** |
| 🟠 **Ubuntu 24.04+ / Debian 12** | GNOME (Mutter) / X11 | **Wayland / X11** | Hybrid IBus + AppIndicator | ✅ **Hỗ trợ hoàn hảo** |
| 🦎 **Manjaro / openSUSE** | KDE Plasma / XFCE / GNOME | **Wayland / X11** | `wtype` / `/dev/uinput` | ✅ **Hỗ trợ hoàn hảo** |

---

## ✨ Tính năng nổi bật

- 🚀 **Zero Underline**: Gõ trực tiếp vào ứng dụng đang kích hoạt. Không dùng bộ đệm tạm thời, không gạch chân gây phân tâm.
- ⚡ **Direct Wayland Virtual Keyboard**: Sử dụng giao thức `zwp_virtual_keyboard_v1` trên Wayland/Hyprland. Ký tự UTF-8 được gửi trực tiếp với độ trễ 0ms.
- 🛡️ **Lọc thiết bị phần cứng chuẩn xác**: Tự động nhận diện và chỉ chiếm giữ cổng `/dev/input/by-path/*-event-kbd`, loại bỏ triệt để lỗi lặp chữ từ đầu thu USB 2.4G và bàn phím kép.
- 🧠 **Nhận diện tiếng Anh thông minh**: Phân tích âm vị học tiếng Việt kết hợp từ điển thuật ngữ kỹ thuật. Giữ nguyên từ tiếng Anh chuẩn khi gõ dấu cách hoặc dấu câu.
- 🔄 **Xoay vòng 3 chế độ tức thì**: Chuyển đổi mượt mà giữa **⚪ ENG ➔ 🔴 VNI ➔ 🔵 TELEX** kèm thông báo OSD.
- 🎋 **Lõi biến đổi Bamboo**: Đầy đủ bảng chữ cái tiếng Việt (`â, ă, ê, ô, ơ, ư, đ`), ghép âm thông minh (`uo ➔ ươ`, `ua ➔ ưa`) và đặt dấu chuẩn xác.
- 📝 **Gõ tắt mở rộng (Macro)**: Hỗ trợ tùy biến viết tắt tốc ký (`ko ➔ không`, `dc ➔ được`, `vs ➔ với`...).
- 🔒 **Bảo mật & Rootless**: Chạy dưới dạng service người dùng systemd (`vietc.service`), không cần quyền root sau khi phân quyền uinput ban đầu.

---

## ⌨️ Phím tắt & Lệnh điều khiển

| Thao tác | Phím tắt / Lệnh |
| :--- | :--- |
| **Xoay vòng 3 chế độ** (ENG ➔ VNI ➔ TELEX) | **`Ctrl + Space`** / `Ctrl + Shift` (hoặc click icon khay hệ thống) |
| **Kiểm tra trạng thái** | `vietcctl status` |
| **Chuyển chế độ qua CLI** | `vietcctl method telex` / `vietcctl method vni` |
| **Khởi động lại dịch vụ** | `systemctl --user restart vietc.service` |

---

## ⚙️ Cấu hình tùy chỉnh

Tệp cấu hình lưu tại: `~/.config/vietc/config.toml`

```toml
input_method = "telex"          # "telex" hoặc "vni"
toggle_key = "space"            # Ctrl+Space bật/tắt tiếng Việt
start_enabled = true            # Tự bật khi đăng nhập
grab = true                     # Chiếm quyền evdev trực tiếp
deduplicate_keys = false        # Cho phép gõ chữ kép Telex (aa, ee, dd)

[auto_restore]
enabled = true                  # Tự động phục hồi từ tiếng Anh khi nhấn dấu cách
trigger_keys = ["space", "escape"]

[app_state]
enabled = false                 # Ghi nhớ trạng thái theo từng ứng dụng

[macros]
"ko" = "không"
"dc" = "được"
"vs" = "với"
"ng" = "người"
```

---

## 🌟 Hỗ trợ & Đóng góp

Nếu bạn yêu thích sự mượt mà và tốc độ của Viet+:

- Tặng dự án **1 Star ⭐** trên GitHub để lan tỏa đến cộng đồng Linux Việt Nam!
- Tham gia thảo luận, báo lỗi hoặc gửi Pull Request tại [GitHub Issues](https://github.com/vndangkhoa/vietc/issues).

<p align="center">
  <a href="https://star-history.com/#vndangkhoa/vietc&Date">
    <img src="https://api.star-history.com/svg?repos=vndangkhoa/vietc&type=Date" alt="Viet+ Star History" width="75%">
  </a>
</p>

---

## 📄 Giấy phép

Phát hành dưới **Giấy phép MIT**. Xem chi tiết tại [`LICENSE`](LICENSE).

Được phát triển với ❤️ bởi **Khoa Vo ([@vndangkhoa](https://github.com/vndangkhoa))**.
