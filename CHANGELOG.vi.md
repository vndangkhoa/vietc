# Nhật ký thay đổi (Changelog)

<p align="center">
  <a href="CHANGELOG.md">English</a>
</p>

## Chưa phát hành (Unreleased)

### Hỗ trợ Bản phân phối (Distro Support)

- **Bảng hỗ trợ Distro**: README hiện đã liệt kê các distro hỗ trợ tốt (Ubuntu, Debian, Mint, Pop!_OS, elementary, Zorin, Neon, Fedora, RHEL, CentOS, Arch, Manjaro), có thể hỗ trợ (openSUSE, Solus, Void), và chưa hỗ trợ (NixOS, Alpine, Gentoo).
- **libwayland-dev** được thêm vào install.sh cho tất cả các họ distro (trước đây bị thiếu — gây lỗi biên dịch trên các hệ thống chỉ có X11 như Linux Mint).
- **libwayland-client0** được thêm vào các gói phụ thuộc runtime (trước đây bị thiếu — gây lỗi "cannot open shared object file" trên Mint).
- **Sửa lỗi chính tả cấu hình**: `mặt khẩu` → `mật khẩu` trong cấu hình mặc định và README.

### Tài liệu hướng dẫn (Documentation)

- Thêm mục **Roadmap** vào README (v0.1.8: Giao thức Wayland IM, AT-SPI2 hướng sự kiện; v0.1.9: CI, Flatpak).
- Loại bỏ **RELEASE_CHECKLIST.md** (quy trình phát hành hiện được ghi nhận trong nội dung các commit phát hành).

---

## v0.1.7 (01-07-2026)

### Tự động nhận diện mật khẩu (Password Auto-Detection)

- **Tích hợp AT-SPI2 D-Bus**: Truy vấn `org.a11y.atspi.Accessible.GetRole` trên a11y bus (không phải session bus) để phát hiện các trường mật khẩu. Hoạt động trên các hộp thoại mật khẩu GUI và các ứng dụng có bật hỗ trợ tiếp cận (a11y).
- **Phát hiện sudo qua cây tiến trình**: Quét `pstree` để tìm các tiến trình `sudo`/`passwd` — tự động tắt tiếng Việt khi có yêu cầu sudo xuất hiện trong terminal.
- **Dự phòng tiêu đề cửa sổ**: Các cửa sổ có tiêu đề chứa "password", "sudo", "mật khẩu" sẽ tự động chuyển sang chế độ gõ tiếng Anh.
- **Dự phòng lớp cửa sổ (Window class)**: Nhận diện các hộp thoại mật khẩu phổ biến (pinentry, polkit, kwallet) thông qua danh sách ứng dụng `password_apps` trong cấu hình.
- **Kiểm tra định kỳ**: Đánh giá lại trạng thái trường mật khẩu sau mỗi 30 phím gõ (giúp phát hiện kịp thời các prompt nhập mật khẩu xuất hiện trong terminal).

### Phương thức gõ Telex (Telex Input Method)

- **Hỗ trợ đầy đủ Telex**: Cả hai phương thức gõ VNI và Telex hiện đã được hỗ trợ toàn diện. Chuyển đổi nhanh qua Ctrl+Shift hoặc menu khay hệ thống "Input Method > Telex / VNI".
- **Tệp lưu phương thức gõ** (`~/.config/vietc/method`): Tiến trình nền (daemon) ghi phương thức gõ hiện tại; khay hệ thống đọc tệp này để hiển thị icon tương ứng.
- **Biểu tượng khay hệ thống**: Màu đỏ "VN" cho VNI, màu xanh dương "TLX" cho Telex, màu xám "EN" cho chế độ tiếng Anh.
- **Cấu hình**: Phím nóng `toggle_method_key = "shift"` dùng để thiết lập tổ hợp phím đổi phương thức gõ.

### Hỗ trợ GNOME/Wayland (GNOME/Wayland Support)

