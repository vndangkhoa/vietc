import React, { useState, useEffect, useRef } from 'react';
import { motion, AnimatePresence } from 'motion/react';
import { Terminal, Send, Play, RefreshCw, Zap, Cpu, Lock, HelpCircle } from 'lucide-react';
import { parseVni } from '../utils/vniParser';
import { TerminalLog } from '../types';

export default function TerminalSimulator() {
  const [inputText, setInputText] = useState('');
  const [typedOutput, setTypedOutput] = useState('');
  const [imeState, setImeState] = useState('S0');
  const [terminalLogs, setTerminalLogs] = useState<TerminalLog[]>([]);
  const [isTypingDemo, setIsTypingDemo] = useState(false);
  const logEndRef = useRef<HTMLDivElement>(null);
  const logContainerRef = useRef<HTMLDivElement>(null);

  // Suggested pre-recorded typing strings (VNI sequence)
  const presets = [
    { label: "Gõ 'Việt Nam'", code: "Vie6t5 Nam" },
    { label: "Gõ 'tiếng việt'", code: "tie61ng vie6t5" },
    { label: "Gõ 'đường sá'", code: "d9uo7ng2 sa1" },
    { label: "Gõ 'rồng con'", code: "ro6ng2 con" },
  ];

  // Process live input
  useEffect(() => {
    const result = parseVni(inputText);
    setTypedOutput(result.text);
    setImeState(result.state);
    
    // Convert string logs into TerminalLog structures
    const parsedLogs: TerminalLog[] = result.logs.map((logStr, idx) => ({
      id: `log-${idx}-${Date.now()}`,
      type: logStr.includes('Diffing') ? 'diff' : logStr.includes('uinput') ? 'ime_state' : 'system',
      text: logStr,
      timestamp: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })
    }));
    setTerminalLogs(parsedLogs);
  }, [inputText]);

  // Auto-scroll the log container to the bottom when new events arrive.
  // Uses scrollTop on the container (never scrollIntoView, which scrolls the page).
  useEffect(() => {
    if (logContainerRef.current) {
      logContainerRef.current.scrollTop = logContainerRef.current.scrollHeight;
    }
  }, [terminalLogs]);

  // Simulate automated character-by-character typing demo
  const startDemo = async (vniString: string) => {
    if (isTypingDemo) return;
    setIsTypingDemo(true);
    setInputText('');
    
    let current = '';
    for (let i = 0; i < vniString.length; i++) {
      await new Promise(resolve => setTimeout(resolve, 200 + Math.random() * 150));
      current += vniString[i];
      setInputText(current);
    }
    setIsTypingDemo(false);
  };

  const handleClear = () => {
    setInputText('');
    setTypedOutput('');
    setImeState('S0');
    setTerminalLogs([]);
  };

  return (
    <div id="demo" className="py-16 bg-[#0a0b0d] border-t border-white/10 scroll-mt-20">
      <div className="max-w-6xl mx-auto px-4 sm:px-6">
        
        {/* Section Header */}
        <div className="text-center max-w-3xl mx-auto mb-12">
          <motion.div
            initial={{ opacity: 0, y: 10 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs font-mono mb-4"
          >
            <Zap size={12} className="text-emerald-400 animate-pulse" />
            <span>INTERACTIVE EXPERIMENT</span>
          </motion.div>
          
          <h2 className="text-3xl sm:text-4xl font-serif text-white tracking-tight">
            Trải Nghiệm <span className="text-transparent bg-clip-text bg-gradient-to-r from-emerald-400 to-teal-400 italic">VietC Simulator</span>
          </h2>
          <p className="mt-4 text-slate-400 text-sm sm:text-base">
            Hãy tự tay gõ chuỗi phím VNI hoặc chọn các mẫu gõ nhanh dưới đây để xem cách State Machine của VietC biên dịch và gửi tín hiệu trực tiếp lên Linux Terminal ảo cực mượt.
          </p>
        </div>

        {/* Interactive Workspace Grid */}
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 items-stretch">
          
          {/* LEFT: The Linux Terminal Emulator (7 cols) */}
          <div className="lg:col-span-7 flex flex-col h-[480px] bg-[#0d0e12] rounded-2xl border border-white/10 shadow-2xl overflow-hidden relative">
            
            {/* Terminal Window Chrome Title bar */}
            <div className="flex items-center justify-between px-4 py-3 bg-[#0a0b0d] border-b border-white/10 flex-shrink-0">
              <div className="flex items-center gap-2">
                <div className="w-3 h-3 rounded-full bg-rose-500/60" />
                <div className="w-3 h-3 rounded-full bg-amber-500/60" />
                <div className="w-3 h-3 rounded-full bg-emerald-500/60" />
                <span className="ml-2 text-xs text-slate-400 font-mono flex items-center gap-1.5 font-semibold">
                  <Terminal size={12} className="text-emerald-500" />
                  vietc@linuxmint-terminal: ~
                </span>
              </div>
              <div className="flex items-center gap-3">
                <span className="text-[10px] font-mono bg-emerald-500/10 text-emerald-400 px-2 py-0.5 rounded border border-emerald-500/20">
                  VIETC: ON (Double Shift)
                </span>
              </div>
            </div>

            {/* Terminal Screen Body */}
            <div className="flex-1 p-5 overflow-y-auto font-mono text-sm leading-relaxed space-y-4 select-text">
              <div className="text-slate-500 text-xs border-b border-white/5 pb-3 leading-relaxed">
                <div>VietC uinput Emulator Engine v1.2.0 (x86_64-linux-mint)</div>
                <div>Trạng thái: Hoạt động trực tiếp ở driver nhân (kernel space)...</div>
                <div>Gõ phím số 1-9 để gõ dấu VNI (vd: 'ro6ngs2' hoặc 'ro6ng2' &rarr; rồng).</div>
              </div>

              {/* History Console Feed */}
              <div className="space-y-2">
                <div className="text-slate-500"># Gõ tiếng Việt cực nhanh không cần DBus/IBus</div>
                <div className="flex items-start gap-1">
                  <span className="text-emerald-500">user@mint:~$</span>
                  <span className="text-slate-300">cat vietc_stats.txt</span>
                </div>
                <div className="text-emerald-400/90 pl-4 text-xs space-y-1 bg-white/[0.01] p-2.5 rounded border border-white/5">
                  <div>+ Keystroke Latency: 0ms (Mức phần cứng)</div>
                  <div>+ Press-Release Latency: &lt;1ms (Driver-level)</div>
                  <div>+ Event Type: evdev grab / virtual uinput raw keypress</div>
                  <div>+ Memory footprint: ~1.2 MB</div>
                </div>
              </div>

              {/* Interactive Terminal Line */}
              <div className="pt-2 border-t border-white/5">
                <div className="flex items-start gap-2">
                  <span className="text-emerald-400 font-mono text-sm whitespace-nowrap shrink-0 pt-0.5">
                    vietc@linuxmint-terminal:~$
                  </span>
                  <input
                    type="text"
                    value={inputText}
                    onChange={(e) => setInputText(e.target.value)}
                    disabled={isTypingDemo}
                    placeholder="Gõ VNI tại đây (vd: Vie6t1 Nam)..."
                    className="flex-1 bg-transparent border-none text-slate-100 placeholder-slate-600 focus:outline-none font-mono text-sm"
                    autoFocus
                  />
                  {inputText && (
                    <button
                      onClick={handleClear}
                      className="p-1 rounded hover:bg-white/5 text-slate-500 hover:text-slate-300 transition-colors cursor-pointer shrink-0"
                      title="Clear"
                    >
                      <RefreshCw size={12} />
                    </button>
                  )}
                </div>
              </div>

              {/* Converted Output */}
              <div className="flex items-start gap-2 ml-0">
                <span className="text-slate-500 font-mono text-sm shrink-0 pt-0.5 select-none">
                  &gt;
                </span>
                <span className="text-emerald-300 font-mono text-sm break-all">
                  {typedOutput || <span className="text-slate-600 italic">Kết quả tiếng Việt sẽ hiện ở đây...</span>}
                </span>
                {typedOutput && (
                  <span className="inline-block w-2 h-4 bg-emerald-400 animate-pulse ml-0.5 shrink-0 mt-0.5" />
                )}
              </div>

            </div>

            {/* Quick Demo bar */}
            <div className="p-3 bg-[#0a0b0d] border-t border-white/10 flex flex-wrap gap-2 items-center flex-shrink-0">
              <span className="text-[10px] text-slate-500 font-mono mr-1">Gợi ý gõ nhanh:</span>
              {presets.map((preset, idx) => (
                <button
                  key={idx}
                  onClick={() => startDemo(preset.code)}
                  disabled={isTypingDemo}
                  className="px-2.5 py-1 rounded bg-white/5 hover:bg-white/10 border border-white/10 hover:border-emerald-500/30 text-slate-300 hover:text-white font-mono text-[10px] transition-all flex items-center gap-1 cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  <Play size={8} className="text-emerald-400" />
                  {preset.label}
                </button>
              ))}
            </div>
          </div>

          {/* RIGHT: Real-time Monitor & Event Logs (5 cols) */}
          <div className="lg:col-span-5 flex flex-col bg-white/[0.02] rounded-2xl border border-white/10 p-5 lg:p-6 shadow-xl relative">
            
            {/* Header */}
            <div className="flex items-center justify-between pb-4 border-b border-white/5 mb-4">
              <div className="flex items-center gap-2">
                <Cpu size={16} className="text-emerald-400" />
                <span className="font-sans font-bold text-sm text-slate-200 tracking-wide uppercase">Màn Hình Kiểm Soát VietC</span>
              </div>
              <div className="flex items-center gap-1 bg-emerald-500/10 text-emerald-400 px-2 py-0.5 rounded-full text-[10px] font-mono border border-emerald-500/20">
                <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-ping" />
                <span>Live Monitor</span>
              </div>
            </div>

            {/* State Machine Visualization */}
            <div className="bg-[#0d0e12] p-4 rounded-xl border border-white/5 mb-5">
              <div className="text-xs text-slate-400 font-semibold mb-3 flex items-center justify-between">
                <span>Deterministic State Machine</span>
                <span className="text-[10px] font-mono text-slate-500">Sự thay đổi trạng thái gốc</span>
              </div>
              
              <div className="flex items-center justify-between px-2 py-1.5 relative">
                {/* Horizontal progress background bar */}
                <div className="absolute left-6 right-6 top-[22px] h-0.5 bg-white/5 z-0" />
                
                {/* S0 */}
                <div className="flex flex-col items-center z-10">
                  <div className={`w-8 h-8 rounded-full border flex items-center justify-center font-mono text-xs font-bold transition-all duration-300 ${
                    imeState === 'S0'
                      ? 'bg-emerald-600 text-white border-emerald-400 shadow-[0_0_15px_rgba(16,185,129,0.35)] scale-110'
                      : 'bg-[#0a0b0d] text-slate-500 border-white/5'
                  }`}>
                    S0
                  </div>
                  <span className="text-[9px] font-mono text-slate-500 mt-1.5">Chờ phím</span>
                </div>

                {/* S1 */}
                <div className="flex flex-col items-center z-10">
                  <div className={`w-8 h-8 rounded-full border flex items-center justify-center font-mono text-xs font-bold transition-all duration-300 ${
                    imeState === 'S1'
                      ? 'bg-emerald-600 text-white border-emerald-400 shadow-[0_0_15px_rgba(16,185,129,0.35)] scale-110'
                      : 'bg-[#0a0b0d] text-slate-500 border-white/5'
                  }`}>
                    S1
                  </div>
                  <span className="text-[9px] font-mono text-slate-500 mt-1.5">Nguyên âm</span>
                </div>

                {/* S2 */}
                <div className="flex flex-col items-center z-10">
                  <div className={`w-8 h-8 rounded-full border flex items-center justify-center font-mono text-xs font-bold transition-all duration-300 ${
                    imeState === 'S2'
                      ? 'bg-emerald-600 text-white border-emerald-400 shadow-[0_0_15px_rgba(16,185,129,0.35)] scale-110'
                      : 'bg-[#0a0b0d] text-slate-500 border-white/5'
                  }`}>
                    S2
                  </div>
                  <span className="text-[9px] font-mono text-slate-500 mt-1.5">Dấu thanh</span>
                </div>

                {/* S3 */}
                <div className="flex flex-col items-center z-10">
                  <div className={`w-8 h-8 rounded-full border flex items-center justify-center font-mono text-xs font-bold transition-all duration-300 ${
                    imeState === 'S3'
                      ? 'bg-emerald-600 text-white border-emerald-400 shadow-[0_0_15px_rgba(16,185,129,0.35)] scale-110'
                      : 'bg-[#0a0b0d] text-slate-500 border-white/5'
                  }`}>
                    S3
                  </div>
                  <span className="text-[9px] font-mono text-slate-500 mt-1.5">Ký tự phụ</span>
                </div>
              </div>
            </div>

            {/* Core Specs metrics */}
            <div className="grid grid-cols-3 gap-2.5 mb-5 font-mono text-center">
              <div className="bg-[#0d0e12] p-2.5 rounded-lg border border-white/5">
                <div className="text-[9px] text-slate-500">Keystroke</div>
                <div className="text-sm font-bold text-emerald-400 mt-0.5">0 ms</div>
              </div>
              <div className="bg-[#0d0e12] p-2.5 rounded-lg border border-white/5">
                <div className="text-[9px] text-slate-500">Press-Release</div>
                <div className="text-sm font-bold text-emerald-400 mt-0.5">&lt;1 ms</div>
              </div>
              <div className="bg-[#0d0e12] p-2.5 rounded-lg border border-white/5">
                <div className="text-[9px] text-slate-500">Clipboard</div>
                <div className="text-sm font-bold text-emerald-400 mt-0.5">1 ms</div>
              </div>
            </div>

            {/* Event Log Stream */}
            <div className="text-xs text-slate-400 font-semibold mb-2 flex items-center justify-between">
              <span>Sự Kiện Thiết Bị Thấp (uinput Event Logs)</span>
              <span className="text-[10px] font-mono text-slate-500">Thời gian thực</span>
            </div>

            <div ref={logContainerRef} className="flex-1 bg-[#0d0e12] p-4 rounded-xl border border-white/5 overflow-y-auto h-[170px] font-mono text-[11px] space-y-2.5">
              <AnimatePresence initial={false}>
                {terminalLogs.map((log) => (
                  <motion.div
                    key={log.id}
                    initial={{ opacity: 0, x: 5 }}
                    animate={{ opacity: 1, x: 0 }}
                    exit={{ opacity: 0 }}
                    className="border-b border-white/5 pb-2 last:border-none"
                  >
                    <div className="flex items-center justify-between text-[9px] text-slate-500 mb-0.5">
                      <span className="flex items-center gap-1">
                        <span className={`w-1 h-1 rounded-full ${
                          log.type === 'diff' ? 'bg-emerald-400' : log.type === 'ime_state' ? 'bg-teal-400' : 'bg-slate-400'
                        }`} />
                        {log.type.toUpperCase()}
                      </span>
                      <span>{log.timestamp}</span>
                    </div>
                    <div className={
                      log.type === 'diff' ? 'text-emerald-300' : log.type === 'ime_state' ? 'text-teal-200' : 'text-slate-300'
                    }>
                      {log.text}
                    </div>
                  </motion.div>
                ))}
              </AnimatePresence>
              
              {terminalLogs.length === 0 && (
                <div className="h-full flex items-center justify-center text-slate-600 italic">
                  Gõ phím hoặc chọn mẫu để hiển thị logs sự kiện nhân (kernel event logs)
                </div>
              )}
              <div ref={logEndRef} />
            </div>

            {/* Privacy note */}
            <div className="mt-4 flex gap-2 items-center text-[10px] text-slate-500 bg-[#0d0e12] p-2.5 rounded-lg border border-white/10">
              <Lock size={12} className="text-emerald-500 flex-shrink-0" />
              <span>An toàn & Bảo mật: VietC thu thập sự kiện phím tại local và không bao giờ gửi bất kỳ dữ liệu nào qua mạng Internet.</span>
            </div>

          </div>

        </div>

      </div>
    </div>
  );
}
