import js from "@eslint/js";
import nextPlugin from "@next/eslint-plugin-next";
import tsPlugin from "@typescript-eslint/eslint-plugin";
import tsParser from "@typescript-eslint/parser";
import prettier from "eslint-config-prettier";

export default [
  // Baseline configuration (js recommended)
  js.configs.recommended,

  // Global settings
  {
    linterOptions: {
      reportUnusedDisableDirectives: true,
    },
    ignores: ["**/node_modules/**", ".next/**", "build/**"],
    // Define shared settings
    settings: {
      next: {
        rootDir: "./",
      },
    },
    // Define which files this config applies to
    files: ["**/*.{js,jsx,ts,tsx}"],
    // Configure language options
    languageOptions: {
      ecmaVersion: 2022,
      sourceType: "module",
      parser: tsParser,
      parserOptions: {
        project: "./tsconfig.json",
        ecmaFeatures: {
          jsx: true,
        },
      },
    },
    // Plugin configurations
    plugins: {
      "@typescript-eslint": tsPlugin,
      "@next/next": nextPlugin,
    },
    // Rules configuration
    rules: {
      // TypeScript specific rules
      "@typescript-eslint/no-unused-vars": [
        "warn",
        {
          argsIgnorePattern: "^_",
          varsIgnorePattern: "^_",
          caughtErrorsIgnorePattern: "^_",
        },
      ],
      "@typescript-eslint/consistent-type-imports": [
        "warn",
        {
          prefer: "type-imports",
          fixStyle: "inline-type-imports",
        },
      ],
      // Next.js specific rules
      "@next/next/no-html-link-for-pages": "error",
      "@next/next/no-img-element": "error",
    },
  },

  // TypeScript specific configuration
  {
    files: ["*.ts", "*.tsx"],
    rules: {
      "@typescript-eslint/explicit-function-return-type": "warn",
    },
  },

  // Test files specific configuration
  {
    files: ["**/__tests__/**/*.[jt]s?(x)", "**/?(*.)+(spec|test).[jt]s?(x)"],
    rules: {
      "@typescript-eslint/no-explicit-any": "off",
    },
  },

  // Apply prettier last to override other formatting rules
  prettier,
];
