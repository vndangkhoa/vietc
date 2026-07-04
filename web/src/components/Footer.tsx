import React from 'react';
import { Github, Heart, MessageSquare } from 'lucide-react';
import DragonMascot from './DragonMascot';

export default function Footer() {
  return (
    <footer className="bg-[#0a0b0d] border-t border-white/10 py-12 px-4">
      <div className="max-w-6xl mx-auto flex flex-col md:flex-row items-center justify-between gap-6">
        
        {/* Left branding */}
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 bg-white/[0.02] rounded-lg border border-white/10 flex items-center justify-center p-0.5">
            <DragonMascot size={34} interactive={false} />
          </div>
          <div className="text-left">
            <span className="font-sans font-black text-slate-100 text-lg tracking-wider block">
              VietC Project
            </span>
            <span className="text-[10px] font-mono text-slate-500 leading-none">
              Bàn phím cơ & Bộ gõ tiếng Việt mức thấp cho Linux Terminal
            </span>
          </div>
        </div>

        {/* Center Credits */}
        <div className="text-center md:text-right text-xs text-slate-500 font-mono space-y-1">
          <div>
            Phát triển bởi <a href="https://github.com/vndangkhoa" target="_blank" rel="noopener noreferrer" className="text-slate-400 hover:text-emerald-400 underline font-semibold">vndangkhoa</a>
          </div>
          <div className="flex items-center justify-center md:justify-end gap-1.5 text-[11px] text-slate-600">
            <span>Made with</span>
            <Heart size={10} className="text-rose-500 animate-pulse fill-rose-500" />
            <span>for Vietnamese Linux Community</span>
          </div>
        </div>

        {/* Right External Links */}
        <div className="flex items-center gap-4 text-slate-500">
          <a
            href="https://github.com/vndangkhoa/vietc"
            target="_blank"
            rel="noopener noreferrer"
            className="hover:text-emerald-400 transition-colors"
            title="GitHub Repository"
          >
            <Github size={18} />
          </a>
          <a
            href="https://github.com/vndangkhoa/vietc/issues"
            target="_blank"
            rel="noopener noreferrer"
            className="hover:text-emerald-400 transition-colors"
            title="Đóng góp ý kiến"
          >
            <MessageSquare size={18} />
          </a>
        </div>

      </div>

      <div className="max-w-6xl mx-auto mt-8 pt-6 border-t border-white/5 text-center text-[10px] font-mono text-slate-600">
        &copy; {new Date().getFullYear()} VietC. Phát hành theo Giấy phép Apache-2.0 / MIT.
      </div>
    </footer>
  );
}
