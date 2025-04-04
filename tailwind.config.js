/** @type {import('tailwindcss').Config} */
module.exports = {
  mode: "all",
  content: ["./src/**/*.{rs,html,css}", "./dist/**/*.html"],
  theme: {
    extend: {
      colors: {
        // Primary accent - more muted slate-blue
        primary: {
          DEFAULT: "#6b7280", // gray-500
          50: "#f8fafc",
          100: "#f1f5f9",
          200: "#e2e8f0",
          300: "#cbd5e1",
          400: "#94a3b8",
          500: "#6b7280",
          600: "#4b5563",
          700: "#374151",
          800: "#1e293b",
          900: "#0f172a",
          950: "#020617",
        },
        // Secondary accents - soft teal/green tones
        secondary: {
          DEFAULT: "#475569", // slate-600
          50: "#f8fafc",
          100: "#f1f5f9",
          200: "#e2e8f0",
          300: "#cbd5e1",
          400: "#94a3b8",
          500: "#64748b",
          600: "#475569",
          700: "#334155",
          800: "#1e293b",
          900: "#0f172a",
          950: "#020617",
        },
        // Success colors - muted sage green
        success: {
          DEFAULT: "#84cc16", // lime-500
          50: "#f7fee7",
          100: "#ecfccb",
          200: "#d9f99d",
          300: "#bef264",
          400: "#a3e635",
          500: "#84cc16",
          600: "#65a30d",
          700: "#4d7c0f",
          800: "#3f6212",
          900: "#365314",
          950: "#1a2e05",
        },
        // Warning colors - soft amber
        warning: {
          DEFAULT: "#d97706", // amber-600
          50: "#fffbeb",
          100: "#fef3c7",
          200: "#fde68a",
          300: "#fcd34d",
          400: "#fbbf24",
          500: "#f59e0b",
          600: "#d97706",
          700: "#b45309",
          800: "#92400e",
          900: "#78350f",
          950: "#451a03",
        },
        // Error/danger colors - muted rose
        danger: {
          DEFAULT: "#be123c", // rose-700
          50: "#fff1f2",
          100: "#ffe4e6",
          200: "#fecdd3",
          300: "#fda4af",
          400: "#fb7185",
          500: "#f43f5e",
          600: "#e11d48",
          700: "#be123c",
          800: "#9f1239",
          900: "#881337",
          950: "#4c0519",
        },
        // Background colors - elegant dark
        background: {
          DEFAULT: "#1a1a1a", // custom dark gray
          darker: "#121212", // near black
          dark: "#1c1c1c", // dark charcoal
          medium: "#2a2a2a", // medium charcoal
          light: "#333333", // lighter charcoal
          sidebar: "#0f0f0f", // very dark
          card: "#202020", // card background
          hover: "#2c2c2c", // hover state
          active: "#252525", // active state
        },
        // Text colors - elegant grayscale
        text: {
          DEFAULT: "#e5e5e5", // off-white
          primary: "#ffffff", // pure white
          secondary: "#a3a3a3", // light gray
          muted: "#737373", // medium gray
          disabled: "#525252", // dark gray
          dark: "#262626", // very dark gray
          invert: "#0a0a0a", // near black
        },
        // Border colors - subtle separators
        border: {
          DEFAULT: "#333333", // medium gray
          light: "#404040", // lighter gray
          dark: "#262626", // darker gray
          active: "#525252", // active state
        },
        // Accent colors for visual interest
        accent: {
          green: "#84cc16", // muted lime
          teal: "#14b8a6", // muted teal
          rose: "#f43f5e", // muted rose
          amber: "#f59e0b", // muted amber
          violet: "#8b5cf6", // muted violet
        },
      },
      borderRadius: {
        sm: "0.125rem",
        md: "0.375rem",
        lg: "0.5rem",
        xl: "0.75rem",
        "2xl": "1rem",
      },
      fontFamily: {
        sans: [
          "Inter",
          "ui-sans-serif",
          "system-ui",
          "-apple-system",
          "sans-serif",
        ],
        mono: ["JetBrains Mono", "Menlo", "Monaco", "Consolas", "monospace"],
      },
      boxShadow: {
        sm: "0 1px 2px 0 rgba(0, 0, 0, 0.25)",
        md: "0 4px 6px -1px rgba(0, 0, 0, 0.3), 0 2px 4px -1px rgba(0, 0, 0, 0.26)",
        lg: "0 10px 15px -3px rgba(0, 0, 0, 0.3), 0 4px 6px -2px rgba(0, 0, 0, 0.25)",
        xl: "0 20px 25px -5px rgba(0, 0, 0, 0.3), 0 10px 10px -5px rgba(0, 0, 0, 0.24)",
        subtle: "0 2px 5px rgba(0, 0, 0, 0.08)",
        inner: "inset 0 2px 4px 0 rgba(0, 0, 0, 0.15)",
      },
    },
  },
  plugins: [],
};
