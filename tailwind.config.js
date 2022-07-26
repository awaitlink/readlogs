const defaultTheme = require("tailwindcss/defaultTheme");
const colors = require("tailwindcss/colors");

const baseColor = colors.gray;

module.exports = {
  content: [
    "./index.html",
    "./readlogs/src/**/*.rs",
  ],
  plugins: [
    require("@tailwindcss/typography"),
    require("@tailwindcss/forms"),
  ],
  theme: {
    extend: {
      fontFamily: {
        sans: ["Inter var", ...defaultTheme.fontFamily.sans],
      },
      colors: {
        brand: {
          "text": baseColor[600],
          "text-primary-hover": baseColor[100],
          "text-primary-active": baseColor[100],

          "primary-hover": baseColor[500],
          "primary-active": baseColor[500],

          "border": baseColor[500],
          "border-table": baseColor[200],

          "focus": baseColor[600],

          "bg-message": baseColor[200],
          "bg": baseColor[100],
          "bg-footer": baseColor[50],
          "bg-text-field": baseColor[50],

          dark: {
            "text": baseColor[300],
            "text-primary-hover": baseColor[800],
            "text-primary-active": baseColor[800],

            "primary-hover": baseColor[400],
            "primary-active": baseColor[400],

            "border": baseColor[400],
            "border-table": baseColor[700],

            "focus": baseColor[500],

            "bg-message": baseColor[700],
            "bg": baseColor[800],
            "bg-footer": baseColor[900],
            "bg-text-field": baseColor[900],
          }
        }
      },
    },
  },
};
