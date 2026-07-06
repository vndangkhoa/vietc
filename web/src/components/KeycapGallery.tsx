import React, { useState } from 'react';
import { motion, AnimatePresence } from 'motion/react';
import { Download, Sliders, Palette, Lightbulb, Type, Layers, CheckCircle2, Star, Sparkles, AlertCircle } from 'lucide-react';
import { KeycapCustomization, KeycapModel } from '../types';
import DragonMascot from './DragonMascot';

export default function KeycapGallery() {
  const [custom, setCustom] = useState<KeycapCustomization>({
    baseColor: '#0E7490', // Cyan 700
    stemColor: '#10B981', // Emerald 500
    dragonColor: '#3B82F6', // Blue 500
    material: 'resin_clear',
    ledColor: '#06B6D4', // Cyan 500
    ledIntensity: 75,
    selectedLetter: 'đ',
    showStem: true
  });

  const [downloadingId, setDownloadingId] = useState<string | null>(null);
  const [downloadProgress, setDownloadProgress] = useState(0);
  const [showSuccessToast, setShowSuccessToast] = useState(false);
  const [toastMessage, setToastMessage] = useState('');

  const lettersList = ['ă', 'â', 'đ', 'ê', 'ô', 'ơ', 'ư', 's (́)', 'f (̀)', 'r (̉)', 'x (̃)', 'j (̣)'];

  const colorPresets = [
    { name: 'Rồng Biển Trầm', value: '#0E7490', text: 'text-cyan-400' },
    { name: 'Ngọc Lục Bảo', value: '#047857', text: 'text-emerald-400' },
    { name: 'Hồng Anh Đào', value: '#BE185D', text: 'text-pink-400' },
    { name: 'Hổ Phách Sáng', value: '#B45309', text: 'text-amber-400' },
    { name: 'Thạch Anh Tím', value: '#6D28D9', text: 'text-violet-400' },
    { name: 'Khói Obsidian', value: '#374151', text: 'text-slate-400' }
  ];

  const ledPresets = [
    { name: 'Cyan Neon', value: '#06B6D4' },
    { name: 'Toxic Green', value: '#10B981' },
    { name: 'Sunset Orange', value: '#F97316' },
    { name: 'Sakura Pink', value: '#EC4899' },
    { name: 'Chroma RGB', value: '#8B5CF6' }
  ];

  const keycapModels: KeycapModel[] = [
    {
      id: 'dragon_keycap_oem',
      name: 'Rồng Con OEM Esc Keycap',
      letter: 'ESC',
      desc: 'Mẫu phím cơ Esc chứa rồng con dễ thương ở trung tâm, đúc khuôn resin thủ công siêu chi tiết.',
      rarity: 'Legendary',
      stlUrl: 'vietc_dragon_esc_oem.stl'
    },
    {
      id: 'vietnamese_diacritic_caps',
      name: 'Bộ Ký Tự Nguyên Âm Tiếng Việt',
      letter: 'ă/â/đ/ê/ô/ơ/ư',
      desc: 'Trọn bộ keycap các chữ cái đặc trưng và bộ thanh dấu trong tiếng Việt dành cho hàng phím Alpha.',
      rarity: 'Epic',
      stlUrl: 'vietc_vietnamese_alphas.zip'
    },
    {
      id: 'numlock_dragon_plate',
      name: 'Tấm Ốp Phím Numlock 3D',
      letter: 'NUM',
      desc: 'Bản thiết kế ốp bàn phím số cơ phong cách Rồng Con đan xen các vảy rồng bảo vệ cực chất.',
      rarity: 'Rare',
      stlUrl: 'vietc_numlock_plate.stl'
    },
    {
      id: 'dragon_spacebar_625u',
      name: 'Thanh Spacebar Thủy Cung Rồng Con 6.25u',
      letter: 'SPACE',
      desc: 'Thanh phím dài uốn lượn phong cách rồng con bay lượn dưới đáy đại dương resin trong suốt.',
      rarity: 'Legendary',
      stlUrl: 'vietc_spacebar_dragon.stl'
    }
  ];

  const startDownload = (model: KeycapModel) => {
    if (downloadingId) return;
    setDownloadingId(model.id);
    setDownloadProgress(0);

    const interval = setInterval(() => {
      setDownloadProgress(p => {
        if (p >= 100) {
          clearInterval(interval);
          setTimeout(() => {
            setDownloadingId(null);
            setToastMessage(`Đã tải về thành công tệp thiết kế 3D: ${model.stlUrl}! Sẵn sàng để in 3D FDM/SLA.`);
            setShowSuccessToast(true);
            setTimeout(() => setShowSuccessToast(false), 4500);
          }, 400);
          return 100;
        }
        return p + 5 + Math.floor(Math.random() * 8);
      });
    }, 120);
  };

  // Compute CSS styles based on material
  const getMaterialStyles = () => {
    switch (custom.material) {
      case 'resin_frosted':
        return {
          backdropFilter: 'blur(8px)',
          background: `rgba(${hexToRgb(custom.baseColor)}, 0.45)`,
          border: '1px solid rgba(255, 255, 255, 0.25)',
          boxShadow: `inset 0 0 15px rgba(255, 255, 255, 0.3), 0 0 25px ${custom.ledColor}${Math.floor(custom.ledIntensity / 100 * 255).toString(16)}`
        };
      case 'glass':
        return {
          backdropFilter: 'blur(3px)',
          background: `rgba(${hexToRgb(custom.baseColor)}, 0.25)`,
          border: '1px solid rgba(255, 255, 255, 0.4)',
          boxShadow: `inset 0 0 20px rgba(255, 255, 255, 0.4), 0 0 35px ${custom.ledColor}${Math.floor(custom.ledIntensity / 100 * 255).toString(16)}`
        };
      case 'matte':
        return {
          background: custom.baseColor,
          border: '1px solid rgba(0, 0, 0, 0.3)',
          boxShadow: 'inset 0 4px 6px rgba(255, 255, 255, 0.1), inset 0 -4px 6px rgba(0, 0, 0, 0.2)'
        };
      default: // resin_clear
        return {
          backdropFilter: 'blur(1px)',
          background: `rgba(${hexToRgb(custom.baseColor)}, 0.65)`,
          border: '1px solid rgba(255, 255, 255, 0.35)',
          boxShadow: `inset 0 0 10px rgba(255, 255, 255, 0.4), 0 0 25px ${custom.ledColor}${Math.floor(custom.ledIntensity / 100 * 255).toString(16)}`
        };
    }
  };

  // Helper to convert hex to rgb
  function hexToRgb(hex: string): string {
    const bigint = parseInt(hex.replace('#', ''), 16);
    const r = (bigint >> 16) & 255;
    const g = (bigint >> 8) & 255;
    const b = bigint & 255;
    return `${r}, ${g}, ${b}`;
  }

  return (
    <div id="keycaps" className="py-16 bg-[#0a0b0d] border-t border-white/10 scroll-mt-20">
      <div className="max-w-6xl mx-auto px-4 sm:px-6">
        
        {/* Section Header */}
        <div className="text-center max-w-3xl mx-auto mb-12">
          <motion.div
            initial={{ opacity: 0, y: 10 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs font-mono mb-4"
          >
            <Sparkles size={12} className="text-emerald-400 animate-pulse" />
            <span>ARTISAN 3D KEYCAPS</span>
          </motion.div>
          
          <h2 className="text-3xl sm:text-4xl font-serif text-white tracking-tight">
            Trang Trí Phím Cơ <span className="text-transparent bg-clip-text bg-gradient-to-r from-emerald-400 to-teal-400 italic">VietC Resin 3D</span>
          </h2>
          <p className="mt-4 text-slate-400 text-sm sm:text-base leading-relaxed">
            Như công bố chính thức, VietC không chỉ là phần mềm gõ phím, chúng tôi chia sẻ bản vẽ thiết kế 3D hoàn toàn miễn phí của <span className="text-emerald-400 font-semibold">Mascot Rồng Con Resin trong suốt</span> và bộ ký tự dấu mũ tiếng Việt để bạn tự in 3D cá nhân hóa bàn phím cơ của mình!
          </p>
        </div>

        {/* WORKSPACE: Customizer on the left, interactive keycap on the right */}
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 items-center mb-16">
          
          {/* Controls Panel (6 cols) */}
          <div className="lg:col-span-6 bg-white/[0.02] p-5 sm:p-6 rounded-2xl border border-white/10 space-y-6">
            <div className="flex items-center gap-2 border-b border-white/5 pb-3">
              <Sliders size={18} className="text-emerald-400" />
              <h3 className="font-sans font-bold text-sm text-slate-200 tracking-wider uppercase">Bảng Điều Khiển Tùy Biến 3D</h3>
            </div>

            {/* Vải Màu Resin */}
            <div className="space-y-2">
              <label className="text-xs font-semibold text-slate-300 flex items-center gap-1.5">
                <Palette size={14} className="text-emerald-400" />
                <span>Màu Sắc Resin Bọc Ngoài</span>
              </label>
              <div className="grid grid-cols-3 gap-2">
                {colorPresets.map((preset) => (
                  <button
                    key={preset.value}
                    onClick={() => setCustom(prev => ({ ...prev, baseColor: preset.value }))}
                    className={`flex items-center gap-2 p-2 rounded-lg border text-[11px] font-medium transition-all cursor-pointer ${
                      custom.baseColor === preset.value
                        ? 'bg-white/5 border-emerald-500 text-white'
                        : 'bg-[#0d0e12] border-white/5 text-slate-400 hover:text-slate-200'
                    }`}
                  >
                    <span className="w-3 h-3 rounded-full border border-slate-700/50" style={{ backgroundColor: preset.value }} />
                    <span className="truncate">{preset.name}</span>
                  </button>
                ))}
              </div>
            </div>

            {/* Chất liệu Resin */}
            <div className="space-y-2">
              <label className="text-xs font-semibold text-slate-300 flex items-center gap-1.5">
                <Layers size={14} className="text-emerald-400" />
                <span>Chất Liệu Đúc Keycap</span>
              </label>
              <div className="grid grid-cols-4 gap-2 text-[10px] font-mono">
                {[
                  { id: 'resin_clear', name: 'Trong Suốt' },
                  { id: 'resin_frosted', name: 'Nhám Mờ' },
                  { id: 'glass', name: 'Thạch Anh' },
                  { id: 'matte', name: 'Nhựa Đục' }
                ].map((mat) => (
                  <button
                    key={mat.id}
                    onClick={() => setCustom(prev => ({ ...prev, material: mat.id as any }))}
                    className={`py-1.5 px-1 rounded-lg border text-center font-medium transition-all cursor-pointer ${
                      custom.material === mat.id
                        ? 'bg-emerald-500/10 border-emerald-500 text-emerald-400'
                        : 'bg-[#0d0e12] border-white/5 text-slate-400 hover:text-slate-200 hover:bg-white/5'
                    }`}
                  >
                    {mat.name}
                  </button>
                ))}
              </div>
            </div>

            {/* Ký Tự Tiếng Việt */}
            <div className="space-y-2">
              <label className="text-xs font-semibold text-slate-300 flex items-center gap-1.5">
                <Type size={14} className="text-emerald-400" />
                <span>Ký Tự / Dấu Thanh Tiếng Việt Khắc Trên Mặt Phím</span>
              </label>
              <div className="flex flex-wrap gap-1.5">
                {lettersList.map((letItem) => (
                  <button
                    key={letItem}
                    onClick={() => setCustom(prev => ({ ...prev, selectedLetter: letItem }))}
                    className={`w-10 h-10 rounded-lg border flex items-center justify-center font-sans font-bold text-sm transition-all cursor-pointer ${
                      custom.selectedLetter === letItem
                        ? 'bg-emerald-600 text-white border-emerald-400 shadow-[0_0_15px_rgba(16,185,129,0.35)]'
                        : 'bg-[#0d0e12] border-white/5 text-slate-400 hover:text-slate-200'
                    }`}
                  >
                    {letItem}
                  </button>
                ))}
              </div>
            </div>

            {/* Đèn LED gầm (Underglow) */}
            <div className="space-y-3 pt-2 border-t border-white/5">
              <div className="flex items-center justify-between">
                <label className="text-xs font-semibold text-slate-300 flex items-center gap-1.5">
                  <Lightbulb size={14} className="text-emerald-400" />
                  <span>Hệ Thống Đèn LED Gầm (PCB Underglow)</span>
                </label>
                <span className="text-[10px] font-mono text-emerald-400">{custom.ledIntensity}% Độ sáng</span>
              </div>
              
              <div className="grid grid-cols-5 gap-2">
                {ledPresets.map((preset) => (
                  <button
                    key={preset.value}
                    onClick={() => setCustom(prev => ({ ...prev, ledColor: preset.value }))}
                    className={`w-full h-7 rounded-md border flex items-center justify-center transition-all cursor-pointer ${
                      custom.ledColor === preset.value
                        ? 'border-white scale-110 shadow-lg'
                        : 'border-transparent opacity-65 hover:opacity-100'
                    }`}
                    style={{ backgroundColor: preset.value, boxShadow: custom.ledColor === preset.value ? `0 0 10px ${preset.value}` : 'none' }}
                    title={preset.name}
                  />
                ))}
              </div>

              <input
                type="range"
                min="0"
                max="100"
                value={custom.ledIntensity}
                onChange={(e) => setCustom(prev => ({ ...prev, ledIntensity: parseInt(e.target.value) }))}
                className="w-full h-1 bg-[#0d0e12] rounded-lg appearance-none cursor-pointer accent-emerald-500"
              />
            </div>

          </div>

          {/* Interactive 3D Render Viewer (6 cols) */}
          <div className="lg:col-span-6 flex flex-col items-center justify-center p-6 bg-white/[0.01] rounded-2xl border border-white/5 h-[420px] relative overflow-hidden group">
            
            {/* Grid background effect */}
            <div className="absolute inset-0 bg-[linear-gradient(to_right,#334155_1px,transparent_1px),linear-gradient(to_bottom,#334155_1px,transparent_1px)] bg-[size:24px_24px] opacity-10" />
            
            {/* Floating glowing indicator */}
            <div className="absolute top-4 right-4 bg-[#0d0e12]/80 px-2.5 py-1 rounded-full border border-emerald-500/20 text-[9px] font-mono text-emerald-400 flex items-center gap-1.5">
              <span className="w-1.5 h-1.5 rounded-full animate-pulse" style={{ backgroundColor: custom.ledColor }} />
              <span>3D RENDER PREVIEW</span>
            </div>

            {/* Rotating 3D Keycap Stage Container */}
            <div className="relative w-64 h-64 flex items-center justify-center perspective-[1000px]">
              
              {/* LED Underglow radial halo */}
              <div
                className="absolute w-48 h-48 rounded-full blur-3xl opacity-40 transition-all duration-300 pointer-events-none"
                style={{
                  backgroundColor: custom.ledColor,
                  transform: 'translateY(50px) scale(0.8)',
                  filter: `blur(45px)`,
                  opacity: (custom.ledIntensity / 100) * 0.5
                }}
              />

              {/* The Keycap Body Wrapper (3D effect) */}
              <motion.div
                className="w-40 h-40 relative transform-style-3d cursor-grab active:cursor-grabbing flex items-center justify-center"
                animate={{
                  rotateY: [0, 360],
                  rotateX: [12, 12]
                }}
                transition={{
                  rotateY: { repeat: Infinity, duration: 16, ease: 'linear' }
                }}
              >
                
                {/* 1. KEYCAP BASE / STEM (Inner structure visible through clear resin) */}
                {custom.material !== 'matte' && (
                  <div className="absolute inset-4 rounded-xl flex items-center justify-center border border-dashed border-slate-700/30 z-10 pointer-events-none">
                    {/* The mechanical switch cross stem (+) inside */}
                    <div className="relative w-8 h-8 flex items-center justify-center">
                      <div className="absolute w-2 h-7 rounded-sm shadow-md" style={{ backgroundColor: custom.stemColor }} />
                      <div className="absolute w-7 h-2 rounded-sm shadow-md" style={{ backgroundColor: custom.stemColor }} />
                    </div>
                  </div>
                )}

                {/* 2. CUTE MASCOT DRAGON RESTING INSIDE */}
                <div className="absolute inset-0 flex items-center justify-center z-20 pointer-events-none scale-65 translate-y-[-5px]">
                  <DragonMascot size={110} interactive={false} />
                </div>

                {/* 3. TRANSPARENT RESIN OUTER SHELL (Styled with standard custom glassmorphism) */}
                <div
                  className="absolute inset-0 rounded-2xl transition-all duration-300 z-30 flex flex-col justify-between p-4.5"
                  style={getMaterialStyles()}
                >
                  {/* Vietnamese Letter Engraving on top facet */}
                  <div className="text-right w-full">
                    <span className="font-sans font-extrabold text-2xl tracking-tight text-white drop-shadow-lg opacity-85 select-none">
                      {custom.selectedLetter.split(' ')[0]}
                    </span>
                  </div>

                  {/* Aesthetic detail: micro-bubbles or text inside */}
                  <div className="flex justify-between items-end w-full text-[8px] font-mono text-white/55">
                    <span>VIETC 3D</span>
                    <span>OEM-R1</span>
                  </div>
                </div>

                {/* Bottom Base rim */}
                <div className="absolute inset-[-4px] rounded-3xl border-2 border-slate-800/40 translate-y-[8px] scale-95 opacity-50 z-0" />

              </motion.div>

            </div>

            {/* Customizer Instructions */}
            <div className="absolute bottom-4 left-4 text-[10px] font-mono text-slate-500 flex items-center gap-1">
              <AlertCircle size={10} />
              <span>Góc xoay 3D giả lập trực quan 360°</span>
            </div>

          </div>

        </div>

        {/* 3D PRINTING FILES DOWNLOAD SECTION */}
        <div className="space-y-6">
          <h3 className="text-xl font-sans font-bold text-slate-100 flex items-center gap-2 border-b border-white/5 pb-3">
            <Download className="text-emerald-400 animate-bounce" size={18} />
            <span>Tải Về File Thiết Kế 3D Miễn Phí (STL/OBJ)</span>
          </h3>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
            {keycapModels.map((model) => (
              <div
                key={model.id}
                className="bg-white/[0.02] rounded-2xl border border-white/5 p-5 flex flex-col justify-between hover:border-emerald-500/30 transition-all group"
              >
                <div>
                  {/* Rarity and Rating info */}
                  <div className="flex items-center justify-between mb-3.5">
                    <span className={`text-[9px] font-mono px-2 py-0.5 rounded-full border ${
                      model.rarity === 'Legendary'
                        ? 'bg-amber-500/10 border-amber-500/20 text-amber-400'
                        : model.rarity === 'Epic'
                        ? 'bg-purple-500/10 border-purple-500/20 text-purple-400'
                        : 'bg-emerald-500/10 border-emerald-500/20 text-emerald-400'
                    }`}>
                      {model.rarity} Design
                    </span>
                    <div className="flex items-center gap-0.5 text-amber-400">
                      <Star size={10} fill="currentColor" />
                      <Star size={10} fill="currentColor" />
                      <Star size={10} fill="currentColor" />
                      <Star size={10} fill="currentColor" />
                      <Star size={10} fill="currentColor" />
                    </div>
                  </div>

                  <h4 className="text-sm font-sans font-bold text-slate-200 group-hover:text-emerald-400 transition-colors">
                    {model.name}
                  </h4>
                  <p className="text-slate-400 text-xs mt-2.5 leading-relaxed min-h-[48px]">
                    {model.desc}
                  </p>
                </div>

                <div className="mt-5 pt-4 border-t border-white/5">
                  <div className="flex items-center justify-between text-[11px] text-slate-500 font-mono mb-3">
                    <span>Định dạng: STL / STEP</span>
                    <span className="text-emerald-400 font-bold">FREE</span>
                  </div>

                  {downloadingId === model.id ? (
                    <div className="space-y-1.5">
                      <div className="flex justify-between text-[10px] font-mono text-emerald-400">
                        <span>Đang tải...</span>
                        <span>{downloadProgress}%</span>
                      </div>
                      <div className="w-full bg-[#0d0e12] h-1.5 rounded-full overflow-hidden">
                        <div className="bg-emerald-500 h-full transition-all duration-100" style={{ width: `${downloadProgress}%` }} />
                      </div>
                    </div>
                  ) : (
                    <button
                      onClick={() => startDownload(model)}
                      className="w-full py-2 rounded-lg bg-[#0d0e12] hover:bg-emerald-600 border border-white/10 hover:border-emerald-500 text-slate-300 hover:text-white font-sans font-bold text-xs transition-all flex items-center justify-center gap-1.5 cursor-pointer"
                    >
                      <Download size={13} />
                      Tải File STL
                    </button>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Global Action Toast Notification */}
        <AnimatePresence>
          {showSuccessToast && (
            <motion.div
              initial={{ opacity: 0, y: 50 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: 20 }}
              className="fixed bottom-6 right-6 z-50 max-w-sm bg-[#0d0e12] border border-emerald-500/30 p-4 rounded-xl shadow-2xl flex items-start gap-3"
            >
              <CheckCircle2 className="text-emerald-400 flex-shrink-0 mt-0.5" size={18} />
              <div>
                <h4 className="text-xs font-sans font-bold text-slate-200">Bắt đầu tải file 3D</h4>
                <p className="text-slate-400 text-[11px] mt-1 leading-relaxed">
                  {toastMessage}
                </p>
              </div>
            </motion.div>
          )}
        </AnimatePresence>

      </div>
    </div>
  );
}
