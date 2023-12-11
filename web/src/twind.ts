import { defineConfig } from "@twind/core";
import presetAutoprefix from "@twind/preset-autoprefix";
import presetTailwind from "@twind/preset-tailwind/base";
import install from "@twind/with-web-components";

/**
 * The configuration for `twind` Tailwind-in-JS
 *
 * This configuration only applies to the use of Tailwind
 * within TypeScript/Javascript (i.e. where `installTwind` is
 * called).
 *
 * For configuration of Tailwind for themes see the `tailwind.config.js` file.
 */
const config = defineConfig({
  presets: [presetAutoprefix(), presetTailwind()],

  theme: {
    extend: {
      fontFamily: {
        sans: ["Lato", "Montserrat"],
      },
      colors: {
        brand: {
          blue: "#2568ef",
          green: "#6f6",
          red: "#e53e3e",
          yellow: "#ecc94b",
        },
        gray: {
          "wild-sand": "#526d8b",
          "shady": "#9d9d9d",
          "aluminium": "#999999",
          "mine-shaft": "#333333"
        },
        "eerie-black": "#171817"
      },
    },
  },

  hash: false,
});

export const installTwind = () => install(config);
