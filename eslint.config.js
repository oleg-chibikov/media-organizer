import js from "@eslint/js";
import eslintConfigPrettier from "eslint-config-prettier";
import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";
import globals from "globals";
import tseslint from "typescript-eslint";

export default tseslint.config(
  {
    ignores: ["dist", "node_modules", "src-tauri/binaries", "src-tauri/gen", "src-tauri/target"]
  },
  {
    files: ["**/*.{ts,tsx}"],
    extends: [js.configs.recommended, ...tseslint.configs.recommended],
    languageOptions: {
      ecmaVersion: 2020,
      globals: globals.browser
    },
    plugins: {
      "react-hooks": reactHooks,
      "react-refresh": reactRefresh
    },
    rules: {
      ...reactHooks.configs.recommended.rules,
      "react-hooks/incompatible-library": "off",
      "react-refresh/only-export-components": ["warn", { allowConstantExport: true }]
    }
  },
  {
    files: ["src/features/**/*.{ts,tsx}"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          patterns: [
            {
              regex: "^\\.\\./(?!\\.\\.)[^/]+/.+",
              message: "Import another feature through its index, for example `../moves`."
            },
            {
              regex: "(^|/)app(/|$)",
              message: "Features must not depend on app-level orchestration."
            }
          ]
        }
      ]
    }
  },
  {
    files: ["src/app/**/*.{ts,tsx}", "src/App.tsx", "src/App.test.tsx"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          patterns: [
            {
              regex: "(^|/)features/[^/]+/.+",
              message: "Import a feature through its index, for example `./features/moves`."
            }
          ]
        }
      ]
    }
  },
  {
    files: ["src/lib/**/*.{ts,tsx}", "src/components/**/*.{ts,tsx}"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          patterns: [
            {
              regex: "(^|/)(features|app)(/|$)",
              message: "Shared lib and design-system code must not depend on features or app."
            }
          ]
        }
      ]
    }
  },
  eslintConfigPrettier
);
