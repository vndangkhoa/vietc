<p align="center">
  <img src="https://img.shields.io/badge/Nền_tảng-Linux-blue?style=for-the-badge" alt="Platform">
  <img src="https://img.shields.io/badge/Ngôn_ngữ-Rust-orange?style=for-the-badge" alt="Rust">
  <img src="https://img.shields.io/badge/Giấy_phép-MIT-green?style=for-the-badge" alt="License">
  <img src="https://img.shields.io/badge/Phiên_bản-0.1.7-purple?style=for-the-badge" alt="Version">
  <img src="https://img.shields.io/badge/Kiểm_thử-108_đạt-brightgreen?style=for-the-badge" alt="Tests">
  <img src="https://img.shields.io/badge/Event_Sourcing-✓-blueviolet?style=for-the-badge" alt="Event Sourcing">
</p>

<h1 align="center">
  <br>
  Viet+
  <br>
</h1>

<p align="center">
  <b>Bộ gõ tiếng Việt cho Linux</b><br>
  <sub>Không gạch chân &bull; Không bộ đệm pre-edit &bull; Đồng bộ Backspace-Replay &bull; Viết bằng Rust</sub>
</p>

<p align="center">
  <a href="README.md">English</a>
</p>

---

## Viet+ là gì?

Viet+ là một bộ gõ tiếng Việt dành cho Linux sử dụng hướng tiếp cận hoàn toàn khác biệt so với tất cả các bộ gõ khác: **Gõ trực tiếp (Direct Input)**.

Hầu hết các bộ gõ tiếng Việt hiện nay sử dụng **bộ đệm pre-edit** — khi gõ, các ký tự sẽ nằm trong một bộ đệm tạm thời với dấu gạch chân bên dưới, và chỉ thực sự được gửi đi khi bạn hoàn thành từ đó. Điều này gây ra lỗi lặp từ, xao nhãng bởi dấu gạch chân, lỗi sao chép/dán, và mất đồng bộ giữa bộ gõ với nội dung hiển thị trên màn hình.

Viet+ loại bỏ hoàn toàn những nhược điểm trên. Các phím gõ được **chuyển đổi ngay lập tức sang Unicode** — những gì bạn gõ là những gì bạn thấy. Không bộ đệm tạm thời. Không gạch chân. Không lặp chữ.

---

## Tính năng nổi bật

| Tính năng | Nguyên lý hoạt động |
|-----------|---------------------|
| **Gõ trực tiếp** | Không dùng bộ đệm pre-edit. Ký tự được hiển thị ngay lập tức thông qua cơ chế giả lập bàn phím uinput |
| **VNI & Telex** | Hỗ trợ đầy đủ cả hai phương thức gõ, chuyển đổi nhanh bằng phím nóng Ctrl+Shift |
| **Bamboo Engine** | Sử dụng mô hình biến đổi Bamboo — ghép âm, bỏ dấu, đặt dấu và xóa dấu linh hoạt |
| **Ghép âm thông minh** | Hỗ trợ tự động thêm râu/mũ như `uo→ươ` (có hỗ trợ xóa ngược), tự động đặt dấu móc `ua→ưa` |
| **Gõ tắt (Macro)** | Hỗ trợ mở rộng viết tắt như `ko → không`, `dc → được`, và cho phép tự định nghĩa |
| **Giữ nguyên hoa/thường** | Bảo toàn định dạng viết hoa như `Tieengs → Tiếng`, `TIEENGS → TIẾNG` |
| **Nhớ trạng thái theo ứng dụng** | Tự động nhớ trạng thái gõ Anh/Việt cho từng ứng dụng riêng biệt, lưu trữ tại `overrides.toml` |
| **Tải lại cấu hình nóng** | Các thay đổi trong tệp cấu hình được áp dụng ngay lập tức mà không cần khởi động lại bộ gõ |
| **Đặt lại khi chuyển cửa sổ** | Tự động xóa bộ đệm của bộ gõ khi nhấn Alt+Tab chuyển ứng dụng |
| **Độ ưu tiên CPU cao** | Được gán cố định vào các nhân P-core (0-3) và mức ưu tiên nice(-10) để giảm tối đa độ trễ |
| **Giả lập uinput** | Sử dụng `/dev/uinput` giúp hoạt động ổn định trên cả X11 và Wayland |
| **Hỗ trợ Terminal** | ✅ Hoạt động mượt mà trên tất cả các terminal phổ biến: kitty, alacritty, gnome-terminal, konsole, foot, wezterm, st, urxvt, xterm |
| **Tự động nhận diện mật khẩu** | 4 lớp bảo vệ: AT-SPI2 → tiến trình sudo → tiêu đề cửa sổ → lớp (class) cửa sổ |
| **Biểu tượng khay hệ thống** | Hiển thị trạng thái hiện tại: Đỏ (VN) / Xanh dương (TLX) / Xám (EN) |
| **GNOME/Wayland** | Tích hợp sâu thông qua cơ chế D-Bus của GNOME Shell |

