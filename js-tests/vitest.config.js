import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    environment: 'node',
    setupFiles: ['./setup.js'],
    testTimeout: 10000,
    globals: true,
    include: ['**/*.test.js'],
    coverage: {
      reporter: ['text', 'html'],
      exclude: ['pkg/**', 'node_modules/**']
    }
  }
});
