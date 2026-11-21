import type { Config } from 'tailwindcss';
import animate from 'tailwindcss-animate';

const config: Config = {
  darkMode: ['class'],
  content: ['./src/**/*.{ts,tsx}'],
  theme: {
    container: {
      center: true,
      padding: { DEFAULT: '1rem', sm: '1.5rem', lg: '2rem' },
      screens: { '2xl': '1440px' },
    },
    extend: {
      colors: {
        // Bảng màu FoodRandom
        primary: {
          DEFAULT: '#E85D78',
          50: '#FEF1F4',
          100: '#FCE0E6',
          200: '#F9C2CE',
          300: '#F49AAD',
          400: '#EE7790',
          500: '#E85D78',
          600: '#D43F5E',
          700: '#B22F4B',
          800: '#8F2840',
          900: '#6F2135',
          foreground: '#FFFFFF',
        },
        secondary: { DEFAULT: '#F4A261', foreground: '#30282A' },
        background: '#FFF3EF',
        surface: '#FFFFFF',
        dark: '#292124',
        ink: { DEFAULT: '#30282A', muted: '#7A6A6E', soft: '#A3959A' },
        border: '#EAD5D1',
        leaf: { DEFAULT: '#3F8F5B', soft: '#E6F4EA' },
      },
      fontFamily: {
        sans: ['var(--font-sans)', 'system-ui', 'sans-serif'],
        display: ['var(--font-display)', 'cursive'],
      },
      borderRadius: {
        xl: '0.875rem',
        '2xl': '1.125rem',
        '3xl': '1.5rem',
      },
      boxShadow: {
        card: '0 8px 30px -12px rgba(120, 60, 70, 0.25)',
        cta: '0 12px 28px -8px rgba(232, 93, 120, 0.65)',
        glass: '0 24px 60px -20px rgba(41, 33, 36, 0.55)',
      },
      keyframes: {
        shimmer: { '100%': { transform: 'translateX(100%)' } },
        'fade-up': { from: { opacity: '0', transform: 'translateY(8px)' }, to: { opacity: '1', transform: 'translateY(0)' } },
      },
      animation: {
        shimmer: 'shimmer 1.6s infinite',
        'fade-up': 'fade-up 0.35s ease-out both',
      },
    },
  },
  plugins: [animate],
};

export default config;