- **Tích hợp D-Bus của GNOME Shell**: Truy vấn `org.gnome.Shell.Eval` để lấy thông tin về lớp cửa sổ (window class), ID, tiêu đề và PID của ứng dụng đang hoạt động — giải pháp thay thế hoàn hảo trên Wayland GNOME nơi xdotool/xprop không khả dụng.
- **Chuỗi nhận diện cửa sổ**: GNOME Shell D-Bus → xprop → wlrctl → xdotool → wmctrl → /proc — hoạt động ổn định trên mọi môi trường máy tính.
- **Nhận diện Compositor**: Tự động phát hiện GNOME/Mutter qua `pgrep gnome-shell` và `XDG_CURRENT_DESKTOP`.
- **Thư viện phụ thuộc**: Sử dụng thư viện `dbus` (0.9) để giao tiếp với AT-SPI2 và GNOME Shell D-Bus.

### Chiếm quyền bàn phím an toàn (Keyboard Grab Safety)

- **Sử dụng sigaction không có SA_RESTART**: Tổ hợp Ctrl+C và tín hiệu SIGTERM hiện đã có thể ngắt lệnh đọc evdev đang bị chặn, giải phóng quyền chiếm giữ bàn phím trước khi thoát.
- **Tự động tải uinput**: Bộ giả lập sẽ tự chạy lệnh `modprobe uinput` trước khi mở `/dev/uinput`.
- **Xử lý EINTR**: Bắt các cuộc gọi hệ thống bị ngắt quãng và tiến hành kiểm tra lại cờ tín hiệu hệ thống.
- **Thời gian chờ an toàn 30 giây**: Tự động giải phóng quyền chiếm giữ bàn phím nếu không nhận được sự kiện nào sau 30 giây (tránh việc người dùng bị khóa bàn phím vĩnh viễn khi bộ gõ gặp sự cố).

### Clipboard & Giả lập nhập liệu (Clipboard & Injection)

- **Tối ưu hóa `wl-copy --paste-once`**: Giữ tiến trình clipboard hoạt động cho đến khi thao tác dán được thực hiện xong, loại bỏ hoàn toàn độ trễ từ 300-900ms trên môi trường Wayland/GNOME.
- **Tắt log SelectionRequest trên X11**: Loại bỏ hoàn toàn các dòng log rác liên quan đến clipboard trong terminal.
- **Ưu tiên uinput**: Giả lập qua uinput luôn được ưu tiên hơn so với giả lập qua X11 XTest.

### Thay đổi cấu hình (Config Changes)

- **Mặc định tắt tính năng tự động khôi phục từ tiếng Anh (auto-restore)**: Tránh việc lặp hoặc mất dấu trên các từ tiếng Việt hợp lệ. Người dùng có thể kích hoạt lại nếu muốn bằng cách đặt `[auto_restore] enabled = true`.

### Cải tiến dòng lệnh (CLI Enhancements)

- **Chuyển tiếp ký tự**: Tất cả các ký tự đều hiển thị trên đầu ra (thay vì chỉ hiển thị các sự kiện chuyển đổi của bộ xử lý Bamboo).
- **Màn hình hiển thị**: Các phím xóa ngược (backspace) được áp dụng trực quan để mang lại trải nghiệm giống thực tế nhất.
- **Đặt lại trạng thái**: Mỗi dòng nhập mới sẽ bắt đầu với trạng thái bộ xử lý hoàn toàn sạch.
- **Lệnh mới**: Thêm các lệnh hỗ trợ `:help`, `:status`, `:vi`, `:en`, `:ar on|off`, `:macros`, `:macro add/rm/clear`, `:events`.

### Sửa lỗi (Bug Fixes)

