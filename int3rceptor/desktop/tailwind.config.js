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

        // Extended Colors
        'i3-green': '#10b981',
        'i3-red': '#ef4444',
        'i3-yellow': '#f59e0b',
        'i3-blue': '#3b82f6',
        'i3-pink': '#ec4899',
        'i3-indigo': '#6366f1',
        'i3-teal': '#14b8a6',
        'i3-rose': '#f43f5e',
        'i3-amber': '#f59e0b',
        'i3-lime': '#84cc16',
        'i3-emerald': '#10b981',
        'i3-cyan-50': '#ecfeff',
        'i3-cyan-100': '#cffafe',
        'i3-cyan-200': '#a5f3fc',
        'i3-cyan-300': '#67e8f9',
        'i3-cyan-400': '#22d3ee',
        'i3-cyan-500': '#06b6d4',
        'i3-cyan-600': '#0891b2',
        'i3-cyan-700': '#0e7490',
        'i3-cyan-800': '#155e75',
        'i3-cyan-900': '#164e63',
        'i3-magenta-50': '#fdf2f8',
        'i3-magenta-100': '#fce7f3',
        'i3-magenta-200': '#fbcfe8',
        'i3-magenta-300': '#f9a8d4',
        'i3-magenta-400': '#f472b6',
        'i3-magenta-500': '#ec4899',
        'i3-magenta-600': '#db2777',
        'i3-magenta-700': '#be185d',
        'i3-magenta-800': '#9d174d',
        'i3-magenta-900': '#831843',
        'i3-orange-50': '#fffbeb',
        'i3-orange-100': '#fef3c7',
        'i3-orange-200': '#fde68a',
        'i3-orange-300': '#fcd34d',
        'i3-orange-400': '#fbbf24',
        'i3-orange-500': '#f59e0b',
        'i3-orange-600': '#d97706',
        'i3-orange-700': '#b45309',
        'i3-orange-800': '#92400e',
        'i3-orange-900': '#78350f',
        'i3-purple-50': '#f5f3ff',
        'i3-purple-100': '#ede9fe',
        'i3-purple-200': '#ddd6fe',
        'i3-purple-300': '#c4b5fd',
        'i3-purple-400': '#a78bfa',
        'i3-purple-500': '#8b5cf6',
        'i3-purple-600': '#7c3aed',
        'i3-purple-700': '#6d28d9',
        'i3-purple-800': '#5b21b6',
        'i3-purple-900': '#4c1d95',
      },
      fontFamily: {
        mono: ['Fira Code', 'JetBrains Mono', 'Consolas', 'monospace'],
        sans: ['Inter', 'Roboto', 'Segoe UI', 'sans-serif'],
        heading: ['Orbitron', 'Rajdhani', 'sans-serif'],
      },
      fontSize: {
        'xxs': ['0.625rem', { lineHeight: '0.75rem' }],
        '3xl': ['1.875rem', { lineHeight: '2.25rem' }],
        '4xl': ['2.25rem', { lineHeight: '2.5rem' }],
        '5xl': ['3rem', { lineHeight: '1' }],
        '6xl': ['3.75rem', { lineHeight: '1' }],
      },
      spacing: {
        '18': '4.5rem',
        '88': '22rem',
        '128': '32rem',
        '144': '36rem',
      },
      borderRadius: {
        '4xl': '2rem',
        '5xl': '2.5rem',
      },
      boxShadow: {
        // Glow Effects
        'glow-cyan': '0 0 10px rgba(0, 212, 255, 0.3)',
        'glow-cyan-lg': '0 0 20px rgba(0, 212, 255, 0.5)',
        'glow-cyan-xl': '0 0 30px rgba(0, 212, 255, 0.7)',
        'glow-magenta': '0 0 10px rgba(255, 0, 110, 0.3)',
        'glow-magenta-lg': '0 0 20px rgba(255, 0, 110, 0.5)',
        'glow-magenta-xl': '0 0 30px rgba(255, 0, 110, 0.7)',
        'glow-orange': '0 0 10px rgba(255, 184, 0, 0.3)',
        'glow-orange-lg': '0 0 20px rgba(255, 184, 0, 0.5)',
        'glow-orange-xl': '0 0 30px rgba(255, 184, 0, 0.7)',
        'glow-purple': '0 0 10px rgba(139, 92, 246, 0.3)',
        'glow-purple-lg': '0 0 20px rgba(139, 92, 246, 0.5)',
        'glow-purple-xl': '0 0 30px rgba(139, 92, 246, 0.7)',
        'glow-green': '0 0 10px rgba(16, 185, 129, 0.3)',
        'glow-red': '0 0 10px rgba(239, 68, 68, 0.3)',
        'glow-yellow': '0 0 10px rgba(245, 158, 11, 0.3)',

        // Elevation Shadows
        'elevation-sm': '0 2px 4px rgba(0, 0, 0, 0.3)',
        'elevation': '0 4px 12px rgba(0, 0, 0, 0.5)',
        'elevation-lg': '0 8px 24px rgba(0, 0, 0, 0.6)',
        'elevation-xl': '0 12px 32px rgba(0, 0, 0, 0.7)',

        // Neon Shadows
        'neon-cyan': '0 0 5px rgba(0, 212, 255, 0.5), 0 0 10px rgba(0, 212, 255, 0.3)',
        'neon-magenta': '0 0 5px rgba(255, 0, 110, 0.5), 0 0 10px rgba(255, 0, 110, 0.3)',
        'neon-orange': '0 0 5px rgba(255, 184, 0, 0.5), 0 0 10px rgba(255, 184, 0, 0.3)',
        'neon-purple': '0 0 5px rgba(139, 92, 246, 0.5), 0 0 10px rgba(139, 92, 246, 0.3)',

        // Inner Shadows
        'inner-glow-cyan': 'inset 0 0 10px rgba(0, 212, 255, 0.2)',
        'inner-glow-magenta': 'inset 0 0 10px rgba(255, 0, 110, 0.2)',
        'inner-glow-orange': 'inset 0 0 10px rgba(255, 184, 0, 0.2)',
        'inner-glow-purple': 'inset 0 0 10px rgba(139, 92, 246, 0.2)',
      },
      animation: {
        // Glow Animations
        'glow-cyan': 'glow-cyan 2s ease-in-out infinite',
        'glow-magenta': 'glow-magenta 2s ease-in-out infinite',
        'glow-orange': 'glow-orange 2s ease-in-out infinite',
        'glow-purple': 'glow-purple 2s ease-in-out infinite',

        // Pulse Variants
        'pulse-slow': 'pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite',
        'pulse-fast': 'pulse 1s cubic-bezier(0.4, 0, 0.6, 1) infinite',

        // Fade Animations
        'fade-in': 'fadeIn 0.3s ease-out',
        'fade-out': 'fadeOut 0.3s ease-out',
        'slide-in': 'slideIn 0.3s ease-out',
        'slide-out': 'slideOut 0.3s ease-out',

        // Spin Variants
        'spin-slow': 'spin 3s linear infinite',
        'spin-fast': 'spin 0.5s linear infinite',

        // Bounce Variants
        'bounce-slow': 'bounce 2s infinite',
        'bounce-fast': 'bounce 0.5s infinite',

        // Shake
        'shake': 'shake 0.5s cubic-bezier(.36,.07,.19,.97) both',

        // Float
        'float': 'float 3s ease-in-out infinite',

        // Gradient
        'gradient-x': 'gradient-x 3s ease infinite',
        'gradient-y': 'gradient-y 3s ease infinite',
        'gradient-xy': 'gradient-xy 3s ease infinite',
      },
      keyframes: {
        // Glow Keyframes
        'glow-cyan': {
          '0%, 100%': { boxShadow: '0 0 5px rgba(0, 212, 255, 0.3)' },
          '50%': { boxShadow: '0 0 20px rgba(0, 212, 255, 0.6)' },
        },
        'glow-magenta': {
          '0%, 100%': { boxShadow: '0 0 5px rgba(255, 0, 110, 0.3)' },
          '50%': { boxShadow: '0 0 20px rgba(255, 0, 110, 0.6)' },
        },
        'glow-orange': {
          '0%, 100%': { boxShadow: '0 0 5px rgba(255, 184, 0, 0.3)' },
          '50%': { boxShadow: '0 0 20px rgba(255, 184, 0, 0.6)' },
        },
        'glow-purple': {
          '0%, 100%': { boxShadow: '0 0 5px rgba(139, 92, 246, 0.3)' },
          '50%': { boxShadow: '0 0 20px rgba(139, 92, 246, 0.6)' },
        },

        // Fade Keyframes
        'fadeIn': {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        'fadeOut': {
          '0%': { opacity: '1' },
          '100%': { opacity: '0' },
        },
        'slideIn': {
          '0%': { transform: 'translateY(-10px)', opacity: '0' },
          '100%': { transform: 'translateY(0)', opacity: '1' },
        },
        'slideOut': {
          '0%': { transform: 'translateY(0)', opacity: '1' },
          '100%': { transform: 'translateY(-10px)', opacity: '0' },
        },

        // Shake Keyframes
        'shake': {
          '10%, 90%': { transform: 'translate3d(-1px, 0, 0)' },
          '20%, 80%': { transform: 'translate3d(2px, 0, 0)' },
          '30%, 50%, 70%': { transform: 'translate3d(-4px, 0, 0)' },
          '40%, 60%': { transform: 'translate3d(4px, 0, 0)' },
        },

        // Float Keyframes
        'float': {
          '0%, 100%': { transform: 'translateY(0)' },
          '50%': { transform: 'translateY(-10px)' },
        },

        // Gradient Keyframes
        'gradient-x': {
          '0%, 100%': {
            'background-size': '200% 200%',
            'background-position': 'left center'
          },
          '50%': {
            'background-size': '200% 200%',
            'background-position': 'right center'
          },
        },
        'gradient-y': {
          '0%, 100%': {
            'background-size': '200% 200%',
            'background-position': 'center top'
          },
          '50%': {
            'background-size': '200% 200%',
            'background-position': 'center bottom'
          },
        },
        'gradient-xy': {
          '0%, 100%': {
            'background-size': '400% 400%',
            'background-position': 'left center'
          },
          '50%': {
            'background-size': '400% 400%',
            'background-position': 'right center'
          },
        },
      },
      transitionDuration: {
        '0': '0ms',
        '100': '100ms',
        '150': '150ms',
        '200': '200ms',
        '300': '300ms',
        '400': '400ms',
        '500': '500ms',
        '600': '600ms',
        '700': '700ms',
        '800': '800ms',
        '900': '900ms',
        '1000': '1000ms',
      },
      transitionTimingFunction: {
        'bounce-in': 'cubic-bezier(0.68, -0.55, 0.265, 1.55)',
        'bounce-out': 'cubic-bezier(0.34, 1.56, 0.64, 1)',
        'ease-in-out-expo': 'cubic-bezier(0.87, 0, 0.13, 1)',
      },
      backdropBlur: {
        'xs': '2px',
      },
      zIndex: {
        '60': '60',
        '70': '70',
        '80': '80',
        '90': '90',
        '100': '100',
      },
      backgroundImage: {
        'gradient-radial': 'radial-gradient(var(--tw-gradient-stops))',
        'gradient-conic': 'conic-gradient(from 180deg at 50% 50%, var(--tw-gradient-stops))',
        'cyber-grid': `
          linear-gradient(rgba(0, 212, 255, 0.03) 1px, transparent 1px),
          linear-gradient(90deg, rgba(0, 212, 255, 0.03) 1px, transparent 1px)
        `,
        'hexagon': `
          url("data:image/svg+xml,%3Csvg width='28' height='49' viewBox='0 0 28 49' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M13.99 9.25l13 7.5v15l-13 7.5L1 31.75v-15l12.99-7.5zM3 17.9v12.7l10.99 6.34 11-6.35V17.9l-11-6.34L3 17.9zM0 15l12.98-7.5V0h-2v6.35L0 12.69v2.3zm0 18.5L12.98 41v8h-2v-6.85L0 35.81v-2.3zM15 0v7.5L27.99 15H28v-2.31h-.01L17 6.35V0h-2zm0 49v-8l12.99-7.5H28v2.31h-.01L17 42.15V49h-2z' fill='%2300d4ff' fill-opacity='0.03' fill-rule='evenodd'/%3E%3C/svg%3E")
        `,
      },
      backgroundSize: {
        'cyber-grid': '20px 20px',
      },
    },
  },
  plugins: [],
}
