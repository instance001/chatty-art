import js from "@eslint/js";
import globals from "globals";

export default [
  {
    ignores: [
      "node_modules/**",
      "target/**",
      "diffuse_runtime/**",
      "audio_runtime/**",
      "runtime/**",
    ],
  },
  {
    files: ["static/**/*.js"],
    languageOptions: {
      ecmaVersion: 2022,
      sourceType: "script",
      globals: {
        ...globals.browser,
        ...globals.es2022,
      },
    },
    rules: {
      ...js.configs.recommended.rules,
      "no-unused-vars": ["warn", { "argsIgnorePattern": "^_" }],
    },
  },
];