- **Lỗi lặp dấu cách khi bật/tắt bằng Ctrl+Space**: Chuyển tiếp phím thô hiện đã kiểm tra trạng thái hoạt động của bộ gõ.
- **Khóa một phiên chạy (Single-instance lock)**: Ghi PID vào tệp khóa; tự động phát hiện và dọn dẹp các tệp khóa cũ khi daemon tắt không bình thường.
- **Dự phòng xprop/wmctrl**: Nhận diện cửa sổ vẫn hoạt động tốt ngay cả khi hệ thống không cài đặt `xdotool`.
- **Kết nối AT-SPI2 a11y bus**: Sửa lỗi kết nối nhầm vào session bus; hiện đã kết nối chính xác vào a11y bus riêng biệt.
- **Đặt lại trạng thái bộ xử lý giữa các dòng nhập trong CLI**.

---

## v0.1.6 (29-06-2026)

### Ưu tiên giả lập uinput (uinput-First Injection)

- **Đảo ngược độ ưu tiên giả lập**: Giả lập qua uinput (`/dev/uinput`) trở thành phương thức giả lập nhập liệu chính trên X11, trong khi XTest chỉ đóng vai trò dự phòng.
- **Sửa mã phím (keycode) X11 XTest**: Áp dụng độ lệch (offset) +8 cho tất cả các mã phím evdev để đảm bảo tương thích với XTest.
- **Sửa lỗi xóa ngược trong `paste_via_clipboard()`**: Khắc phục lỗi gửi nhầm mã phím 14 (tương ứng với số "5") thành mã phím 22 (phím Backspace).

### Nhận diện chuyển đổi cửa sổ (Window-Switch Detection)

- **Xác thực ID cửa sổ trên mỗi phím gõ**: Loại bỏ khoảng thời gian bảo vệ 100ms — nhận diện ngay cả khi chuyển cửa sổ cực nhanh dưới 100ms.

### Phương thức nhập liệu (Input Method)

- **Tạm ẩn Telex trên khay hệ thống**: Hiển thị màu xám đi kèm ghi chú "(phiên bản tiếp theo)". Chỉ có phương thức gõ VNI hoạt động ở bản này.
- **Đổi phương thức gõ mặc định** thành `"vni"`.

### Đóng gói (Packaging)

- **Gỡ bỏ Flatpak và AppImage**: Hiện tại chỉ duy trì và phân phối gói cài đặt `.deb`.
- **Cải tiến postinst**: Tự động dọn dẹp tệp tin cũ và cấu hình lỗi thời; hiển thị thông báo yêu cầu đăng xuất để áp dụng thay đổi.

---

## v0.1.5 (29-06-2026)

### Đặt lại bộ gõ khi chuyển cửa sổ (Window-Switch Engine Reset)

- **Đặt lại trạng thái bộ gõ khi chuyển cửa sổ**: Khi Alt+Tab giữa các ứng dụng, bộ đệm ký tự của bộ gõ sẽ được xóa sạch. Tránh tình trạng ký tự gõ ở ứng dụng cũ áp dụng quy tắc gõ dấu sang ứng dụng mới gây lỗi hiển thị.
- **Bỏ tính năng ghi nhận `last_key_time` cho phím điều hướng/bổ trợ**: Các phím bổ trợ đơn thuần (Alt, Ctrl, Shift) không còn làm mới bộ đếm thời gian, giúp việc kiểm tra cửa sổ bằng xprop kích hoạt chính xác sau khi chuyển đổi ứng dụng.

### Nhận diện cửa sổ hoạt động (Active Window Detection)

- **Dự phòng xprop**: Thử gọi `xdotool` trước, sau đó tự động chuyển sang `xprop -root _NET_ACTIVE_WINDOW` (có sẵn trong `x11-utils`). Hoạt động ổn định dưới quyền sudo kể cả khi không cài `xdotool`.

### Dọn dẹp mã nguồn (Code Cleanup)

