<p align="center">
  <img src="https://img.shields.io/badge/Nền_tảng-Linux_(Wayland_|_X11)-blue?style=for-the-badge" alt="Platform">
  <img src="https://img.shields.io/badge/Ngôn_ngữ-Rust_1.85+-orange?style=for-the-badge" alt="Rust">
  <img src="https://img.shields.io/badge/Giấy_phép-MIT-green?style=for-the-badge" alt="License">
  <img src="https://img.shields.io/badge/Phiên_bản-0.1.23-purple?style=for-the-badge" alt="Version">
  <img src="https://img.shields.io/badge/Kiểm_thử-151_đạt_(100%25)-brightgreen?style=for-the-badge" alt="Tests">
</p>

<h1 align="center">
  <br>
  ⌨️ Viet+ (VietC)
  <br>
</h1>

<p align="center">
  <b>Bộ gõ tiếng Việt thế hệ mới cho Linux (Wayland & X11)</b><br>
  <sub>Zero Underline &bull; Gõ trực tiếp không độ trễ &bull; Tự động phục hồi tiếng Anh &bull; Viết bằng Rust 🦀</sub>
</p>

<p align="center">
  <a href="README.en.md">🌐 English Documentation</a>
</p>

---

## 🌟 Giới thiệu

**Viet+ (VietC)** là bộ gõ tiếng Việt hiệu năng cao dành cho Linux. Được phát triển hoàn toàn bằng **Rust**, Viet+ loại bỏ triệt để hiện tượng gạch chân khó chịu (pre-edit underline) và lỗi tranh chấp clipboard của các bộ gõ truyền thống, mang lại trải nghiệm gõ tiếng Việt mượt mà trực tiếp trên cả **Wayland** (Hyprland, Sway, GNOME, KDE Plasma) lẫn **X11**.

```
Nguyeenx DDawng Khoa   ➔   Nguyễn Đăng Khoa
Khoong cos gif quis    ➔   Không có gì quí
search for the test    ➔   search for the test  (Tự động giữ nguyên từ tiếng Anh)
```

---

## 🚀 Cài đặt nhanh (1 lệnh duy nhất)

Cài đặt Viet+ nhanh chóng trên mọi bản phân phối Linux:

```bash
curl -fsSL https://raw.githubusercontent.com/vndangkhoa/vietc/main/install.sh | bash
```

### Chuyển đổi chế độ gõ
- Nhấn **`Ctrl + Space`** (hoặc `Ctrl + Shift`) để xoay vòng: **⚪ ENG (Tiếng Anh) ➔ 🔴 VNI ➔ 🔵 TELEX ➔ ⚪ ENG**
- Hoặc nhấp chuột vào biểu tượng khay hệ thống động (**EN / VN / TLX**) bất cứ lúc nào.
- Dòng lệnh CLI: `vietcctl status` | `vietcctl cycle` | `vietcctl method telex`

---

## 🐧 Danh sách bản phân phối được hỗ trợ & tối ưu

Viet+ được kiểm thử và tối ưu chuyên sâu cho các bản phân phối Linux thịnh hành hiện nay:

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

| Tính năng | Mô tả chi tiết |
| :--- | :--- |
| 🚀 **Zero Underline** | Gõ trực tiếp vào ứng dụng đang kích hoạt. Không dùng bộ đệm tạm thời, không gạch chân gây phân tâm, không lỗi sao chép/dán. |
| ⚡ **Direct Wayland Virtual Keyboard** | Sử dụng `wtype` (`zwp_virtual_keyboard_v1`) trên Wayland/Hyprland. Ký tự UTF-8 được gửi trực tiếp với độ trễ 0ms, không phụ thuộc clipboard. |
| 🛡️ **Lọc thiết bị phần cứng chuẩn xác** | Tự động nhận diện và chỉ chiếm giữ cổng `/dev/input/by-path/*-event-kbd`, khắc phục triệt để lỗi gõ lặp chữ từ đầu thu USB không dây 2.4G và bàn phím kép. |
| 🧠 **Tự động nhận diện tiếng Anh thông minh** | Phân tích âm vị học tiếng Việt kết hợp từ điển thuật ngữ kỹ thuật tiếng Anh. Khi gõ từ tiếng Anh ở chế độ Telex/VNI, từ sẽ tự động giữ nguyên định dạng tiếng Anh chuẩn khi nhấn dấu cách hoặc dấu câu. |
| 🔄 **Xoay vòng 3 chế độ tức thì** | Chuyển đổi mượt mà **⚪ ENG ➔ 🔴 VNI ➔ 🔵 TELEX** bằng `Ctrl + Shift` hoặc click khay hệ thống, đồng bộ thông báo OSD trên màn hình. |
| 🎋 **Lõi biến đổi Bamboo** | Đầy đủ bảng chữ cái tiếng Việt (`â, ă, ê, ô, ơ, ư, đ`), ghép âm thông minh (`uo ➔ ươ`, `ua ➔ ưa`) và đặt dấu chuẩn xác. |
| 🔡 **Bảo toàn viết hoa/thường** | Giữ nguyên định dạng hoa thường: `Tieengs ➔ Tiếng`, `TIEENGS ➔ TIẾNG`. |
| 📝 **Gõ tắt mở rộng (Macro)** | Hỗ trợ mở rộng gõ tắt mặc định và tùy biến (`ko ➔ không`, `dc ➔ được`, `vs ➔ với`...). |
| 📊 **Biểu tượng khay hệ thống động** | Khay hệ thống StatusNotifier hiển thị huy hiệu màu sắc rõ nét theo từng chế độ (**EN / VN / TLX**). |
| 🔒 **Bảo mật & Rootless** | Chạy dưới dạng service người dùng systemd (`vietc.service`), không cần quyền root sau khi thiết lập uinput ban đầu. |

