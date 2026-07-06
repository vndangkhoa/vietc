import React, { useState } from 'react';
import { motion } from 'motion/react';
import { ShieldCheck, Cpu, GitCompare, HelpCircle, Layers, ArrowRight } from 'lucide-react';

export default function Features() {
  const [hoveredState, setHoveredState] = useState<number | null>(null);

  const stateDetails = [
    {
      id: 0,
      name: "S0 - Idle (Chờ phím)",
      desc: "Trạng thái nghỉ ngơi ban đầu. VietC lắng nghe thụ động thiết bị đầu vào evdev mà không can thiệp, đảm bảo CPU tiêu thụ ~0%."
    },
    {
      id: 1,
      name: "S1 - Vowel Buffer (Thu nhận nguyên âm)",
      desc: "Kích hoạt khi phát hiện nguyên âm gốc (a, e, o, u, y, i). VietC tạo bộ đệm từ cục bộ để chuẩn bị ghép dấu thanh."
    },
    {
      id: 2,
      name: "S2 - Accent Applied (Tạo dấu thanh)",
      desc: "Nạp các phím gõ dấu thanh VNI (1-5). Thuật toán tối ưu hóa vị trí đặt dấu theo đúng ngữ pháp Việt ngữ chuẩn."
    },
    {
      id: 3,
      name: "S3 - Modifiers (Ký tự đặc biệt)",
      desc: "Áp dụng mũ/râu (6-9) để biến đổi thành ă, â, đ, ê, ô, ơ, ư. Kết thúc chu kỳ xử lý và sẵn sàng phát phím uinput mới."
    }
  ];

  return (
    <div id="features" className="py-16 bg-[#0a0b0d] border-t border-white/10 scroll-mt-20">
      <div className="max-w-6xl mx-auto px-4 sm:px-6">
        
        {/* Section Title */}
        <div className="text-center max-w-3xl mx-auto mb-16">
          <motion.div
            initial={{ opacity: 0, y: 10 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs font-mono mb-4"
          >
            <Cpu size={12} className="text-emerald-500" />
            <span>HOW IT WORKS</span>
          </motion.div>
          <motion.h2
            initial={{ opacity: 0, y: 15 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ delay: 0.1 }}
            className="text-3xl sm:text-4xl font-serif text-white tracking-tight"
          >
            Sự Khác Biệt Làm Nên Sức Mạnh <span className="text-transparent bg-clip-text bg-gradient-to-r from-emerald-400 to-teal-400 italic">VietC</span>
          </motion.h2>
          <p className="mt-4 text-slate-400 text-sm sm:text-base leading-relaxed">
            VietC được phát triển dựa trên 3 trụ cột kỹ thuật cốt lõi giúp tối đa hóa tốc độ, độ ổn định tuyệt đối và khả năng tương thích 100% với môi trường giả lập Linux Terminal.
          </p>
        </div>

        {/* 3 Pillars Grid */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-8 mb-16">
          
          {/* Pillar 1: State Machine */}
          <div className="bg-white/[0.02] rounded-2xl border border-white/5 p-6 flex flex-col justify-between hover:border-emerald-500/30 transition-all">
            <div>
              <div className="w-10 h-10 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 flex items-center justify-center mb-5">
                <Layers size={20} />
              </div>
              <h3 className="text-base font-sans font-bold text-white mb-3">
                1. State Machine Deterministic
              </h3>
              <p className="text-slate-400 text-xs sm:text-sm leading-relaxed mb-4">
                Sử dụng mô hình toán học Finite State Machine (FSM) tất định để phân tích tổ hợp phím gõ tiếng Việt. Mọi ký tự được tính toán rõ ràng giúp tránh tình trạng xung đột, mất từ hoặc sai vị trí đặt dấu khi gõ nhanh.
              </p>
            </div>
            <div className="text-[11px] font-mono text-emerald-400 mt-2 bg-emerald-950/15 p-2.5 rounded-lg border border-emerald-500/10">
              S0 (Chờ) &rarr; S1 (Gõ) &rarr; S2 (Dấu) &rarr; S3 (Chữ mũ)
            </div>
          </div>

          {/* Pillar 2: Token-Level Diffing */}
          <div className="bg-white/[0.02] rounded-2xl border border-white/5 p-6 flex flex-col justify-between hover:border-emerald-500/30 transition-all">
            <div>
              <div className="w-10 h-10 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 flex items-center justify-center mb-5">
                <GitCompare size={20} />
              </div>
              <h3 className="text-base font-sans font-bold text-white mb-3">
                2. Token-Level Diffing
              </h3>
              <p className="text-slate-400 text-xs sm:text-sm leading-relaxed mb-4">
                Thay vì xóa trắng toàn bộ từ hoặc phát lại một loạt phím Backspace dồn dập gây giật màn hình trong Terminal, VietC tính toán sự khác biệt nhỏ nhất giữa từ đã gõ và từ mong muốn để thay thế cục bộ tức thì.
              </p>
            </div>
            <div className="text-[11px] font-mono text-emerald-400 mt-2 bg-emerald-950/15 p-2.5 rounded-lg border border-emerald-500/10 flex items-center justify-between">
              <span>trang thái</span>
              <ArrowRight size={10} />
              <span className="font-bold">trạng thái [1ms]</span>
            </div>
          </div>

          {/* Pillar 3: Privacy-First Event Sourcing */}
          <div className="bg-white/[0.02] rounded-2xl border border-white/5 p-6 flex flex-col justify-between hover:border-emerald-500/30 transition-all">
            <div>
              <div className="w-10 h-10 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 flex items-center justify-center mb-5">
                <ShieldCheck size={20} />
              </div>
              <h3 className="text-base font-sans font-bold text-white mb-3">
                3. Privacy-First Event Sourcing
              </h3>
              <p className="text-slate-400 text-xs sm:text-sm leading-relaxed mb-4">
                Xử lý sự kiện bàn phím theo luồng độc lập dưới quyền user thông qua uinput cục bộ. VietC nói KHÔNG với kết nối Internet, đảm bảo toàn bộ mật khẩu, lệnh Terminal nhạy cảm luôn được bảo vệ an toàn tuyệt đối.
              </p>
            </div>
            <div className="text-[11px] font-mono text-emerald-400 mt-2 bg-emerald-950/15 p-2.5 rounded-lg border border-emerald-500/10">
              Kiểm soát cục bộ 100% &bull; Không thu thập dữ liệu
            </div>
          </div>

        </div>

        {/* Detalized State Machine Explanation Block */}
        <div className="bg-white/[0.02] p-6 sm:p-8 rounded-3xl border border-white/10 flex flex-col lg:flex-row gap-8 items-center">
          
          {/* Diagrams Left */}
          <div className="w-full lg:w-1/2 space-y-4">
            <h3 className="text-lg sm:text-xl font-bold text-white mb-2">
              Tìm Hiểu Trạng Thái Finite State Machine
            </h3>
            <p className="text-slate-400 text-xs sm:text-sm leading-relaxed">
              Khi bạn gõ phím, VietC không lưu trữ ký tự dưới dạng văn bản tĩnh thô sơ. Hệ thống sẽ thay đổi các nút liên kết (S0, S1, S2, S3) dựa trên loại phím nhận được để tính toán cách phản hồi phím nhanh nhất. Di chuột vào các nút dưới đây để xem mô tả:
            </p>

            <div className="grid grid-cols-2 gap-3 pt-3">
              {stateDetails.map((det) => (
                <div
                  key={det.id}
                  onMouseEnter={() => setHoveredState(det.id)}
                  onMouseLeave={() => setHoveredState(null)}
                  className={`p-3 rounded-xl border transition-all cursor-pointer ${
                    hoveredState === det.id
                      ? 'bg-emerald-500/10 border-emerald-500 text-emerald-300'
                      : 'bg-[#0a0b0d] border-white/5 text-slate-400 hover:border-emerald-500/30'
                  }`}
                >
                  <div className="font-mono text-xs font-bold text-white mb-1">
                    Trạng thái S{det.id}
                  </div>
                  <div className="text-[10px] leading-relaxed">
                    {det.name.split(' - ')[1]}
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Interactive State explanation box Right */}
          <div className="w-full lg:w-1/2 bg-[#0a0b0d] p-6 rounded-2xl border border-white/10 min-h-[220px] flex flex-col justify-center">
            {hoveredState !== null ? (
              <motion.div
                key={`state-det-${hoveredState}`}
                initial={{ opacity: 0, x: 10 }}
                animate={{ opacity: 1, x: 0 }}
                className="space-y-3"
              >
                <div className="inline-flex px-2 py-0.5 rounded bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 font-mono text-[10px] font-bold">
                  SỰ KIỆN ĐANG HOẠT ĐỘNG: S{hoveredState}
                </div>
                <h4 className="text-base font-sans font-bold text-white">
                  {stateDetails[hoveredState].name}
                </h4>
                <p className="text-slate-400 text-xs sm:text-sm leading-relaxed">
                  {stateDetails[hoveredState].desc}
                </p>
              </motion.div>
            ) : (
              <div className="text-center text-slate-500 text-xs sm:text-sm leading-relaxed py-8">
                <HelpCircle className="mx-auto text-slate-600 mb-3" size={24} />
                Hãy di chuột qua các nút trạng thái bên cạnh để khám phá cách thiết lập hệ thống logic gõ phím tất định của VietC!
              </div>
            )}
          </div>

        </div>

      </div>
    </div>
  );
}
