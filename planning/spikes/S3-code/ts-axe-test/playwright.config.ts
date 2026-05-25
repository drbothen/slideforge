import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  // Run against the slideforge web preview dev server.
  // For CI: `cargo run --bin slideforge-preview -- --port 4173` before tests.
  use: {
    baseURL: process.env.SLIDEFORGE_PREVIEW_URL ?? 'http://localhost:4173',
  },
  // Accessibility tests can be slow due to axe scanning DOM.
  timeout: 30_000,
  reporter: [['list'], ['json', { outputFile: 'test-results/a11y-report.json' }]],
});
