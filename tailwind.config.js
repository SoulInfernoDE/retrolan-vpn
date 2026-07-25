/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./ui/**/*.html",
    "./ui/**/*.ts",
  ],
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