- **Gỡ bỏ khoảng 400 dòng mã không an toàn (unsafe) không sử dụng**: Xóa toàn bộ khối quản lý chia sẻ trạng thái clipboard X11. Loại bỏ hoàn toàn các cảnh báo `#[warn(dead_code)]` và `#[warn(static_mut_refs)]`.
- **Xóa mã chết trong bộ gõ**: Loại bỏ các phương thức không dùng đến trong bộ xử lý `BambooEngine` và `InputMethodRules`.
- **Ghi nhật ký vận hành**: Gỡ bỏ các lệnh `eprintln!` in thông tin theo từng phím gõ trong vòng lặp evdev và luồng dán uinput. Chỉ giữ lại các bản ghi quan trọng (khởi động, lỗi, chuyển cửa sổ) ghi ra stderr và tệp log.

### Biên dịch Flatpak & Khay hệ thống (Flatpak Build & System Tray)

- **Tích hợp khay hệ thống** (`vietc-tray` viết bằng thư viện ksni/DBus) vào trong gói cài đặt Flatpak. Khay hệ thống sẽ tự khởi chạy daemon và hiển thị trạng thái hiện tại.
- **Lối tắt Menu ứng dụng**: Bộ gõ hiển thị đầy đủ khi tìm kiếm từ khóa **"Viet+"** trên khay hệ thống.
- **Bỏ hộp thoại mật khẩu khi chạy Flatpak**: Skip sudo khi ứng dụng chạy trong Flatpak do Flatpak đã có sẵn quyền truy cập thiết bị thông qua cờ `--device=all`.

### Nhận diện cửa sổ hoạt động (Sửa lỗi cho Flatpak)

- **Tự động gọi thư viện hệ thống X11** `libX11.so.6` thông qua `dlopen`: Đóng vai trò là phương án dự phòng thứ ba. Giải pháp này giúp nhận diện cửa sổ hoạt động bình thường bên trong sandbox Flatpak nơi `xdotool`/`xprop` bị chặn quyền truy cập.

### Chế độ mặc định (Default Mode)

- **Mặc định bật bộ gõ**: `start_enabled` hiện mặc định là `true` — chế độ tiếng Việt sẽ kích hoạt ngay khi mở ứng dụng.

---

## v0.1.4 (28-06-2026)

### Đóng gói Flatpak (Flatpak Packaging)

- Cung cấp đầy đủ các thành phần đóng gói Flatpak bao gồm daemon, CLI, khay hệ thống, uinputd, và kịch bản khởi chạy.

### Tài liệu hướng dẫn (Documentation)

- Cập nhật README chi tiết về hướng dẫn cài đặt và biên dịch ứng dụng thông qua Flatpak.

### Clipboard & Giả lập nhập liệu (Clipboard & Injection)

- Khắc phục triệt để lỗi tranh chấp clipboard khi giả lập ký tự Unicode tiếng Việt.
- Thiết lập quy trình tự động đóng gói `.deb` và `.AppImage` trên mỗi lượt đẩy mã nguồn lên GitHub thông qua GitHub Actions.

### Kiểm thử (Tests)

- Hoàn thành **106 bài kiểm thử** đạt yêu cầu (72 bài cho nhân bộ gõ, 16 cho dòng lệnh CLI, 12 cho giao thức, 5 cho tính năng tự động phục hồi và 1 cho quy tắc đặt dấu thanh).

---

## v0.1.3 (26-06-2026)

- Sửa lỗi cụm nguyên âm `ua-horn`, lưu và khôi phục ngữ cảnh clipboard, tối ưu hóa các phím chức năng điều khiển.

---

## v0.1.2 (26-06-2026)

- Chuyển tiếp phím gốc khi xóa đệm, tự động khôi phục từ tiếng Anh gõ nhầm.
- Sửa quy tắc dấu cho các cụm `qu`/`gi`/`uê`/`uơ`, bỏ lặp phím tự động, tối ưu hóa phím Enter.

---

## v0.1.1 (26-06-2026)

- Khắc phục lỗi nuốt phím Telex khi gõ dấu, duy trì kết nối X11 liên tục.

---

## v0.1.0 (26-06-2026)

- Bản phát hành đầu tiên — chuyển đổi từ bamboo engine, bắt phím evdev, giả lập uinput.
