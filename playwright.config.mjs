import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./tests/e2e",
  timeout: 20_000,
  expect: {
    timeout: 6_000,
  },
  fullyParallel: false,
  use: {
    baseURL: "http://127.0.0.1:4173",
    browserName: "chromium",
    headless: true,
    viewport: {
      width: 1280,
      height: 720,
    },
    trace: "retain-on-failure",
  },
  webServer: {
    command: "python3 -m http.server 4173 --directory web",
    url: "http://127.0.0.1:4173",
    reuseExistingServer: !process.env.CI,
    timeout: 30_000,
  },
});
