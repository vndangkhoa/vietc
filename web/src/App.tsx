import React, { useState } from 'react';
import { motion, AnimatePresence } from 'motion/react';
import Navbar from './components/Navbar';
import Hero from './components/Hero';
import Features from './components/Features';
import TerminalSimulator from './components/TerminalSimulator';
import SetupGuide from './components/SetupGuide';
import KeycapGallery from './components/KeycapGallery';
import Footer from './components/Footer';

export default function App() {
  const [activeView, setActiveView] = useState<'home' | 'keycaps'>('home');

  return (
    <div className="min-h-screen bg-[#0a0b0d] text-slate-200 flex flex-col font-sans antialiased selection:bg-emerald-500/30 selection:text-white">
      
      {/* Dynamic Navigation bar */}
      <Navbar activeView={activeView} setActiveView={setActiveView} />

      {/* Main page content layout with view switcher transitions */}
      <main className="flex-1">
        <AnimatePresence mode="wait">
          {activeView === 'home' ? (
            <motion.div
              key="home-view"
              initial={{ opacity: 0, y: 15 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -15 }}
              transition={{ duration: 0.3, ease: 'easeInOut' }}
            >
              {/* Hero & Official Announcement Card */}
              <Hero setActiveView={setActiveView} />

              {/* Core technical pillars section */}
              <Features />

              {/* Live Interactive Terminal Simulator VNI Engine */}
              <TerminalSimulator />

              {/* Step-by-step Linux System-Level Setup Guide */}
              <SetupGuide />
            </motion.div>
          ) : (
            <motion.div
              key="keycaps-view"
              initial={{ opacity: 0, y: 15 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -15 }}
              transition={{ duration: 0.3, ease: 'easeInOut' }}
            >
              {/* Trang phụ: 3D Transparent Resin Keycap Customizer & Gallery */}
              <KeycapGallery />
            </motion.div>
          )}
        </AnimatePresence>
      </main>

      {/* Footer component with social repository links & author credits */}
      <Footer />
      
    </div>
  );
}
