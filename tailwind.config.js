/** @type {import('tailwindcss').Config} */
export default { content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"], theme: { extend: { fontFamily: { sans: ["Inter", "Segoe UI", "sans-serif"], display: ["Georgia", "serif"] }, colors: { ink: "#080d17", panel: "#101827", gold: "#d9ae62" }, boxShadow: { glow: "0 0 40px rgba(217,174,98,.12)" } } }, plugins: [] };
