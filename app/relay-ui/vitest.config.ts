import { fileURLToPath, URL } from 'node:url'

import { defineConfig } from 'vitest/config'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
      '@fixtures': fileURLToPath(new URL('../fixtures', import.meta.url)),
    },
  },
  test: {
    environment: 'jsdom',
    include: ['test/**/*.vitest.ts'],
    setupFiles: ['test/vitest-setup.ts'],
  },
})