---

## Phương thức gõ

Viet+ hỗ trợ đầy đủ hai phương thức gõ **VNI** và **Telex**. Bạn có thể chuyển đổi qua lại bằng phím tắt **Ctrl+LeftShift** hoặc qua menu khay hệ thống.

### VNI

| Phím gõ | Kết quả | Ví dụ |
|---------|---------|-------|
| `1` | á (sắc) | `a1` → `á` |
| `2` | à (huyền) | `a2` → `à` |
| `3` | ả (hỏi) | `a3` → `ả` |
| `4` | ã (ngã) | `a4` → `ã` |
| `5` | ạ (nặng) | `a5` → `ạ` |
| `6` | â/ê/ô | `a6→â`, `e6→ê`, `o6→ô` |
| `7` | ơ/ư | `o7→ơ`, `u7→ư` |
| `8` | ă | `a8→ă` |
| `9` | đ | `d9→đ` |

### Telex

| Phím gõ | Kết quả | Ví dụ |
|---------|---------|-------|
| `s` | á (sắc) | `as→á` |
| `f` | à (huyền) | `af→à` |
| `r` | ả (hỏi) | `ar→ả` |
| `x` | ã (ngã) | `ax→ã` |
| `j` | ạ (nặng) | `aj→ạ` |
| `aa` | â | `aa→â` |
| `ee` | ê | `ee→ê` |
| `oo` | ô | `oo→ô` |
| `ow` | ơ | `ow→ơ` |
| `aw` | ă | `aw→ă` |
| `uw` | ư | `uw→ư` |
| `dd` | đ | `dd→đ` |
| `w` | ươ | `chuongw→chương` |

---

## Phím tắt mặc định

| Tổ hợp phím | Hành động |
|-------------|-----------|
| **Ctrl+Space** | Bật/Tắt bộ gõ tiếng Việt |
| **Ctrl+LeftShift** | Chuyển đổi giữa VNI ↔ Telex |

---

## Tự động nhận diện mật khẩu

Viet+ tích hợp hệ thống nhận diện mật khẩu 4 lớp tự động. Khi phát hiện trường nhập mật khẩu, bộ gõ tiếng Việt sẽ tự động tạm thời tắt để tránh lỗi gõ ký tự đặc biệt:

| Lớp nhận diện | Phương pháp | Đối tượng phát hiện |
|---------------|-------------|---------------------|
| 1 | AT-SPI2 D-Bus (kiểm tra thuộc tính a11y) | Các trường mật khẩu trong các ứng dụng có hỗ trợ a11y |
| 2 | Cây tiến trình (pstree) | Tiến trình `sudo` / `passwd` chạy trong terminal |
| 3 | Từ khóa tiêu đề cửa sổ | Cửa sổ có tiêu đề chứa các từ khóa như `password`, `sudo`, `mật khẩu` |
| 4 | Lớp cửa sổ (Window class) | Các hộp thoại bảo mật như pinentry, polkit, kwallet |

---

## Khả năng hỗ trợ Distro

