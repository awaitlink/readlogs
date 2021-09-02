const defaultTheme = require("tailwindcss/defaultTheme");
const colors = require("tailwindcss/colors");

const baseColor = colors.coolGray;

module.exports = {
  mode: "jit",
  purge: [
    "./index.html",
    "./src/**/*.rs",
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
        rose: colors.rose,
        fuchsia: colors.fuchsia,

        brand: {
          "text": baseColor[600],
          "text-primary-hover": baseColor[100],
          "text-primary-active": baseColor[100],

          "primary-hover": baseColor[500],
          "primary-active": baseColor[500],

          "border": baseColor[500],
          "border-table": baseColor[200],

          "focus": baseColor[600],

          "bg": baseColor[100],
          "bg-footer": baseColor[50],
        }
      },
    },
  },
};
