import js from '@eslint/js';
import prettier from 'eslint-config-prettier';
import svelte from 'eslint-plugin-svelte';
import globals from 'globals';
import ts from 'typescript-eslint';
import svelteConfig from './svelte.config.js';

export default ts.config(
  js.configs.recommended,
  ...ts.configs.recommended,
  ...svelte.configs.recommended,
  prettier,
  ...svelte.configs.prettier,
  {
    languageOptions: {
      globals: { ...globals.browser, ...globals.node },
    },
    rules: {
      '@typescript-eslint/no-explicit-any': 'error',
    },
  },
  {
    // Design boundary: only src/lib/ipc talks to Tauri directly.
    files: ['src/**/*.{ts,svelte}'],
    ignores: ['src/lib/ipc/**'],
    rules: {
      'no-restricted-imports': [
        'error',
        {
          patterns: [
            {
              group: ['@tauri-apps/api', '@tauri-apps/api/*', '@tauri-apps/plugin-*'],
              message:
                'Import Tauri only from src/lib/ipc (design: components never call Tauri directly).',
            },
          ],
        },
      ],
    },
  },
  {
    files: ['**/*.svelte', '**/*.svelte.ts'],
    languageOptions: {
      parserOptions: {
        parser: ts.parser,
        extraFileExtensions: ['.svelte'],
        svelteConfig,
      },
    },
  },
  {
    ignores: ['dist/', 'node_modules/', 'src-tauri/', '.kiro/', '.claude/', 'doc/'],
  },
);
