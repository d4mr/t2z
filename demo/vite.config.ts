import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';

export default defineConfig({
  plugins: [wasm(), topLevelAwait()],
  server: {
    headers: {
      // Required for SharedArrayBuffer (multithreaded WASM)
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
    },
  },
  preview: {
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
    },
  },
  optimizeDeps: {
    exclude: ['@d4mr/t2z-wasm'],
  },
  build: {
    target: 'esnext',
  },
  worker: {
    format: 'es',
    plugins: () => [wasm(), topLevelAwait()],
  },
});

