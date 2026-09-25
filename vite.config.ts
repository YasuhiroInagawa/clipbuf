import { svelte } from '@sveltejs/vite-plugin-svelte';
import { svelteTesting } from '@testing-library/svelte/vite';
import { fileURLToPath, URL } from 'node:url';
import { defineConfig } from 'vitest/config';

const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig({
  plugins: [svelte(), svelteTesting()],
  resolve: {
    alias: {
      $lib: fileURLToPath(new URL('./src/lib', import.meta.url)),
    },
  },
  // Vite options tailored for Tauri development
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
  test: {
    include: ['src/**/*.test.ts', 'scripts/**/*.test.mjs'],
    // Pure modules run in node; component tests opt into jsdom with `@vitest-environment jsdom`.
    environment: 'node',
    // Component styles are applied so visibility assertions (always-visible buttons) are real.
    css: true,
    setupFiles: ['./src/test-setup.ts'],
  },
});
