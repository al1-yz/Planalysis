import js from "@eslint/js";
import prettier from "eslint-config-prettier";
import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";
import { defineConfig, globalIgnores } from "eslint/config";
import globals from "globals";
import tseslint from "typescript-eslint";

export default defineConfig([
  globalIgnores([
    "dist",
    "coverage",
    "playwright-report",
    "test-results",
    "src-tauri/target",
    "src-tauri/gen",
  ]),
  {
    files: ["**/*.{ts,tsx}"],
    extends: [
      js.configs.recommended,
      tseslint.configs.strict,
      reactHooks.configs.flat["recommended-latest"],
      reactRefresh.configs.vite,
    ],
    languageOptions: {
      ecmaVersion: 2023,
      globals: globals.browser,
    },
  },
  {
    // Node-side config and E2E files.
    files: ["*.config.{js,ts}", "e2e/**/*.ts"],
    languageOptions: { globals: globals.node },
  },
  {
    // shadcn/ui primitives export variant helpers alongside components.
    files: ["src/components/ui/**/*.tsx"],
    rules: { "react-refresh/only-export-components": "off" },
  },
  {
    // Architecture guardrail: the domain layer is pure TypeScript.
    files: ["src/domain/**/*.ts"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          patterns: [
            {
              group: ["react", "react-*", "react/*", "radix-ui", "lucide-react", "@tauri-apps/*"],
              message: "The domain layer must not depend on UI or persistence libraries.",
            },
            {
              group: [
                "@/app/*",
                "@/features/*",
                "@/components/*",
                "@/application/*",
                "@/infrastructure/*",
              ],
              message: "The domain layer must not import from outer layers.",
            },
          ],
        },
      ],
    },
  },
  {
    // Application layer depends on the domain only, never on UI or concrete persistence.
    files: ["src/application/**/*.ts"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          patterns: [
            {
              group: ["react", "react-*", "react/*", "@tauri-apps/*"],
              message: "Use cases must not depend on React or the Tauri runtime.",
            },
            {
              group: ["@/app/*", "@/features/*", "@/components/*", "@/infrastructure/*"],
              message: "Use cases must depend on domain types and repository interfaces only.",
            },
          ],
        },
      ],
    },
  },
  {
    // Infrastructure implements domain interfaces; it never reaches into UI or use cases.
    files: ["src/infrastructure/**/*.ts"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          patterns: [
            {
              group: ["@/app/*", "@/features/*", "@/components/*", "@/application/*"],
              message: "Infrastructure may depend on the domain layer only.",
            },
          ],
        },
      ],
    },
  },
  prettier,
]);
