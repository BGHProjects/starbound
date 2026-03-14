/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./src/**/*.rs", "./index.html"],
  theme: {
    extend: {
      colors: {
        navy: "#0a0f1e",
        navy2: "#0d1526",
        navy3: "#111d35",
        navy4: "#162040",
        border: "#1e2e50",
        orange: "#f4681a",
        orange2: "#e05510",
        muted: "#7a8aaa",
        dim: "#3a4e70",
      },
      fontFamily: {
        orbitron: ["Orbitron", "monospace"],
        exo: ["Exo 2", "sans-serif"],
      },
      keyframes: {
        "fade-up": {
          "0%": { opacity: "0", transform: "translateY(20px)" },
          "100%": { opacity: "1", transform: "translateY(0)" },
        },
        "fade-in": {
          "0%": { opacity: "0" },
          "100%": { opacity: "1" },
        },
        "slide-in-right": {
          "0%": { transform: "translateX(100%)", opacity: "0" },
          "100%": { transform: "translateX(0)", opacity: "1" },
        },
        "slide-in-left": {
          "0%": { transform: "translateX(-100%)", opacity: "0" },
          "100%": { transform: "translateX(0)", opacity: "1" },
        },
        "scale-in": {
          "0%": { opacity: "0", transform: "scale(0.95)" },
          "100%": { opacity: "1", transform: "scale(1)" },
        },
        "pulse-glow": {
          "0%,100%": { "box-shadow": "0 0 0px 0px rgba(244,104,26,0)" },
          "50%": { "box-shadow": "0 0 24px 4px rgba(244,104,26,0.25)" },
        },
        float: {
          "0%,100%": { transform: "translateY(0px)" },
          "50%": { transform: "translateY(-8px)" },
        },
        shimmer: {
          "0%": { "background-position": "-200% 0" },
          "100%": { "background-position": "200% 0" },
        },
      },
      animation: {
        "fade-up": "fade-up 0.45s cubic-bezier(0.4,0,0.2,1) forwards",
        "fade-in": "fade-in 0.3s ease forwards",
        "slide-in-right":
          "slide-in-right 0.35s cubic-bezier(0.4,0,0.2,1) forwards",
        "slide-in-left":
          "slide-in-left 0.35s cubic-bezier(0.4,0,0.2,1) forwards",
        "scale-in": "scale-in 0.25s cubic-bezier(0.4,0,0.2,1) forwards",
        "pulse-glow": "pulse-glow 2.5s ease-in-out infinite",
        float: "float 3s ease-in-out infinite",
        shimmer: "shimmer 1.8s linear infinite",
      },
    },
  },
  plugins: [],
};
