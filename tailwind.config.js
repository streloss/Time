/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        background: "#000000",
        surface: "#0E0E0E",
        "surface-container": "#181818",
        "surface-high": "#222222",
        "surface-highest": "#2E2E2E",
        "outline-variant": "#3A3A3A",
      },
      borderRadius: {
        '3xl': '24px',
        '4xl': '28px',
      },
      fontFamily: {
        sans: ['"Google Sans"', '"Inter"', '-apple-system', 'BlinkMacSystemFont', 'sans-serif'],
      },
    },
  },
  plugins: [],
}