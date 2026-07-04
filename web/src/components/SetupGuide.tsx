import React, { useState } from 'react';
import { motion, AnimatePresence } from 'motion/react';
import { Copy, Check, Terminal, Shield, Cpu, RefreshCw, Layers, GitBranch, Hammer } from 'lucide-react';
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
        title: "Cài đặt VietC (Pre-built)",
        description: "Chạy lệnh dưới đây để tự động tải về, cài đặt phụ thuộc và biên dịch VietC trên hệ thống của bạn.",
        command: `git clone https://github.com/vndangkhoa/vietc.git /tmp/vietc \\
  && cd /tmp/vietc && sudo ./install.sh`,
        notes: "Script tự động phát hiện distro, cài đặt dependencies, build và cấu hình udev rules cho uinput."
      },
      {
        id: 2,
        title: "Gỡ cài đặt (Uninstall)",
        description: "Xoá hoàn toàn VietC khỏi hệ thống, bao gồm binary, service và udev rules.",
        command: `curl -sSL https://raw.githubusercontent.com/vndangkhoa/vietc/main/uninstall.sh | sudo bash`,
        notes: "Lệnh này sẽ xoá /usr/local/bin/vietc, systemd service và các file cấu hình."
      }
    ],
    arch: [
      {
        id: 1,
        title: "Cài đặt VietC (Pre-built)",
        description: "Tự động clone, build và cài đặt VietC trên Arch Linux.",
        command: `git clone https://github.com/vndangkhoa/vietc.git /tmp/vietc \\
  && cd /tmp/vietc && sudo ./install.sh`,
        notes: "Script hỗ trợ pacman, tự động cài đặt base-devel và các thư viện cần thiết."
      },
      {
        id: 2,
        title: "Gỡ cài đặt (Uninstall)",
        description: "Xoá VietC hoàn toàn khỏi hệ thống Arch.",
        command: `curl -sSL https://raw.githubusercontent.com/vndangkhoa/vietc/main/uninstall.sh | sudo bash`,
      }
    ],
    fedora: [
      {
        id: 1,
        title: "Cài đặt VietC (Pre-built)",
        description: "Tự động clone, build và cài đặt VietC trên Fedora.",
        command: `git clone https://github.com/vndangkhoa/vietc.git /tmp/vietc \\
  && cd /tmp/vietc && sudo ./install.sh`,
        notes: "Script hỗ trợ dnf, tự động cài đặt Development Tools và thư viện X11."
      },
      {
        id: 2,
        title: "Gỡ cài đặt (Uninstall)",
        description: "Xoá VietC hoàn toàn khỏi hệ thống Fedora.",
        command: `curl -sSL https://raw.githubusercontent.com/vndangkhoa/vietc/main/uninstall.sh | sudo bash`,
      }
    ]
  };

  const devSteps: SetupStep[] = [
    {
      id: 1,
      title: "Clone mã nguồn",
      description: "Nhánh main chứa code mới nhất.",
      command: `git clone https://github.com/vndangkhoa/vietc.git
cd vietc`,
    },
    {
      id: 2,
      title: "Cài đặt Rust (nếu chưa có)",
      description: "Dùng rustup để cài Rust toolchain mới nhất.",
      command: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"`,
      notes: "Kiểm tra với 'rustc --version' và 'cargo --version'."
    },
    {
      id: 3,
      title: "Cài đặt hệ thống phụ thuộc",
      description: "Thư viện dev cho X11, evdev và dbus.",
      command: `sudo apt install build-essential pkg-config libx11-dev libxtst-dev \\
  libevdev-dev libdbus-1-dev libwayland-dev wl-clipboard`,
      notes: "Trên Fedora: dnf install; trên Arch: pacman -S. Xem install.sh để biết chi tiết."
    },
    {
      id: 4,
      title: "Biên dịch (debug)",
      description: "Build nhanh không tối ưu, phù hợp khi phát triển.",
      command: `cargo build`,
      notes: "Binary ở target/debug/vietc. Chạy thử: ./target/debug/vietc"
    },
    {
      id: 5,
      title: "Biên dịch (release - tối ưu)",
      description: "Build với tối ưu hóa cho hiệu năng cao nhất.",
      command: `cargo build --release`,
      notes: "Binary ở target/release/vietc. Chạy thử: ./target/release/vietc"
    },
    {
      id: 6,
      title: "Cấp quyền uinput",
      description: "VietC cần quyền ghi /dev/uinput. Thêm user vào group input và uinput.",
      command: `sudo gpasswd -a $USER input
sudo groupadd -f uinput
sudo gpasswd -a $USER uinput
echo 'KERNEL=="uinput", GROUP="uinput", MODE="0660", OPTIONS+="static_node=uinput"' | sudo tee /etc/udev/rules.d/99-vietc.rules
sudo udevadm control --reload-rules && sudo udevadm trigger`,
      notes: "Đăng xuất và đăng nhập lại (hoặc reboot) để group có hiệu lực."
    },
    {
      id: 7,
      title: "Chạy thử (không cần cài đặt)",
      description: "Chạy trực tiếp từ thư mục build, không cần systemd service.",
      command: `./target/release/vietc`,
      notes: "Tắt bằng Ctrl+C. Có thể chạy ở chế độ nền với '&' và dùng 'fg' để đưa lên foreground."
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
            <span>NATIVE LINUX INTEGRATION</span>
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
            Vì VietC là bộ gõ mức thấp (System & Application level) không phụ thuộc IBus hay Fcitx5, việc cài đặt sẽ tác động trực tiếp lên driver uinput hệ thống để đạt tốc độ gõ tuyệt đối.
          </p>
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
              Các bước dưới đây hướng dẫn bạn tự biên dịch VietC từ source, chạy thử mà không cần cài đặt
              system-wide. Phù hợp cho developer muốn đóng góp hoặc tùy chỉnh.
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
          </div>
        )}

        {/* Architecture graphic */}
        <div className="mt-16 bg-gradient-to-br from-white/[0.03] to-transparent p-6 sm:p-8 rounded-3xl border border-white/10">
          <h3 className="text-lg sm:text-xl font-semibold text-slate-100 flex items-center gap-2 mb-6">
            <Layers className="text-emerald-400" size={18} />
            <span>Mô Hình Hoạt Động Khác Biệt của VietC</span>
          </h3>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div className="p-5 rounded-2xl bg-[#0d0e12] border border-white/5">
              <div className="w-8 h-8 rounded-lg bg-red-500/10 border border-red-500/20 text-red-400 flex items-center justify-center font-bold text-xs mb-3">
                OLD
              </div>
              <h4 className="text-sm font-semibold text-slate-200 mb-2">IBus / Fcitx5</h4>
              <p className="text-slate-400 text-xs leading-relaxed">
                Hoạt động ở Application Layer qua cơ chế giao tiếp DBus phức tạp. Khi gõ trong Terminal ảo, các lệnh Backspace/Delete giả lập thường bị trễ hoặc nuốt ký tự gây lỗi nhân đôi hoặc mất chữ.
              </p>
            </div>
            
            <div className="p-5 rounded-2xl bg-[#0d0e12] border border-white/5">
              <div className="w-8 h-8 rounded-lg bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 flex items-center justify-center font-bold text-xs mb-3">
                NEW
              </div>
              <h4 className="text-sm font-semibold text-slate-200 mb-2">VietC (uinput + evdev)</h4>
              <p className="text-slate-400 text-xs leading-relaxed">
                Chặn (grab) sự kiện gốc từ bàn phím vật lý thông qua driver <code className="text-emerald-400 font-mono">evdev</code>, sau đó tự tính toán bằng State Machine và xuất ra bàn phím ảo mới thông qua <code className="text-emerald-400 font-mono">uinput</code>.
              </p>
            </div>

            <div className="p-5 rounded-2xl bg-emerald-950/15 border border-emerald-500/20">
              <div className="w-8 h-8 rounded-lg bg-emerald-500/20 text-emerald-300 flex items-center justify-center font-bold text-xs mb-3">
                WIN
              </div>
              <h4 className="text-sm font-semibold text-emerald-300 mb-2">Trải Nghiệm "Như Bay"</h4>
              <p className="text-emerald-400/80 text-xs leading-relaxed">
                Độ trễ phản hồi phím <span className="text-white font-semibold">Keystroke: 0ms</span> và giải phóng nút <span className="text-white font-semibold">&lt;1ms</span>. Gõ tiếng Việt gốc 100% không bị lag, không kén Terminal nào (Alacritty, Kitty, GNOME Terminal, v.v.).
              </p>
            </div>
          </div>
        </div>

      </div>
    </div>
  );
}
