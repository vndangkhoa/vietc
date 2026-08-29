import Navbar from './components/Navbar';
import Hero from './components/Hero';
import Features from './components/Features';
import TerminalSimulator from './components/TerminalSimulator';
import SetupGuide from './components/SetupGuide';
import Footer from './components/Footer';

export default function App() {
  return (
    <div className="min-h-screen bg-[#0a0b0d] text-slate-200 flex flex-col font-sans antialiased selection:bg-emerald-500/30 selection:text-white">
      
      {/* Navigation bar */}
      <Navbar />

      {/* Main page content */}
      <main className="flex-1">
        {/* Hero & Official Announcement Card */}
        <Hero />

        {/* Core technical pillars section */}
        <Features />

        {/* Live Interactive Terminal Simulator VNI Engine */}
        <TerminalSimulator />

        {/* Step-by-step Linux System-Level Setup Guide */}
        <SetupGuide />
      </main>

      {/* Footer component with social repository links & author credits */}
      <Footer />
      
    </div>
  );
}
