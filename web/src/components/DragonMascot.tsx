import React, { useState } from 'react';
import { motion } from 'motion/react';

interface DragonMascotProps {
  className?: string;
  size?: number;
  interactive?: boolean;
}

export default function DragonMascot({ className = '', size = 150, interactive = true }: DragonMascotProps) {
  const [isClicked, setIsClicked] = useState(false);
  const [isHovered, setIsHovered] = useState(false);

  const handleClick = () => {
    if (!interactive) return;
    setIsClicked(true);
    setTimeout(() => setIsClicked(false), 800);
  };

  return (
    <div
      className={`relative select-none flex flex-col items-center justify-center ${className}`}
      style={{ width: size, height: size }}
      onClick={handleClick}
      onMouseEnter={() => interactive && setIsHovered(true)}
      onMouseLeave={() => interactive && setIsHovered(false)}
    >
      <motion.svg
        viewBox="0 0 200 200"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
        className="w-full h-full drop-shadow-xl"
        animate={{
          y: isHovered ? [0, -6, 0] : [0, -2, 0],
          rotate: isClicked ? [0, -8, 8, -4, 4, 0] : 0,
          scale: isClicked ? [1, 1.12, 0.96, 1.04, 1] : 1,
        }}
        transition={{
          y: { repeat: Infinity, duration: isHovered ? 1.2 : 3, ease: "easeInOut" },
          rotate: { duration: 0.5 },
          scale: { duration: 0.5 }
        }}
      >
        {/* Left ear */}
        <motion.path
          d="M38 72 C 15 45, 12 85, 34 108 Z"
          fill="#1E3A5F"
          animate={{ rotate: isHovered ? -3 : 0 }}
          transition={{ duration: 0.3 }}
        />
        <path d="M36 70 C 20 52, 18 80, 32 100 Z" fill="#2563EB" opacity="0.6" />

        {/* Right ear */}
        <motion.path
          d="M162 72 C 185 45, 188 85, 166 108 Z"
          fill="#1E3A5F"
          animate={{ rotate: isHovered ? 3 : 0 }}
          transition={{ duration: 0.3 }}
        />
        <path d="M164 70 C 180 52, 182 80, 168 100 Z" fill="#2563EB" opacity="0.6" />

        {/* Main head - bigger, fills more of the viewbox */}
        <ellipse cx="100" cy="100" rx="62" ry="58" fill="#3B82F6" />

        {/* Head highlight */}
        <ellipse cx="100" cy="72" rx="44" ry="22" fill="#60A5FA" opacity="0.3" />

        {/* Cheeks */}
        <ellipse cx="48" cy="112" rx="18" ry="20" fill="#3B82F6" />
        <ellipse cx="152" cy="112" rx="18" ry="20" fill="#3B82F6" />

        {/* Horns */}
        <motion.path
          d="M76 44 C 65 16, 84 6, 90 30 C 92 38, 88 44, 82 46 Z"
          fill="#1E3A5F"
          stroke="#0F2942"
          strokeWidth="1"
          animate={{ rotate: isHovered ? -3 : 0 }}
        />
        <path d="M80 40 C 72 22, 80 14, 86 34 Z" fill="#2563EB" opacity="0.5" />

        <motion.path
          d="M124 44 C 135 16, 116 6, 110 30 C 108 38, 112 44, 118 46 Z"
          fill="#1E3A5F"
          stroke="#0F2942"
          strokeWidth="1"
          animate={{ rotate: isHovered ? 3 : 0 }}
        />
        <path d="M120 40 C 128 22, 120 14, 114 34 Z" fill="#2563EB" opacity="0.5" />

        {/* Forehead ridges */}
        <path d="M82 52 C 90 45, 110 45, 118 52" stroke="#2563EB" strokeWidth="2.5" strokeLinecap="round" fill="none" opacity="0.5" />
        <path d="M86 60 C 92 55, 108 55, 114 60" stroke="#2563EB" strokeWidth="2" strokeLinecap="round" fill="none" opacity="0.35" />

        {/* Snout */}
        <ellipse cx="100" cy="120" rx="28" ry="20" fill="#60A5FA" />
        <path d="M76 114 C 86 106, 114 106, 124 114 C 118 118, 82 118, 76 114 Z" fill="#93C5FD" opacity="0.3" />

        {/* Nostrils */}
        <ellipse cx="91" cy="117" rx="3" ry="3.5" fill="#2563EB" />
        <ellipse cx="109" cy="117" rx="3" ry="3.5" fill="#2563EB" />

        {/* Blush */}
        <ellipse cx="60" cy="120" rx="12" ry="6" fill="#F87171" opacity="0.35" />
        <ellipse cx="140" cy="120" rx="12" ry="6" fill="#F87171" opacity="0.35" />

        {/* Eyes */}
        <g>
          <ellipse cx="78" cy="92" rx="16" ry="18" fill="#111827" />
          <ellipse cx="78" cy="92" rx="13" ry="15" fill="#0D9488" />
          <ellipse cx="78" cy="92" rx="8" ry="10" fill="#111827" />
          <motion.circle
            cx="72" cy="85" r="5" fill="#FFFFFF"
            animate={{ scale: isHovered ? [1, 1.25, 1] : 1 }}
            transition={{ repeat: Infinity, duration: 1.8 }}
          />
          <circle cx="85" cy="99" r="2.2" fill="#FFFFFF" opacity="0.7" />
          <circle cx="71" cy="97" r="1.4" fill="#FFFFFF" opacity="0.4" />
          <motion.path
            d="M60 72 H 96 V 94 H 60 Z"
            fill="#3B82F6"
            transformOrigin="78px 72px"
            animate={{ scaleY: [0, 0, 1, 0, 0, 0, 1, 0] }}
            transition={{ repeat: Infinity, duration: 3.5, times: [0, 0.4, 0.45, 0.5, 0.85, 0.9, 0.95, 1] }}
          />
        </g>
        <g>
          <ellipse cx="122" cy="92" rx="16" ry="18" fill="#111827" />
          <ellipse cx="122" cy="92" rx="13" ry="15" fill="#0D9488" />
          <ellipse cx="122" cy="92" rx="8" ry="10" fill="#111827" />
          <motion.circle
            cx="116" cy="85" r="5" fill="#FFFFFF"
            animate={{ scale: isHovered ? [1, 1.25, 1] : 1 }}
            transition={{ repeat: Infinity, duration: 1.8 }}
          />
          <circle cx="129" cy="99" r="2.2" fill="#FFFFFF" opacity="0.7" />
          <circle cx="115" cy="97" r="1.4" fill="#FFFFFF" opacity="0.4" />
          <motion.path
            d="M104 72 H 140 V 94 H 104 Z"
            fill="#3B82F6"
            transformOrigin="122px 72px"
            animate={{ scaleY: [0, 0, 1, 0, 0, 0, 1, 0] }}
            transition={{ repeat: Infinity, duration: 3.5, times: [0, 0.4, 0.45, 0.5, 0.85, 0.9, 0.95, 1] }}
          />
        </g>

        {/* Smile */}
        {isHovered || isClicked ? (
          <g>
            <path d="M82 124 C 82 138, 118 138, 118 124 Z" fill="#991B1B" />
            <path d="M86 130 C 90 134, 110 134, 114 130 Z" fill="#FCA5A5" />
            <path d="M84 124 L 88 128 L 92 124 Z" fill="#FFFFFF" />
            <path d="M116 124 L 112 128 L 108 124 Z" fill="#FFFFFF" />
          </g>
        ) : (
          <g>
            <path d="M84 124 C 88 130, 98 128, 100 124 C 102 128, 112 130, 116 124" stroke="#2563EB" strokeWidth="2.5" strokeLinecap="round" fill="none" />
            <path d="M88 123 L 90 125 L 92 123 Z" fill="#FFFFFF" opacity="0.7" />
            <path d="M112 123 L 110 125 L 108 123 Z" fill="#FFFFFF" opacity="0.7" />
          </g>
        )}
      </motion.svg>

      {interactive && (
        <motion.div
          className="absolute -top-6 bg-[#0d0e12] text-emerald-400 text-[10px] font-mono px-2 py-0.5 rounded-full border border-emerald-500/20 shadow-md pointer-events-none"
          initial={{ opacity: 0, scale: 0.8 }}
          animate={{
            opacity: isHovered ? 1 : 0,
            scale: isHovered ? 1 : 0.8,
            y: isHovered ? -4 : 0,
          }}
          transition={{ duration: 0.2 }}
        >
          {isClicked ? "Rawrr! ^_^" : "Click me!"}
        </motion.div>
      )}
    </div>
  );
}
