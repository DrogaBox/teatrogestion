import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import tailwindcss from '@tailwindcss/vite';
import { fileURLToPath } from 'node:url';
export default defineConfig({
  plugins: [tailwindcss(), svelte()],
  resolve: { alias: { '@tauri-apps/api/core': fileURLToPath(new URL('./src/mock/tauri-core.ts', import.meta.url)) } },
  server: { port: 1431, strictPort: true },
});
