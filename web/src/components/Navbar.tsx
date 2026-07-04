import React from 'react';
import { motion } from 'motion/react';
import { Github, Key, Terminal, Code, Home, Sparkles } from 'lucide-react';
import DragonMascot from './DragonMascot';

interface NavbarProps {
  activeView: 'home' | 'keycaps';
  setActiveView: (view: 'home' | 'keycaps') => void;
}

export default function Navbar({ activeView, setActiveView }: NavbarProps) {
  const scrollToId = (id: string) => {
    // Switch to home first if on keycaps and clicking scroll targets
    if (activeView !== 'home') {
      setActiveView('home');
      setTimeout(() => {
        document.getElementById(id)?.scrollIntoView({ behavior: 'smooth' });
      }, 150);
    } else {
      document.getElementById(id)?.scrollIntoView({ behavior: 'smooth' });
    }
  };

  return (
    <nav className="sticky top-0 z-50 bg-[#0a0b0d]/90 backdrop-blur-md border-b border-white/10 px-6 h-20 flex items-center">
      <div className="w-full max-w-6xl mx-auto flex items-center justify-between">
        
        {/* LOGO AND BRANDING */}
        <div
          className="flex items-center gap-3 cursor-pointer select-none group"
          onClick={() => { setActiveView('home'); window.scrollTo({ top: 0, behavior: 'smooth' }); }}
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
            onClick={() => { setActiveView('home'); window.scrollTo({ top: 0, behavior: 'smooth' }); }}
            className={`hover:text-emerald-400 cursor-pointer transition-colors pb-1 border-b-2 ${
              activeView === 'home' ? 'text-emerald-400 border-emerald-400 font-bold' : 'border-transparent'
            }`}
          >
            Giới Thiệu
          </button>

          {activeView === 'home' && (
            <>
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
            </>
          )}

          <button
            onClick={() => setActiveView('keycaps')}
            className={`hover:text-emerald-400 cursor-pointer transition-all flex items-center gap-1.5 px-3 py-1.5 rounded-full border ${
              activeView === 'keycaps'
                ? 'bg-emerald-500/10 border-emerald-500/30 text-emerald-400 font-bold'
                : 'border-white/10 text-slate-400 hover:border-emerald-500/30'
            }`}
          >
            <Sparkles size={11} className={activeView === 'keycaps' ? 'animate-pulse text-emerald-400' : 'text-slate-500'} />
            Artisan Keycaps
          </button>

        </div>

        {/* EXTERNAL GITHUB BUTTON */}
        <div className="flex items-center gap-2">
          {/* Mobile view toggle */}
          <button
            onClick={() => setActiveView(activeView === 'home' ? 'keycaps' : 'home')}
            className="md:hidden text-[10px] font-bold px-3 py-1.5 rounded-md bg-white/5 border border-white/10 text-slate-300"
          >
            {activeView === 'home' ? 'Keycaps 3D' : 'Bộ Gõ VietC'}
          </button>

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
