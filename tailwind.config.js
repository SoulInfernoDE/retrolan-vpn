/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{html,ts}",
    "./ui/index.html",
    "./ui/src/**/*.{html,ts}"
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        retrodark: "#0f172a",
        retrocard: "#1e293b",
        retrocyan: "#06b6d4",
        retrogreen: "#10b981",
      },
    },
  },
  plugins: [],
}