| Mức độ | Bản phân phối (Distro) | Cách cài đặt | Trạng thái |
|--------|------------------------|--------------|------------|
| ✅ **Hỗ trợ tốt** | Ubuntu, Debian, Linux Mint, Pop!_OS, elementary OS, Zorin, Neon | Trình quản lý `apt` (tự động nhận diện) | Đã kiểm thử, cài đặt bằng một câu lệnh |
| ✅ **Hỗ trợ tốt** | Fedora, RHEL, CentOS | Trình quản lý `dnf` (tự động nhận diện) | Đã kiểm thử, cài đặt bằng một câu lệnh |
| ✅ **Hỗ trợ tốt** | Arch, Manjaro | Trình quản lý `pacman` (tự động nhận diện) | Đã kiểm thử, cài đặt bằng một câu lệnh |
| ⚠️ **Có thể hỗ trợ** | openSUSE, Solus, Void | Trình quản lý `zypper`/`eopkg`/`xbps` (thủ công) | Tên gói phụ thuộc có thể khác biệt; chạy install.sh và tự cài thủ công các gói thiếu nếu lỗi |
| ❌ **Chưa hỗ trợ** | NixOS, Alpine, Gentoo, các hệ thống khác | N/A | Không có sẵn trong quản lý gói — cần cài gói phụ thuộc thủ công rồi chạy `cargo build --release` |

> **⚠️ Lưu ý về biểu tượng khay hệ thống:** Môi trường GNOME (Ubuntu) và Cinnamon (Mint) cần có phần mềm theo dõi StatusNotifier để hiển thị khay hệ thống:
> - Ubuntu: `sudo apt install gnome-shell-extension-appindicator`
> - Mint: Đã được tích hợp sẵn, hoạt động ngay sau khi cài đặt

---

## Cài đặt

### Cài đặt nhanh bằng một câu lệnh

Áp dụng cho tất cả các distro được đánh dấu ✅ **Hỗ trợ tốt** ở trên. Kịch bản cài đặt sẽ tự động nhận diện trình quản lý gói của hệ thống:

**Từ GitHub (khuyên dùng):**
```bash
git clone https://github.com/vndangkhoa/vietc.git /tmp/vietc \
  && cd /tmp/vietc && sudo ./install.sh
```

**Từ Forgejo (máy chủ riêng):**
```bash
git clone https://git.khoavo.myds.me/vndangkhoa/vietc.git /tmp/vietc \
  && cd /tmp/vietc && sudo ./install.sh
```

Kịch bản sẽ tự động cài các thư viện phụ thuộc, biên dịch mã nguồn, cài đặt chương trình vào `/usr/bin/`, thiết lập phân quyền cho uinput qua udev rules, và thêm người dùng hiện tại vào nhóm `input`.

**Sau khi cài đặt:** Đăng xuất (Log out) và đăng nhập lại hệ thống, sau đó khởi chạy ứng dụng `vietc-tray` từ menu ứng dụng.

### Gỡ cài đặt nhanh bằng một câu lệnh

**Từ GitHub:**
```bash
curl -sSL https://raw.githubusercontent.com/vndangkhoa/vietc/main/uninstall.sh | sudo bash
```

**Từ Forgejo:**
```bash
curl -sSL https://git.khoavo.myds.me/vndangkhoa/vietc/raw/branch/main/uninstall.sh | sudo bash
```

### Biên dịch & Chạy thủ công

```bash
# Cài đặt các thư viện phụ thuộc
sudo apt install git curl build-essential pkg-config \
  libx11-dev libxtst-dev libevdev-dev libdbus-1-dev libwayland-dev wl-clipboard

# Kích hoạt tính năng hỗ trợ tiếp cận (Ubuntu Wayland — dùng cho nhận diện mật khẩu)
gsettings set org.gnome.desktop.a11y.applications screen-reader-enabled true

# Biên dịch mã nguồn
git clone https://github.com/vndangkhoa/vietc.git
cd vietc
cargo build --release

# Chạy (Hệ điều hành Mint — không cần quyền sudo cho uinput)
./target/release/vietc

# Chạy (Hệ điều hành Ubuntu — cần quyền sudo để bắt sự kiện bàn phím)
sudo ./target/release/vietc
```

---

## Cấu hình

Tệp cấu hình đặt tại: `~/.config/vietc/config.toml` hoặc `./vietc.toml`

