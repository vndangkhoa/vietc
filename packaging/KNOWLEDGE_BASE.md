# 🧠 Viet+ Distribution Knowledge Base & Troubleshooting Guide

Tài liệu này tổng hợp toàn bộ kiến thức kỹ thuật, các lỗi đặc thù (gotchas) và giải pháp đóng gói phân phối **Viet+** cho **Arch Linux** và **Ubuntu/Debian (Launchpad PPA)**.

---

## 📑 Mục lục
1. [Kiến trúc phân phối đa nền tảng](#1-kiến-trúc-phân-phối-đa-nền-tảng)
2. [Ubuntu Launchpad PPA: Các lỗi thực tế & Giải pháp](#2-ubuntu-launchpad-ppa-các-lỗi-thực-tế--giải-pháp)
3. [Arch Linux: Kho Pacman riêng & AUR](#3-arch-linux-kho-pacman-riêng--aur)
4. [Quy trình phát hành phiên bản mới (Release Cheatsheet)](#4-quy-trình-phát-hành-phiên-bản-mới-release-cheatsheet)

---

## 1. Kiến trúc phân phối đa nền tảng

| Kênh phân phối | Định dạng | Phương thức cài đặt | Cơ chế tự động hóa |
| :--- | :--- | :--- | :--- |
| **Arch Custom Repo** | `.pkg.tar.zst` + `vietc.db` | `sudo pacman -S vietc` | [`.github/workflows/arch-repo.yml`](../.github/workflows/arch-repo.yml) qua GitHub Pages |
| **Arch AUR (Source)** | `PKGBUILD` | `yay -S vietc` | [`.github/workflows/aur.yml`](../.github/workflows/aur.yml) |
| **Arch AUR (Binary)** | `PKGBUILD` | `yay -S vietc-bin` | [`.github/workflows/aur.yml`](../.github/workflows/aur.yml) |
| **Ubuntu PPA** | `.dsc` + `.orig.tar.gz` | `sudo apt install vietc` | [`packaging/ppa/build-source-package.sh`](ppa/build-source-package.sh) |
| **Debian / Ubuntu .deb** | `.deb` | `sudo dpkg -i vietc_*.deb` | [`packaging/deb/build-deb.sh`](deb/build-deb.sh) |
| **Generic Tarball** | `.tar.gz` | `./install.sh` | [`packaging/build-tarball.sh`](build-tarball.sh) |

---

## 2. Ubuntu Launchpad PPA: Các lỗi thực tế & Giải pháp

### 🚫 Lỗi 1: Máy chủ Launchpad không có mạng Internet (Offline Build Farm)
* **Hiện tượng:** `error: failed to download from https://index.crates.io/config.json - Could not resolve host`.
* **Nguyên nhân:** Máy chủ Canonical cách ly mạng 100% trong lúc build để đảm bảo an toàn.
* **Giải pháp:**
  1. Chạy `cargo vendor --sync ui/Cargo.toml vendor` để tải sẵn toàn bộ mã nguồn của các thư viện vào thư mục `vendor/`.
  2. Tạo file `.cargo/config.toml` và `ui/.cargo/config.toml` trỏ nguồn `crates-io` về `vendor`.
  3. Chạy `cargo build --offline --release` trong `debian/rules`.

---

### 🚫 Lỗi 2: Tính bất biến của file `.orig.tar.gz` (Source Tarball Immutability)
* **Hiện tượng:** Email từ chối `Rejected: File vietc_0.1.9.orig.tar.gz already exists, but uploaded version has different contents`.
* **Nguyên nhân:** Launchpad không cho phép ghi đè tệp `.orig.tar.gz` của cùng 1 phiên bản nếu nội dung/checksum thay đổi.
* **Giải pháp:** Khi cần thay đổi cấu trúc mã nguồn (như thêm thư viện vendor), phải tăng số phiên bản upstream (ví dụ từ `0.1.9` lên `0.1.9.1`).

---

### 🚫 Lỗi 3: Không tương thích `Cargo.lock version = 4`
* **Hiện tượng:** `error: failed to parse lock file: lock file version 4 requires -Znext-lockfile-bump`.
* **Nguyên nhân:** Ubuntu 24.04 và 22.04 sử dụng Cargo 1.75 / 1.70. Phiên bản này chỉ đọc được `Cargo.lock version = 3`.
* **Giải pháp:**
  Tự động ép phiên bản lockfile về `version = 3`:
  ```bash
  sed -i 's/^version = 4/version = 3/' Cargo.lock ui/Cargo.lock
  ```

---

### 🚫 Lỗi 4: Thư viện crate dùng Rust `edition = "2024"`
* **Hiện tượng:** `failed to parse manifest at /vendor/getrandom/Cargo.toml: feature edition2024 is required`.
* **Nguyên nhân:** Các bản cập nhật mới của một số crate trên crates.io (`hashbrown`, `indexmap`, `getrandom`) dùng cú pháp Rust 2024. Cargo 1.75 trên Ubuntu không nhận diện được.
* **Giải pháp:**
  Tự động patch toàn bộ các file `Cargo.toml` trong `vendor/` về `edition = "2021"` và xóa hash kiểm tra:
  ```bash
  find vendor/ -name "Cargo.toml*" -exec sed -i 's/edition = "2024"/edition = "2021"/g' {} +
  find vendor/ -name ".cargo-checksum.json" -exec sed -i 's/"Cargo.toml":"[^"]*"/"Cargo.toml":null/g' {} +
  ```

---

### 🚫 Lỗi 5: Module kiểm thử riêng biệt gây phình thư viện
* **Hiện tượng:** Thư mục `vk/` chứa các thư viện thử nghiệm riêng làm tăng dung lượng và gây xung đột phiên bản.
* **Giải pháp:** Loại bỏ thư mục `vk/` trong script đóng gói PPA (`rm -rf "$SOURCE_DIR/vk"`).

---

## 3. Arch Linux: Kho Pacman riêng & AUR

### Kho Pacman riêng (Custom Repository)
* **Cấu trúc dữ liệu:**
  ```text
  dist/arch/x86_64/
  ├── vietc-0.1.9-1-x86_64.pkg.tar.zst  # Gói cài đặt nhị phân
  ├── vietc.db -> vietc.db.tar.gz       # Cơ sở dữ liệu repo
  ├── vietc.db.tar.gz
  ├── vietc.files -> vietc.files.tar.gz # Danh sách file
  └── vietc.files.tar.gz
  ```
* **Lệnh tạo cục bộ:**
  `make arch-repo` (sử dụng container Arch Linux qua Docker để biên dịch và chạy `repo-add`).
* **Triển khai tự động:** GitHub Actions đẩy nội dung thư mục `dist/arch/` lên nhánh `gh-pages` để người dùng có thể tải qua đường dẫn `https://vndangkhoa.github.io/vietc/arch/$arch`.

---

## 4. Quy trình phát hành phiên bản mới (Release Cheatsheet)

Khi bạn muốn phát hành một phiên bản mới (ví dụ `0.2.0`):

### Cách 1: Tự động hóa toàn bộ bằng 1 lệnh duy nhất (Khuyên dùng)

```bash
# Build tất cả (.deb, tarball, Arch Pacman repo, AUR) và TỰ ĐỘNG UPLOAD lên Launchpad PPA:
make release-ppa
# Hoặc chỉ định phiên bản:
bash packaging/release.sh 0.2.0 --upload-ppa
```

### Cách 2: Các bước đẩy lên GitHub

```bash
# 1. Commit toàn bộ thay đổi
git add .
git commit -m "Release v0.2.0"
git push origin main

# 2. Tạo và đẩy Release Tag
git tag v0.2.0
git push origin v0.2.0
```

> **Sau khi đẩy Git Tag:**
> * GitHub Actions sẽ tự động biên dịch và cập nhật kho Pacman trên **GitHub Pages**.
> * Tự động đồng bộ và đẩy lên **Arch Linux AUR**.
> * Đính kèm file `.deb`, `.tar.gz`, `.pkg.tar.zst` vào **GitHub Releases**.
