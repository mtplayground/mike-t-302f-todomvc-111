const baseURL = process.env.E2E_BASE_URL || 'http://127.0.0.1:8080';

/** @type {import('playwright/test').PlaywrightTestConfig} */
module.exports = {
  testDir: './e2e',
  fullyParallel: false,
  timeout: 30_000,
  expect: {
    timeout: 5_000,
  },
  use: {
    baseURL,
    trace: 'retain-on-failure',
  },
  webServer: process.env.E2E_START_SERVER
    ? {
        command: process.env.E2E_START_SERVER,
        url: baseURL,
        reuseExistingServer: !process.env.CI,
        timeout: 120_000,
      }
    : undefined,
};
