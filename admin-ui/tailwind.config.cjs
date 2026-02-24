/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ['./index.html', './src/**/*.{ts,tsx,js,jsx}'],
  darkMode: ['class', '[data-theme="mermaidcore-dark"]'],
  theme: {
    extend: {
      colors: {
        ocean: {
          deep: '#020617',
          deeper: '#04101F',
        },
        surface: {
          glass: 'rgba(15, 23, 42, 0.55)',
        },
        accent: {
          aqua: '#4FD1C5',
          teal: '#14B8A6',
          lilac: '#A855F7',
          silver: '#E5E7EB',
        },
        text: {
          primary: '#F9FAFB',
          muted: '#9CA3AF',
        },
        status: {
          danger: '#FB7185',
          warn: '#FBBF24',
        },
      },
      borderRadius: {
        'mc-button': '999px',
        'mc-card': '18px',
      },
      boxShadow: {
        'mc-primary-glow': '0 0 24px rgba(79, 209, 197, 0.35)',
      },
      backgroundImage: {
        'mc-primary': 'linear-gradient(135deg, #4FD1C5 0%, #A855F7 60%, #E5E7EB 100%)',
        'mc-ocean': 'linear-gradient(to bottom, #020617, #04101F)',
      },
    },
  },
  plugins: [],
};
