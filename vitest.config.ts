import { svelte } from '@sveltejs/vite-plugin-svelte'
import { defineConfig } from 'vitest/config'

export default defineConfig({
  plugins: [svelte({ configFile: 'ui/svelte.config.js' })],
  test: {
    include: ['ui/src/**/*.test.ts'],
    coverage: {
      provider: 'v8',
      include: ['ui/src/**/*.{ts,svelte}'],
      exclude: ['ui/src/**/*.test.ts'],
      reporter: ['text', 'html', 'json'],
      reportsDirectory: 'coverage/ui',
    },
  },
})
