# 📦 Viet+ Packaging & Distribution Guide

Tài liệu hướng dẫn chi tiết cách đóng gói và phân phối **Viet+** cho các bản phân phối Linux: **Arch Linux (AUR & Pacman Repo)**, **Ubuntu/Debian (Launchpad PPA / .deb)**, và **Generic Tarball**.

> 💡 **Tài liệu chuyên sâu:** Xem toàn bộ kiến thức kỹ thuật và cách xử lý lỗi tại [**Knowledge Base & Troubleshooting Guide**](KNOWLEDGE_BASE.md).

---

## ⚡ Phát hành phiên bản mới bằng 1 lệnh duy nhất:
```bash
# Đóng gói tất cả và tự động upload lên Ubuntu Launchpad PPA:
make release-ppa
```

---

## 📑 Mục lục
1. [Tự động hóa phát hành (Master Release Script)](#tự-động-hóa-phát-hành)
2. [Arch Linux — Kho Pacman riêng (Custom Repository)](#1-arch-linux--kho-pacman-riêng-custom-repository)
3. [Arch Linux — Arch User Repository (AUR)](#2-arch-linux--arch-user-repository-aur)
4. [Ubuntu & Debian — Launchpad PPA & .deb](#3-ubuntu--debian--launchpad-ppa--deb)
5. [Generic Linux Tarball](#4-generic-linux-tarball)
6. [Lý do không sử dụng Ubuntu Snap Store](#5-lý-do-không-sử-dụng-ubuntu-snap-store)

---

## 1. Arch Linux — Kho Pacman riêng (Custom Repository)

Kho Pacman riêng giúp người dùng **Arch Linux, CachyOS, Manjaro, EndeavourOS** cài đặt các gói `.pkg.tar.zst` dựng sẵn bằng lệnh `pacman` mà **không cần AUR helper và không mất thời gian biên dịch**.

### Cách người dùng Arch thêm repo:
Thêm các dòng sau vào cuối file `/etc/pacman.conf`:
```ini
[vietc]
SigLevel = Optional TrustAll
Server = https://vndangkhoa.github.io/vietc/arch/$arch
```

Cập nhật và cài đặt:
```bash
sudo pacman -Syu vietc
```

### Cách tạo và cập nhật kho Pacman:
Chạy lệnh từ thư mục dự án (sử dụng Docker container Arch Linux chính thức):
```bash
make arch-repo
```
Lệnh này sẽ tự động sinh gói `.pkg.tar.zst` và cơ sở dữ liệu `vietc.db` / `vietc.files` trong thư mục `dist/arch/x86_64/`.

Workflow [`.github/workflows/arch-repo.yml`](../.github/workflows/arch-repo.yml) sẽ tự động biên dịch và triển khai lên **GitHub Pages (`gh-pages`)** mỗi khi bạn push code lên `main` hoặc tạo tag release!

---

## 2. Arch Linux — Arch User Repository (AUR)

Viet+ cung cấp sẵn 2 package chuẩn cho Arch Linux:
* **`vietc`** ([packaging/aur/PKGBUILD](aur/PKGBUILD)): Biên dịch từ mã nguồn (source code) với đầy đủ hỗ trợ Wayland & X11.
* **`vietc-bin`** ([packaging/aur-bin/PKGBUILD](aur-bin/PKGBUILD)): Tải file nhị phân dựng sẵn từ GitHub Releases (cài đặt siêu nhanh không cần cài Rust).

### Đăng ký tài khoản AUR & SSH Key

1. Đăng ký tài khoản tại [aur.archlinux.org](https://aur.archlinux.org/register).
2. Thêm SSH Public Key của bạn vào phần **My Account** -> **SSH Public Key**.
3. Cấu hình SSH (`~/.ssh/config`):
   ```ssh
   Host aur.archlinux.org
       IdentityFile ~/.ssh/id_ed25519
       User aur
   ```

### Quy trình đẩy lên AUR thủ công

#### Bước 1: Cập nhật version và checksums
Chạy lệnh sau trong thư mục dự án:
```bash
make aur
# hoặc: bash packaging/aur/update-aur.sh 0.1.9
```
Lệnh này sẽ tự động tải release tag hoặc tarball, tính toán SHA256 checksums, và tạo file `.SRCINFO` (nếu có `makepkg`).

#### Bước 2: Khởi tạo/Clone repo trên AUR

```bash
# Đối với package vietc (source)
git clone ssh://aur@aur.archlinux.org/vietc.git /tmp/vietc-aur
cp packaging/aur/PKGBUILD packaging/aur/.SRCINFO /tmp/vietc-aur/
cd /tmp/vietc-aur
git add PKGBUILD .SRCINFO
git commit -m "Initial release v0.1.9"
git push -u origin master

# Đối với package vietc-bin (binary)
git clone ssh://aur@aur.archlinux.org/vietc-bin.git /tmp/vietc-bin-aur
cp packaging/aur-bin/PKGBUILD packaging/aur-bin/.SRCINFO /tmp/vietc-bin-aur/
cd /tmp/vietc-bin-aur
git add PKGBUILD .SRCINFO
git commit -m "Initial release v0.1.9"
git push -u origin master
```

Sau khi push, người dùng Arch Linux / Manjaro / CachyOS / EndeavourOS có thể cài đặt ngay:
```bash
yay -S vietc
# hoặc
yay -S vietc-bin
# hoặc dùng paru:
paru -S vietc
```

---

### Tự động hóa qua GitHub Actions

Dự án đã cấu hình sẵn workflow [`.github/workflows/aur.yml`](../.github/workflows/aur.yml). Mỗi khi bạn push tag `v*` (ví dụ `v0.1.9`), workflow sẽ tự động đẩy lên AUR.

Để kích hoạt, chỉ cần thêm 3 Secrets vào GitHub Repo Settings (**Settings -> Secrets and variables -> Actions**):
* `AUR_SSH_PRIVATE_KEY`: Private SSH Key dùng để push AUR.
* `AUR_USERNAME`: Tên tài khoản trên AUR (ví dụ: `vndangkhoa`).
* `AUR_EMAIL`: Email tài khoản AUR.

---

## 2. Ubuntu & Debian — Launchpad PPA & .deb

### Tạo Launchpad PPA

1. Đăng ký tài khoản trên [launchpad.net](https://launchpad.net).
2. Tạo một PPA mới (ví dụ: `ppa:khoavo93/vietc`).
3. Thêm GPG Key vào Launchpad để ký các gói source package.

### Build và upload Source Package

Chạy script tạo source package:
```bash
make ppa
# hoặc: bash packaging/ppa/build-source-package.sh 0.1.9 noble
```

Ký và tải lên Launchpad:
```bash
cd packaging/ppa/build_noble/vietc-0.1.9
debuild -S -sa -kYOUR_GPG_KEY_ID
dput ppa:khoavo93/vietc ../vietc_0.1.9-1~ubuntunoble1_source.changes
```
Launchpad sẽ tự động biên dịch trên server của Canonical cho các kiến trúc `amd64`, `arm64`, `armhf`, v.v.

### Cài đặt cho người dùng Ubuntu

Người dùng Ubuntu chỉ cần chạy:
```bash
sudo add-apt-repository ppa:khoavo93/vietc
sudo apt update
sudo apt install vietc
```

---

## 3. Generic Linux Tarball

Tạo gói `.tar.gz` chứa toàn bộ binaries, desktop files, udev rules và `install.sh`:
```bash
make tarball
# Tệp xuất ra tại: target/dist/vietc_0.1.9_linux_amd64.tar.gz
```

---

## 4. Lý do không sử dụng Ubuntu Snap Store

Bộ gõ tiếng Việt (IME) là một thành phần hệ thống cấp thấp (Low-level system input handler) với các đặc thù:
1. **Lắng nghe phím toàn cục:** Cần đọc trực tiếp `/dev/input/event*` hoặc bắt sự kiện qua X11/Wayland protocols.
2. **Inject phím ảo:** Cần truy cập `/dev/uinput` hoặc `wtype` (zwp_virtual_keyboard_v1).
3. **Cơ chế bảo mật Strict Sandbox của Snap:** Snap cô lập hoàn toàn quyền truy cập bàn phím để phòng chống keylogger. Vì vậy, một snap `strict` không thể hoạt động như một bộ gõ hệ thống.
4. **Quyền `classic` confinement:** Snap Store yêu cầu xét duyệt thủ công rất khắt khe và thường từ chối cấp quyền `classic` cho IME.

Do đó, **Launchpad PPA (`.deb`)** và **AUR** là hai kênh phân phối chính thức, chuẩn mực và tối ưu nhất trên Linux.
