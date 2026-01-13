/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        // INT3RCEPTOR Brand Colors
        'i3-bg': '#0a0a0f',
        'i3-bg-alt': '#1a1a2e',
        'i3-bg-panel': '#2a2a3e',
        'i3-cyan': '#00d4ff',
        'i3-magenta': '#ff006e',
        'i3-orange': '#ffb800',
        'i3-purple': '#8b5cf6',
        'i3-text': '#ffffff',
        'i3-text-secondary': '#a0a0a0',
        'i3-text-muted': '#606060',
        'i3-border': '#2a2a3e',
        'i3-border-active': '#00d4ff',
      },
      fontFamily: {
        mono: ['Fira Code', 'JetBrains Mono', 'Consolas', 'monospace'],
        sans: ['Inter', 'Roboto', 'Segoe UI', 'sans-serif'],
        heading: ['Orbitron', 'Rajdhani', 'sans-serif'],
      },
      boxShadow: {
        'glow-cyan': '0 0 10px rgba(0, 212, 255, 0.3)',
        'glow-cyan-lg': '0 0 20px rgba(0, 212, 255, 0.5)',
        'glow-magenta': '0 0 10px rgba(255, 0, 110, 0.3)',
        'glow-magenta-lg': '0 0 20px rgba(255, 0, 110, 0.5)',
        'elevation': '0 4px 12px rgba(0, 0, 0, 0.5)',
      },
      animation: {
        'glow-cyan': 'glow-cyan 2s ease-in-out infinite',
        'glow-magenta': 'glow-magenta 2s ease-in-out infinite',
        'pulse-slow': 'pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite',
      },
      keyframes: {
        'glow-cyan': {
          '0%, 100%': { boxShadow: '0 0 5px rgba(0, 212, 255, 0.3)' },
          '50%': { boxShadow: '0 0 20px rgba(0, 212, 255, 0.6)' },
        },
        'glow-magenta': {
          '0%, 100%': { boxShadow: '0 0 5px rgba(255, 0, 110, 0.3)' },
          '50%': { boxShadow: '0 0 20px rgba(255, 0, 110, 0.6)' },
        },
      },
      transitionDuration: {
        '0': '0ms',
        '150': '150ms',
        '300': '300ms',
        '500': '500ms',
      },
    },
  },
  plugins: [],
}
