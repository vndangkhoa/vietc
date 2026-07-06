import React from 'react';
import { motion } from 'motion/react';
import { Terminal, ArrowRight, Sparkles, Shield, Cpu, Zap, Download } from 'lucide-react';
import DragonMascot from './DragonMascot';

interface HeroProps {
  setActiveView: (view: 'home' | 'keycaps') => void;
}

export default function Hero({ setActiveView }: HeroProps) {
  const scrollToDemo = () => {
    document.getElementById('demo')?.scrollIntoView({ behavior: 'smooth' });
  };

  const scrollToSetup = () => {
    document.getElementById('setup-guide')?.scrollIntoView({ behavior: 'smooth' });
  };

  return (
    <div className="relative pt-10 pb-20 px-4 sm:px-6 overflow-hidden bg-[#0a0b0d]">
      
      {/* Background ambient lighting */}
      <div className="absolute top-0 left-1/4 w-[500px] h-[500px] rounded-full bg-emerald-500/5 blur-[120px] pointer-events-none" />
      <div className="absolute top-1/3 right-1/4 w-[600px] h-[600px] rounded-full bg-teal-500/5 blur-[150px] pointer-events-none" />

      <div className="max-w-6xl mx-auto grid grid-cols-1 lg:grid-cols-12 gap-12 items-center relative z-10">
        
        {/* LEFT COLUMN: Main Presentation & CTAs */}
        <div className="lg:col-span-6 space-y-6 text-left">
          
          {/* Version badge */}
          <motion.div
            initial={{ opacity: 0, scale: 0.9 }}
            animate={{ opacity: 1, scale: 1 }}
            className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs font-mono font-semibold"
          >
            <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse" />
            <span>VietC v1.2.0 - Native Linux Input Mode</span>
          </motion.div>

          <motion.h1
            initial={{ opacity: 0, y: 15 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.1 }}
            className="text-4xl sm:text-5xl lg:text-6xl font-serif text-white leading-tight"
          >
            Gõ Tiếng Việt <br />
            <span className="text-transparent bg-clip-text bg-gradient-to-r from-emerald-400 to-teal-400 italic">
              "Như Bay"
            </span> <br />
            Mượt Mà Trên Linux!
          </motion.h1>

          <p className="text-slate-400 text-sm sm:text-base leading-relaxed max-w-lg">
            VietC là giải pháp nhập liệu mã nguồn mở hiện đại cho môi trường Linux, tối ưu hóa tốc độ và sự đơn giản với linh vật chú rồng con Long-kun. Không qua IBus/Fcitx5 phức tạp, giải quyết triệt để lỗi nuốt phím và lag chữ.
          </p>

          {/* Quick Metrics */}
          <div className="grid grid-cols-3 gap-3 max-w-md pt-2">
            <div className="p-3 rounded-xl bg-white/[0.02] border border-white/5 flex flex-col items-center">
              <span className="text-xs text-slate-500 font-mono">Keystroke</span>
              <span className="text-sm font-bold text-emerald-400 font-mono">0ms</span>
            </div>
            <div className="p-3 rounded-xl bg-white/[0.02] border border-white/5 flex flex-col items-center">
              <span className="text-xs text-slate-500 font-mono">IBus/Fcitx5</span>
              <span className="text-sm font-bold text-emerald-300 font-mono">Bypass</span>
            </div>
            <div className="p-3 rounded-xl bg-white/[0.02] border border-white/5 flex flex-col items-center">
              <span className="text-xs text-slate-500 font-mono">Trễ Phím</span>
              <span className="text-sm font-bold text-emerald-400 font-mono">Giảm 20x</span>
            </div>
          </div>

          {/* Action buttons */}
          <div className="flex flex-col sm:flex-row gap-3 pt-4">
            <button
              onClick={scrollToSetup}
              className="px-6 py-3.5 rounded-xl bg-emerald-500 hover:bg-emerald-400 text-[#0a0b0d] font-sans font-bold text-sm transition-all shadow-[0_0_20px_rgba(16,185,129,0.25)] flex items-center justify-center gap-2 group cursor-pointer"
            >
              Cài Đặt Trên Linux
              <ArrowRight size={16} className="group-hover:translate-x-1 transition-transform" />
            </button>

            <button
              onClick={() => setActiveView('keycaps')}
              className="px-6 py-3.5 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-slate-300 hover:text-white font-sans font-bold text-sm transition-all flex items-center justify-center gap-2 cursor-pointer"
            >
              <Sparkles size={14} className="text-emerald-400 animate-pulse" />
              Artisan Keycaps 3D
            </button>
          </div>

        </div>

        {/* RIGHT COLUMN: Official Announcement Card */}
        <div className="lg:col-span-6 flex flex-col items-center">
          
          <motion.div
            initial={{ opacity: 0, scale: 0.95, y: 15 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            transition={{ delay: 0.2 }}
            className="w-full max-w-md bg-gradient-to-b from-white/[0.04] to-transparent p-6 sm:p-8 rounded-[2rem] border border-white/10 shadow-2xl relative overflow-hidden"
          >
            {/* Soft inner corner borders */}
            <div className="absolute top-2 left-2 w-4 h-4 border-t border-l border-white/20 rounded-tl" />
            <div className="absolute top-2 right-2 w-4 h-4 border-t border-r border-white/20 rounded-tr" />
            <div className="absolute bottom-2 left-2 w-4 h-4 border-b border-l border-white/20 rounded-bl" />
            <div className="absolute bottom-2 right-2 w-4 h-4 border-b border-r border-white/20 rounded-br" />

            {/* Mascot on top of announcement */}
            <div className="flex flex-col items-center mb-6">
              <DragonMascot size={110} />
              <h2 className="font-sans font-black text-white text-2xl tracking-widest uppercase mt-1">
                VIETC<span className="text-emerald-500">.</span>
              </h2>
            </div>

            {/* Official Announcement body */}
            <div className="space-y-6 text-center">
              
              <div className="bg-emerald-950/20 border border-emerald-500/20 py-2.5 px-4 rounded-xl">
                <span className="font-sans font-black text-xl sm:text-2xl text-transparent bg-clip-text bg-gradient-to-r from-emerald-300 via-teal-300 to-emerald-400 tracking-wide block uppercase">
                  TUYÊN BỐ CHÍNH THỨC
                </span>
              </div>

              <p className="text-slate-300 text-xs sm:text-sm leading-relaxed text-justify">
                Để đơn giản hóa tối đa việc gõ VNI trên Terminal bấy lâu nay vô cùng <span className="text-emerald-400 font-bold">‘gian khổ’</span> cho dân Linux, <span className="text-emerald-400 font-bold">VIETC</span> tự hào công bố đã support native gõ VNI trên Terminal.
              </p>

              <p className="text-slate-400 text-xs leading-relaxed text-justify">
                Quý khách có thể tải toàn bộ các bản thiết kế <span className="text-emerald-400 font-bold">3D phím Numlock và Keycap</span> trên website VIETC xuống. Sau đấy, tận dụng trí tưởng tượng phong phú để <span className="text-emerald-400 font-bold">lắp ghép</span> và trải nghiệm cảm giác <span className="text-teal-300 font-bold">‘gõ như bay’</span> ngay trên Terminal ảo của bạn mà không cần bất kỳ phần cứng vật lý nào.
              </p>

            </div>

            {/* Try online indicator */}
            <div className="mt-6 pt-4 border-t border-white/5 flex justify-center">
              <button
                onClick={scrollToDemo}
                className="text-[11px] font-mono text-emerald-400 hover:text-emerald-300 flex items-center gap-1.5 transition-colors cursor-pointer"
              >
                <Terminal size={12} className="text-emerald-500" />
                <span>Trải nghiệm giả lập Terminal bên dưới &darr;</span>
              </button>
            </div>

          </motion.div>

        </div>

      </div>

    </div>
  );
}