```toml
input_method = "vni"            # "vni" hoặc "telex"
toggle_key = "space"            # Ctrl+Space để bật/tắt gõ tiếng Việt
toggle_method_key = "shift"     # Ctrl+Shift để chuyển đổi VNI/Telex
start_enabled = true            # Mặc định bật tiếng Việt khi khởi động
grab = true                     # Độc chiếm bàn phím (evdev)

[auto_restore]
enabled = false                 # Tự động hoàn tác từ tiếng Anh gõ nhầm (mặc định tắt)
trigger_keys = ["space", "escape"]

[password_detection]
enabled = true
check_atspi2 = true
check_window_title = true
title_keywords = ["password", "passphrase", "secret", "mật khẩu", "sudo"]
password_apps = ["pinentry", "pinentry-gtk-2", "pinentry-qt",
  "lxqt-sudo", "kdesudo", "gksudo",
  "polkit-gnome-authentication-agent-1",
  "kwallet", "gnome-keyring", "ssh-askpass"]

[app_state]
enabled = true
english_apps = ["code", "vim"]
vietnamese_apps = ["telegram", "discord", "firefox"]
bypass_apps = ["steam"]
terminal_apps = ["kitty", "alacritty", "gnome-terminal", "konsole", "foot",
  "wezterm", "st", "urxvt", "xterm"]
terminal_input_method = "vni"   # Tự động chuyển sang VNI khi chạy trong terminal

[macros]
ko = "không"
dc = "được"
vs = "với"
```

### Sử dụng trong Terminal

Viet+ hoạt động cực kỳ mượt mà trong các môi trường terminal. Khi bạn sử dụng một terminal (ví dụ: gnome-terminal, kitty), bộ gõ tiếng Việt sẽ tự động áp dụng phương thức gõ được thiết lập tại cấu hình `terminal_input_method` trong mục `[app_state]`.

Các terminal được hỗ trợ sẵn: `kitty`, `alacritty`, `gnome-terminal`, `konsole`, `foot`, `wezterm`, `st`, `urxvt`, `xterm`

Gõ tiếng Việt trực tiếp — không có thanh gạch chân khó chịu, không lặp từ. Chỉ cần gõ phím số VNI hoặc phím chữ Telex và ký tự Unicode sẽ xuất hiện ngay lập tức!

---

## Kiến trúc hệ thống

```
vietc/
├── engine/                  # Bộ xử lý chuyển đổi chữ tiếng Việt (chuyển đổi từ bamboo-core)
├── protocol/                # Thư viện bắt và giả lập sự kiện bàn phím
│   ├── uinput_monitor.rs    # Giả lập qua /dev/uinput (chính)
│   ├── x11_inject.rs        # Giả lập qua XTest (dự phòng)
│   ├── x11_capture.rs       # Bắt phím qua XRecord
│   └── wayland_im.rs        # Giao thức Wayland IM (đang phát triển)
├── daemon/                  # Tiến trình nền chính điều khiển bộ gõ
│   ├── main.rs              # Vòng lặp sự kiện, chiếm quyền bàn phím, xử lý tín hiệu
│   ├── config.rs            # Tải tệp cấu hình TOML + tự động cập nhật cấu hình nóng
│   ├── app_state.rs         # Quản lý bộ nhớ trạng thái theo ứng dụng + nhận diện mật khẩu
│   ├── password_detector.rs # Nhận diện trường mật khẩu qua AT-SPI2 D-Bus
│   └── display.rs           # Nhận diện máy chủ đồ họa X11/Wayland/Compositor
├── ui/                      # Biểu tượng khay hệ thống (sử dụng ksni)
│   └── tray.rs              # Khay hiển thị chế độ VN/TLX/EN
├── cli/                     # Công cụ dòng lệnh kiểm thử bộ xử lý tiếng Việt
└── uinputd/                 # Tiến trình đặc quyền quản lý socket uinput
```

---

## Lộ trình phát triển

### Phiên bản v0.1.8
- Hỗ trợ giao thức nhập liệu Wayland (`zwp_input_method_v2`) — loại bỏ việc dùng clipboard và tranh chấp Backspace, khắc phục triệt để lỗi mất khoảng trắng.
- Cơ chế giám sát tiêu điểm AT-SPI2 hướng sự kiện (đăng ký nhận sự kiện tiêu điểm từ a11y thay vì liên tục truy vấn).

### Phiên bản v0.1.9
- Tự động hóa việc đóng gói tệp tin `.deb` bằng GitHub Actions CI.
- Khôi phục hỗ trợ Flatpak cho các hệ điều hành bất biến (immutable distros).

---

## Giấy phép sử dụng

Giấy phép MIT — xem chi tiết tại tệp tin [LICENSE](LICENSE).

---

<p align="center">
  <sub>Được phát triển với tình yêu dành cho cộng đồng Linux Việt Nam</sub>
</p>
