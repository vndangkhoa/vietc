import React from 'react';
import { Github, Terminal } from 'lucide-react';
import DragonMascot from './DragonMascot';

export default function Navbar() {
  const scrollToId = (id: string) => {
    document.getElementById(id)?.scrollIntoView({ behavior: 'smooth' });
  };

  return (
    <nav className="sticky top-0 z-50 bg-[#0a0b0d]/90 backdrop-blur-md border-b border-white/10 px-6 h-20 flex items-center">
      <div className="w-full max-w-6xl mx-auto flex items-center justify-between">
        
        {/* LOGO AND BRANDING */}
        <div
          className="flex items-center gap-3 cursor-pointer select-none group"
          onClick={() => window.scrollTo({ top: 0, behavior: 'smooth' })}
        >
          <div className="w-10 h-10 bg-emerald-500 rounded-xl flex items-center justify-center shadow-[0_0_20px_rgba(16,185,129,0.35)] transition-transform group-hover:scale-105 duration-300">
            <DragonMascot size={32} interactive={false} />
          </div>
          <div className="flex flex-col">
            <span className="font-sans font-black text-2xl text-white tracking-tighter">
              VietC<span className="text-emerald-500">.</span>
            </span>
            <span className="text-[9px] font-mono text-emerald-500 font-bold -mt-1 tracking-widest uppercase">
              Native Linux IME
            </span>
          </div>
        </div>

        {/* NAVIGATION LINKS */}
        <div className="hidden md:flex items-center gap-8 text-xs font-semibold tracking-widest uppercase text-slate-400">
          
          <button
            onClick={() => window.scrollTo({ top: 0, behavior: 'smooth' })}
            className="hover:text-emerald-400 cursor-pointer transition-colors pb-1 border-b-2 border-transparent text-emerald-400 border-emerald-400 font-bold"
          >
            Giới Thiệu
          </button>

          <button
            onClick={() => scrollToId('features')}
            className="hover:text-emerald-400 cursor-pointer transition-colors pb-1 border-b-2 border-transparent"
          >
            Tính Năng
          </button>
              
          <button
            onClick={() => scrollToId('demo')}
            className="hover:text-emerald-400 cursor-pointer transition-colors pb-1 border-b-2 border-transparent flex items-center gap-1.5"
          >
            <Terminal size={12} className="text-emerald-500" />
            Giả Lập Demo
          </button>

          <button
            onClick={() => scrollToId('setup-guide')}
            className="hover:text-emerald-400 cursor-pointer transition-colors pb-1 border-b-2 border-transparent"
          >
            Setup Guide
          </button>

        </div>

        {/* EXTERNAL GITHUB BUTTON */}
        <div className="flex items-center gap-2">
          <a
            href="https://github.com/vndangkhoa/vietc"
            target="_blank"
            rel="noopener noreferrer"
            className="px-3.5 py-1.5 rounded-xl bg-white/5 border border-white/10 hover:border-emerald-500/30 text-slate-300 hover:text-emerald-400 transition-all flex items-center gap-1.5 text-xs font-semibold"
          >
            <Github size={14} />
            <span className="hidden sm:inline">GitHub</span>
          </a>
        </div>

      </div>
    </nav>
  );
}