---

## 🎮 Hướng dẫn sử dụng

### 1. Phương thức gõ

#### **Chế độ Telex (🔵)**
| Phím gõ | Kết quả | Ví dụ |
| :--- | :--- | :--- |
| `s` | Sắc (´) | `as` ➔ `á` |
| `f` | Huyền (`) | `af` ➔ `à` |
| `r` | Hỏi (̉ ) | `ar` ➔ `ả` |
| `x` | Ngã (~) | `ax` ➔ `ã` |
| `j` | Nặng (.) | `aj` ➔ `ạ` |
| `aa`, `ee`, `oo` | Â, Ê, Ô | `aa` ➔ `â`, `ee` ➔ `ê`, `oo` ➔ `ô` |
| `aw`, `ow`, `uw` | Ă, Ơ, Ư | `aw` ➔ `ă`, `ow` ➔ `ơ`, `uw` ➔ `ư` |
| `dd` | Đ | `dd` ➔ `đ`, `DD` ➔ `Đ` |
| `w` | Ươ | `chuongw` ➔ `chương` |

#### **Chế độ VNI (🔴)**
| Phím gõ | Kết quả | Ví dụ |
| :--- | :--- | :--- |
| `1` | Sắc (´) | `a1` ➔ `á` |
| `2` | Huyền (`) | `a2` ➔ `à` |
| `3` | Hỏi (̉ ) | `a3` ➔ `ả` |
| `4` | Ngã (~) | `a4` ➔ `ã` |
| `5` | Nặng (.) | `a5` ➔ `ạ` |
| `6` | Â, Ê, Ô | `a6` ➔ `â`, `e6` ➔ `ê`, `o6` ➔ `ô` |
| `7` | Ơ, Ư | `o7` ➔ `ơ`, `u7` ➔ `ư` |
| `8` | Ă | `a8` ➔ `ă` |
| `9` | Đ | `d9` ➔ `đ`, `D9` ➔ `Đ` |

---

### 2. Phím tắt & Lệnh điều khiển

| Thao tác | Phím tắt / Lệnh |
| :--- | :--- |
| **Xoay vòng 3 chế độ** (ENG ➔ VNI ➔ TELEX) | **`Ctrl + Space`** / `Ctrl + Shift` (hoặc nhấp chuột trái vào khay) |
| **Kiểm tra trạng thái** | `vietcctl status` |
| **Chuyển chế độ qua CLI** | `vietcctl method telex` / `vietcctl method vni` |
| **Khởi động lại dịch vụ** | `systemctl --user restart vietc.service` |

---

## ⚙️ Tệp cấu hình

Vị trí tệp cấu hình: `~/.config/vietc/config.toml`

```toml
input_method = "telex"          # "telex" hoặc "vni"
toggle_key = "space"            # Ctrl+Space bật/tắt tiếng Việt
start_enabled = true            # Tự bật khi khởi động
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

## 🏗️ Cấu trúc mã nguồn

```
vietc/
├── engine/                  # Lõi biến đổi tiếng Việt Bamboo & kiểm tra chính tả
│   ├── bamboo.rs            # Dấu thanh, râu mũ, xóa ngược & bảo toàn hoa thường
│   ├── english.rs           # Từ điển từ tiếng Anh & thuật ngữ kỹ thuật
│   ├── spelling.rs          # Phân tích cấu trúc âm tiết tiếng Việt
│   └── tests.rs             # Bộ benchmark kiểm thử tự động
├── daemon/                  # Tiến trình dịch vụ nền chính
│   ├── evdev_loop.rs        # Vòng lặp bắt phím evdev độ trễ cực thấp
│   ├── device.rs            # Phát hiện bàn phím qua by-path & lọc thiết bị
│   ├── daemon.rs            # Điều khiển chế độ, phím tắt & đồng bộ trạng thái
│   └── app_state.rs         # Quản lý cấu hình & ghi nhớ ứng dụng
├── protocol/                # Giao thức giả lập phím ảo Wayland / X11
│   ├── uinput_monitor.rs    # Gõ trực tiếp qua wtype + /dev/uinput
│   └── wayland_im.rs        # Ngữ cảnh Wayland input-method
├── ui/                      # Ứng dụng khay hệ thống (ksni)
│   └── tray.rs              # Hiển thị icon 3 trạng thái (EN / VN / TLX)
├── vietcctl/                # Công cụ dòng lệnh IPC
├── web/                     # Trang web tài liệu tương tác (React + Vite)
└── install.sh               # Kịch bản cài đặt tự động 1 lệnh duy nhất
```

---

## 🧪 Kiểm thử tự động (151 Tests)

Toàn bộ 151 ca kiểm thử đơn vị và tích hợp đều đạt 100%:

```bash
# Chạy toàn bộ kiểm thử trong workspace
cargo test --workspace
```

---

## 📦 Mã nguồn & Kho lưu trữ

Viet+ được đồng bộ trên cả Forgejo và GitHub:
- **GitHub**: [https://github.com/vndangkhoa/vietc](https://github.com/vndangkhoa/vietc)
- **Forgejo**: [https://git.khoavo.myds.me/vndangkhoa/vietc](https://git.khoavo.myds.me/vndangkhoa/vietc)

---

## 📄 Giấy phép

Phát hành dưới **Giấy phép MIT**. Xem tệp [LICENSE](LICENSE) để biết thêm chi tiết.

Được xây dựng với ❤️ dành cho cộng đồng Linux Việt Nam.

