import React, { useState } from 'react';
import { motion, AnimatePresence } from 'motion/react';
import { Copy, Check, Terminal, Shield, Cpu, RefreshCw, Layers, GitBranch, Hammer, Info, Sparkles } from 'lucide-react';
import { SetupStep } from '../types';

type TabId = 'mint_ubuntu' | 'arch' | 'fedora' | 'dev';

export default function SetupGuide() {
  const [activeTab, setActiveTab] = useState<TabId>('mint_ubuntu');
  const [copiedText, setCopiedText] = useState<string | null>(null);

  const handleCopy = (text: string, id: string) => {
    navigator.clipboard.writeText(text);
    setCopiedText(id);
    setTimeout(() => setCopiedText(null), 2000);
  };

  const installSteps: Record<Exclude<TabId, 'dev'>, SetupStep[]> = {
    mint_ubuntu: [
      {
        id: 1,
        title: "Cài đặt VietC (một lệnh)",
        description: "Script tự động phát hiện distro, cài đặt dependencies, biên dịch và cài đặt toàn bộ daemon, công cụ vietcctl và ứng dụng khay hệ thống vietc-tray vào /usr/bin/.",
        command: `git clone https://github.com/vndangkhoa/vietc.git /tmp/vietc \\
  && cd /tmp/vietc && sudo ./install.sh`,
        notes: "Tự động: cấu hình IBus component, thiết lập nguồn gõ vietc duy nhất trên GNOME (ẩn chữ VI/en thừa), cài đặt biểu tượng khay EN/VN/TLX đa kích thước và tự khởi chạy khi đăng nhập."
      },
      {
        id: 2,
        title: "Kích hoạt & gõ tiếng Việt",
        description: "VietC tự động chạy và tích hợp trực tiếp vào bàn phím hệ thống. Sử dụng phím tắt Ctrl + Shift để xoay vòng giữa Tiếng Anh, VNI và Telex hoàn toàn mượt mà.",
        command: `# Xoay vòng kiểu gõ bất kỳ lúc nào:
# Nhấn: Ctrl + Shift (ENG ➔ VNI ➔ TELEX ➔ ENG)

# Kiểm tra trạng thái hiện tại bằng CLI:
vietcctl status`,
        notes: "Gõ trực tiếp vào tài liệu không có gạch chân (Zero Underline). Biểu tượng trên khay hệ thống (EN / VN / TLX) sẽ tự động đổi màu và chữ tương ứng."
      },
      {
        id: 3,
        title: "Gỡ cài đặt (Uninstall)",
        description: "Xoá hoàn toàn VietC khỏi hệ thống, bao gồm binary, service và udev rules. IBus sẽ được khởi động lại nếu trước đó bị thay thế.",
        command: `curl -sSL https://raw.githubusercontent.com/vndangkhoa/vietc/main/uninstall.sh | sudo bash`,
        notes: "Xoá /usr/bin/vietc-daemon, /usr/bin/vietcctl, /usr/bin/vietc-tray, /usr/share/ibus/component/vietc.xml và ~/.config/vietc/."
      }
    ],
    arch: [
      {
        id: 1,
        title: "Cài đặt VietC trên Arch",
        description: "Tự động clone, build và cài đặt VietC. Trên Arch X11, mặc định dùng evdev+uinput 0ms direct. Trên Arch GNOME Wayland cũng hỗ trợ IBus nếu bạn để auto_ibus=true.",
        command: `git clone https://github.com/vndangkhoa/vietc.git /tmp/vietc \\
  && cd /tmp/vietc && sudo ./install.sh
# ép IBus như Funput funput-ibus: sudo ./install.sh --ibus`,
        notes: "Hỗ trợ pacman, tự cài base-devel, libx11, libxkbcommon, wayland. Ép chế độ: --ibus (force IBus), --bamboo (dùng ibus-bamboo per-app), --grab (ép evdev)."
      },
      {
        id: 2,
        title: "Tuỳ chọn: ép chế độ",
        description: "VietC tự chọn đường dẫn tối ưu, nhưng bạn có thể ép buộc nếu cần test hoặc dùng wm đặc biệt (Sway/Hyprland hỗ trợ zwp_input_method_v2).",
        command: `# Ép dùng IBus engine (kể cả trên X11)
VIETC_FORCE_IBUS=1 vietc-daemon
# Hoặc sửa ~/.config/vietc/config.toml:
# auto_ibus = true
# ibus_engine = true`,
        notes: "Xem docs/wayland-rootless.md để hiểu thứ tự ưu tiên: IBus -> zwp_input_method_v2 -> evdev -> X11 keymap."
      },
      {
        id: 3,
        title: "Gỡ cài đặt",
        description: "Xoá VietC hoàn toàn khỏi hệ thống Arch.",
        command: `curl -sSL https://raw.githubusercontent.com/vndangkhoa/vietc/main/uninstall.sh | sudo bash`,
      }
    ],
    fedora: [
      {
        id: 1,
        title: "Cài đặt VietC trên Fedora",
        description: "Tự động cài đặt VietC trên Fedora/RHEL. Hỗ trợ dnf, tự cài Development Tools và thư viện Wayland/X11.",
        command: `git clone https://github.com/vndangkhoa/vietc.git /tmp/vietc \\
  && cd /tmp/vietc && sudo ./install.sh`,
        notes: "Tương tự Ubuntu nhưng dùng dnf. Trên Fedora GNOME Wayland cũng tự bật IBus engine nếu auto_ibus=true."
      },
      {
        id: 2,
        title: "Kích hoạt & kiểm tra",
        description: "Bật service và kiểm tra xem VietC đang dùng đường dẫn nào (IBus / evdev / X11).",
        command: `systemctl --user enable --now vietc.service
journalctl --user -u vietc.service --since today | grep -E "Display|IBus|evdev|X11"`,
        notes: "Kỳ vọng trên Fedora Wayland: [vietc] Wayland input method ... hoặc IBus engine mode auto-enabled. Toggle VN/EN: Ctrl+Space, đổi VNI/Telex: Ctrl+Shift."
      },
      {
        id: 3,
        title: "Gỡ cài đặt",
        description: "Xoá VietC hoàn toàn khỏi Fedora.",
        command: `curl -sSL https://raw.githubusercontent.com/vndangkhoa/vietc/main/uninstall.sh | sudo bash`,
      }
    ]
  };

  const devSteps: SetupStep[] = [
    {
      id: 1,
      title: "Clone mã nguồn",
      description: "Nhánh main chứa code mới nhất với fix Ubuntu IBus.",
      command: `git clone https://github.com/vndangkhoa/vietc.git
cd vietc`,
    },
    {
      id: 2,
      title: "Cài đặt Rust (nếu chưa có)",
      description: "Dùng rustup để cài Rust toolchain mới nhất (cần >=1.85).",
      command: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"`,
      notes: "Kiểm tra với 'rustc --version' và 'cargo --version'."
    },
    {
      id: 3,
      title: "Cài đặt hệ thống phụ thuộc",
      description: "Thư viện dev cho X11, evdev, dbus và Wayland (cần cho cả ba đường dẫn: IBus, zwp v2, evdev).",
      command: `sudo apt install build-essential pkg-config libx11-dev libxtst-dev \\
  libevdev-dev libdbus-1-dev libwayland-dev libxkbcommon-dev wl-clipboard
# Trên Fedora: sudo dnf install gcc pkgconfig libxkbcommon-devel libX11-devel libXtst-devel wayland-devel libevdev-devel dbus-devel
# Trên Arch: sudo pacman -S base-devel pkgconf libxkbcommon wayland libevdev dbus`,
      notes: "libxkbcommon-dev là bắt buộc để build Wayland IM (xkbcommon). Xem install.sh để biết danh sách chính xác theo distro."
    },
    {
      id: 4,
      title: "Biên dịch (debug)",
      description: "Build nhanh không tối ưu, phù hợp khi phát triển và test IBus/wayland_im.",
      command: `cargo build
# Test toàn bộ
cargo test -p vietc-engine -p vietc-daemon`,
      notes: "Chạy thử IBus engine trực tiếp: VIETC_FORCE_IBUS=1 cargo run --bin vietc —ดูđể ép đường IBus trên máy X11."
    },
    {
      id: 5,
      title: "Biên dịch (release - tối ưu)",
      description: "Build với tối ưu hóa cho hiệu năng cao nhất (LTO, strip).",
      command: `cargo build --release
# UI tray (tách workspace)
(cd ui && cargo build --release)`,
      notes: "Binary ở target/release/vietc (daemon) và ui/target/release/vietc-tray. Dùng cho install.sh --from-source."
    },
    {
      id: 6,
      title: "Cấp quyền uinput (chỉ cần cho evdev/X11)",
      description: "Nếu bạn test đường evdev trực tiếp (grab=true), cần quyền ghi /dev/uinput. Đường IBus trên Ubuntu Wayland KHÔNG cần bước này.",
      command: `sudo gpasswd -a $USER input
sudo groupadd -f uinput
sudo gpasswd -a $USER uinput
echo 'KERNEL=="uinput", GROUP="uinput", MODE="0660", OPTIONS+="static_node=uinput"' | sudo tee /etc/udev/rules.d/99-vietc.rules
sudo udevadm control --reload-rules && sudo udevadm trigger`,
      notes: "Đăng xuất và đăng nhập lại (hoặc reboot) để group có hiệu lực. Trên Ubuntu Wayland với IBus, có thể bỏ qua bước này hoàn toàn."
    },
    {
      id: 7,
      title: "Chạy thử (không cần cài đặt)",
      description: "Chạy trực tiếp từ thư mục build, không cần systemd service. Tự chọn đường dẫn theo session.",
      command: `# Tự động chọn IBus trên GNOME Wayland, evdev trên X11
./target/release/vietc
# Ép đường IBus để test Ubuntu fix
VIETC_FORCE_IBUS=1 ./target/release/vietc
# Xem log chi tiết
journalctl --user -f -u vietc.service  # nếu chạy qua service`,
      notes: "Tắt bằng Ctrl+C. Config đọc từ ~/.config/vietc/config.toml (auto_ibus=true mặc định). Sửa config và daemon sẽ hot-reload."
    }
  ];

  const tabs: { id: TabId; label: string; icon?: React.ReactNode }[] = [
    { id: 'mint_ubuntu', label: 'Mint / Ubuntu' },
    { id: 'arch', label: 'Arch Linux' },
    { id: 'fedora', label: 'Fedora' },
    { id: 'dev', label: 'Dev Build', icon: <Hammer size={12} /> },
  ];

  return (
    <div id="setup-guide" className="py-16 bg-[#0a0b0d] border-t border-white/10 scroll-mt-20">
      <div className="max-w-6xl mx-auto px-4 sm:px-6">
        
        {/* Section Header */}
        <div className="text-center max-w-3xl mx-auto mb-12">
          <motion.div
            initial={{ opacity: 0, y: 10 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs font-mono mb-4"
          >
            <Terminal size={12} className="text-emerald-400" />
            <span>NATIVE LINUX INTEGRATION — HYBRID ENGINE</span>
          </motion.div>
          <motion.h2
            initial={{ opacity: 0, y: 15 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ delay: 0.1 }}
            className="text-3xl sm:text-4xl font-serif text-white tracking-tight"
          >
            Hướng Dẫn Cài Đặt <span className="text-transparent bg-clip-text bg-gradient-to-r from-emerald-400 to-teal-400 italic">VietC</span>
          </motion.h2>
          <p className="mt-4 text-slate-400 text-sm sm:text-base">
            VietC <span className="text-emerald-400 font-semibold">gõ tiếng Việt trực tiếp không gạch chân (Zero Underline)</span>: Tương thích hoàn hảo trên <span className="text-slate-200">Ubuntu 24.04+ Wayland</span>, <span className="text-slate-200">Linux Mint, Debian, Arch Linux và Fedora</span>. Chuyển đổi 3 chế độ nhanh chóng với <span className="text-emerald-400 font-semibold">Ctrl + Shift</span>.
          </p>
          <div className="mt-4 flex flex-wrap justify-center gap-2 text-[11px] font-mono">
            <span className="px-2 py-1 rounded bg-emerald-500/10 border border-emerald-500/20 text-emerald-400">Zero Underline: Gõ trực tiếp</span>
            <span className="px-2 py-1 rounded bg-white/5 border border-white/10 text-slate-400">Phím tắt: Ctrl + Shift</span>
            <span className="px-2 py-1 rounded bg-white/5 border border-white/10 text-slate-400">Tray icon: EN · VN · TLX</span>
          </div>
        </div>

        {/* Tabs */}
        <div className="flex justify-center mb-10">
          <div className="bg-white/[0.02] p-1.5 rounded-xl border border-white/10 flex gap-2 w-full max-w-2xl">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={`flex-1 py-2.5 rounded-lg text-xs font-semibold tracking-wide transition-all cursor-pointer flex items-center justify-center gap-1.5 ${
                  activeTab === tab.id
                    ? 'bg-emerald-500 text-[#0a0b0d] font-bold shadow-[0_0_15px_rgba(16,185,129,0.25)]'
                    : 'text-slate-400 hover:text-slate-200 hover:bg-white/5'
                }`}
              >
                {tab.icon}
                {tab.label}
              </button>
            ))}
          </div>
        </div>

        {/* Content */}
        {activeTab === 'dev' ? (
          <div>
            <div className="flex items-center gap-2 mb-6">
              <GitBranch size={18} className="text-emerald-400" />
              <h3 className="text-lg font-semibold text-slate-100">Build từ mã nguồn (dành cho Developer)</h3>
            </div>
            <p className="text-slate-400 text-sm mb-8 max-w-3xl">
              Tự biên dịch VietC từ source. Mặc định <code className="text-emerald-400 font-mono">auto_ibus=true</code> nên trên GNOME Wayland sẽ tự dùng IBus engine — không cần setcap/evdev để test Wayland-native.
            </p>

            <div className="space-y-6">
              {devSteps.map((step, idx) => (
                <motion.div
                  key={`dev-step-${step.id}`}
                  initial={{ opacity: 0, x: -15 }}
                  whileInView={{ opacity: 1, x: 0 }}
                  viewport={{ once: true }}
                  transition={{ delay: idx * 0.08 }}
                  className="relative bg-white/[0.02] rounded-2xl border border-white/5 p-5 sm:p-6 lg:p-8 hover:border-emerald-500/30 transition-all group"
                >
                  {idx !== devSteps.length - 1 && (
                    <div className="absolute left-[33px] sm:left-[37px] top-[75px] bottom-[-35px] w-0.5 bg-white/5 pointer-events-none group-hover:bg-emerald-500/15 transition-all" />
                  )}

                  <div className="flex items-start gap-4 sm:gap-6">
                    <div className="flex-shrink-0 w-10 h-10 sm:w-12 sm:h-12 rounded-full bg-emerald-500/10 border border-emerald-500/20 flex items-center justify-center text-emerald-400 font-mono font-bold text-sm sm:text-base shadow-inner">
                      0{step.id}
                    </div>

                    <div className="flex-1 min-w-0">
                      <h3 className="text-base sm:text-lg font-sans font-semibold text-slate-100 mb-2">
                        {step.title}
                      </h3>
                      <p className="text-slate-400 text-xs sm:text-sm leading-relaxed mb-4">
                        {step.description}
                      </p>

                      {step.command && (
                        <div className="relative rounded-xl overflow-hidden bg-[#0d0e12] border border-white/10 shadow-2xl font-mono text-xs text-slate-300 group/term">
                          <div className="flex items-center justify-between px-4 py-2 bg-[#0a0b0d] border-b border-white/5">
                            <div className="flex items-center gap-1.5">
                              <div className="w-2.5 h-2.5 rounded-full bg-rose-500/60" />
                              <div className="w-2.5 h-2.5 rounded-full bg-amber-500/60" />
                              <div className="w-2.5 h-2.5 rounded-full bg-emerald-500/60" />
                              <span className="ml-2 text-[10px] text-slate-500 font-mono font-medium">BASH TERMINAL</span>
                            </div>
                            <button
                              onClick={() => handleCopy(step.command!, `dev-${step.id}`)}
                              className="p-1 rounded hover:bg-white/5 text-slate-400 hover:text-slate-200 transition-colors cursor-pointer"
                              title="Sao chép lệnh"
                            >
                              {copiedText === `dev-${step.id}` ? (
                                <Check size={14} className="text-emerald-400" />
                              ) : (
                                <Copy size={14} />
                              )}
                            </button>
                          </div>
                          <div className="p-4 overflow-x-auto whitespace-pre leading-5 selection:bg-emerald-500/30 selection:text-white">
                            {step.command}
                          </div>
                        </div>
                      )}

                      {step.notes && (
                        <div className="mt-3 flex gap-2 p-3.5 rounded-xl bg-emerald-950/15 border border-emerald-500/10 text-xs text-emerald-300/90">
                          <Shield size={14} className="flex-shrink-0 mt-0.5 text-emerald-400" />
                          <div className="leading-relaxed">
                            <span className="font-semibold text-emerald-400">Lưu ý:</span> {step.notes}
                          </div>
                        </div>
                      )}
                    </div>
                  </div>
                </motion.div>
              ))}
            </div>
          </div>
        ) : (
          <div className="space-y-8">
            {installSteps[activeTab].map((step, idx) => (
              <motion.div
                key={`${activeTab}-${step.id}`}
                initial={{ opacity: 0, x: -15 }}
                whileInView={{ opacity: 1, x: 0 }}
                viewport={{ once: true }}
                transition={{ delay: idx * 0.1 }}
                className="relative bg-white/[0.02] rounded-2xl border border-white/5 p-5 sm:p-6 lg:p-8 hover:border-emerald-500/30 transition-all group"
              >
                {idx !== installSteps[activeTab].length - 1 && (
                  <div className="absolute left-[33px] sm:left-[37px] top-[75px] bottom-[-45px] w-0.5 bg-white/5 pointer-events-none group-hover:bg-emerald-500/15 transition-all" />
                )}

                <div className="flex items-start gap-4 sm:gap-6">
                  <div className="flex-shrink-0 w-10 h-10 sm:w-12 sm:h-12 rounded-full bg-white/5 border border-white/10 flex items-center justify-center text-emerald-400 font-mono font-bold text-sm sm:text-base shadow-inner group-hover:border-emerald-500/30 group-hover:bg-emerald-500/10 transition-all">
                    0{step.id}
                  </div>

                  <div className="flex-1 min-w-0">
                    <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 mb-3">
                      <h3 className="text-base sm:text-lg font-sans font-semibold text-slate-100 group-hover:text-emerald-300 transition-colors">
                        {step.title}
                      </h3>
                    </div>

                    <p className="text-slate-400 text-xs sm:text-sm leading-relaxed mb-4">
                      {step.description}
                    </p>

                    {step.command && (
                      <div className="relative rounded-xl overflow-hidden bg-[#0d0e12] border border-white/10 shadow-2xl font-mono text-xs text-slate-300 group/term">
                        <div className="flex items-center justify-between px-4 py-2 bg-[#0a0b0d] border-b border-white/5">
                          <div className="flex items-center gap-1.5">
                            <div className="w-2.5 h-2.5 rounded-full bg-rose-500/60" />
                            <div className="w-2.5 h-2.5 rounded-full bg-amber-500/60" />
                            <div className="w-2.5 h-2.5 rounded-full bg-emerald-500/60" />
                            <span className="ml-2 text-[10px] text-slate-500 font-mono font-medium">BASH TERMINAL</span>
                          </div>
                          <button
                            onClick={() => handleCopy(step.command!, `${activeTab}-${step.id}`)}
                            className="p-1 rounded hover:bg-white/5 text-slate-400 hover:text-slate-200 transition-colors cursor-pointer"
                            title="Sao chép lệnh"
                          >
                            {copiedText === `${activeTab}-${step.id}` ? (
                              <Check size={14} className="text-emerald-400" />
                            ) : (
                              <Copy size={14} />
                            )}
                          </button>
                        </div>
                        <div className="p-4 overflow-x-auto whitespace-pre leading-5 selection:bg-emerald-500/30 selection:text-white">
                          {step.command}
                        </div>
                      </div>
                    )}

                    {step.notes && (
                      <div className="mt-3 flex gap-2 p-3.5 rounded-xl bg-emerald-950/15 border border-emerald-500/10 text-xs text-emerald-300/90">
                        <Shield size={14} className="flex-shrink-0 mt-0.5 text-emerald-400" />
                        <div className="leading-relaxed">
                          <span className="font-semibold text-emerald-400">Lưu ý:</span> {step.notes}
                        </div>
                      </div>
                    )}
                  </div>
                </div>
              </motion.div>
            ))}
            {activeTab === 'mint_ubuntu' && (
              <div className="rounded-2xl border border-amber-500/20 bg-amber-950/10 p-5 flex gap-3">
                <Info size={18} className="text-amber-400 flex-shrink-0 mt-0.5" />
                <div className="text-xs leading-relaxed">
                  <div className="font-semibold text-amber-300 mb-1 flex items-center gap-2">
                    <Sparkles size={12} /> Tại sao Ubuntu Wayland cần IBus?
                  </div>
                  <p className="text-amber-200/80">
                    Mutter (GNOME Wayland) không hỗ trợ <code className="font-mono text-amber-300">zwp_input_method_v2</code> — đường Wayland thuần không thể kích hoạt. 
                    VietC học từ <span className="text-amber-300 font-semibold">Funput</span>: trên Ubuntu Wayland, tự chuyển thành <span className="text-white font-semibold">IBus engine</span> gốc (qua D-Bus <code className="font-mono">org.freedesktop.IBus</code>), có preedit gạch chân mượt, hoạt động trong mọi app Wayland-native (Firefox, Text Editor, Ptyxis) mà đường X11 không thấy. <br/>
                    <span className="text-slate-400">Trên Mint X11/evdev vẫn giữ direct 0ms không IBus.</span> Đổi hành vi bằng <code className="font-mono text-white">auto_ibus=false</code> trong <code className="font-mono">~/.config/vietc/config.toml</code> hoặc <code className="font-mono">VIETC_FORCE_IBUS=1</code>.
                  </p>
                </div>
              </div>
            )}
          </div>
        )}

        {/* Architecture graphic */}
        <div className="mt-16 bg-gradient-to-br from-white/[0.03] to-transparent p-6 sm:p-8 rounded-3xl border border-white/10">
          <h3 className="text-lg sm:text-xl font-semibold text-slate-100 flex items-center gap-2 mb-6">
            <Layers className="text-emerald-400" size={18} />
            <span>Kiến trúc Hybrid thông minh của VietC</span>
          </h3>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div className="p-5 rounded-2xl bg-[#0d0e12] border border-white/5">
              <div className="w-8 h-8 rounded-lg bg-sky-500/10 border border-sky-500/20 text-sky-400 flex items-center justify-center font-bold text-xs mb-3">
                IBUS
              </div>
              <h4 className="text-sm font-semibold text-slate-200 mb-2">Ubuntu 24.04+ Wayland</h4>
              <p className="text-slate-400 text-xs leading-relaxed">
                Chạy như <code className="text-sky-400 font-mono">IBus engine</code> gốc qua D-Bus (học từ Funput). Có preedit gạch chân, <code className="text-slate-300">Ctrl+Space</code> mượt, bao phủ 100% app Wayland-native (Firefox, ptyxis, gedit) mà evdev/X11 không thấy.
              </p>
              <div className="mt-3 text-[10px] font-mono text-sky-400 bg-sky-950/20 px-2 py-1 rounded border border-sky-500/10">GNOME Wayland → org.freedesktop.IBus</div>
            </div>
            
            <div className="p-5 rounded-2xl bg-[#0d0e12] border border-white/5">
              <div className="w-8 h-8 rounded-lg bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 flex items-center justify-center font-bold text-xs mb-3">
                EVDEV
              </div>
              <h4 className="text-sm font-semibold text-slate-200 mb-2">Mint / Arch / X11</h4>
              <p className="text-slate-400 text-xs leading-relaxed">
                Chặn (grab) sự kiện gốc từ bàn phím qua <code className="text-emerald-400 font-mono">evdev</code>, tính toán bằng State Machine và xuất ra bàn phím ảo <code className="text-emerald-400 font-mono">uinput</code>.
              </p>
              <div className="mt-3 text-[10px] font-mono text-emerald-400 bg-emerald-950/20 px-2 py-1 rounded border border-emerald-500/10">0ms direct — không IBus</div>
            </div>

            <div className="p-5 rounded-2xl bg-emerald-950/15 border border-emerald-500/20">
              <div className="w-8 h-8 rounded-lg bg-emerald-500/20 text-emerald-300 flex items-center justify-center font-bold text-xs mb-3">
                AUTO
              </div>
              <h4 className="text-sm font-semibold text-emerald-300 mb-2">Tự động chọn</h4>
              <p className="text-emerald-400/80 text-xs leading-relaxed">
                <code className="text-white font-mono">auto_ibus=true</code> tự phát hiện GNOME Wayland và chọn đường mượt nhất. Dev có thể ép <code className="text-white font-mono">VIETC_FORCE_IBUS=1</code> hoặc <code className="text-white font-mono">ibus_engine=true</code> trong config.
              </p>
              <div className="mt-3 text-[10px] font-mono text-emerald-300 bg-emerald-500/10 px-2 py-1 rounded border border-emerald-500/20">Thứ tự: IBus → zwp_v2 → evdev → X11</div>
            </div>
          </div>
          <div className="mt-6 text-center text-[11px] font-mono text-slate-500">
            Xem <code className="text-emerald-400">docs/wayland-rootless.md</code> và <code className="text-emerald-400">daemon/src/main.rs</code> để hiểu thứ tự thử đường dẫn chi tiết.
          </div>
        </div>

      </div>
    </div>
  );
}
