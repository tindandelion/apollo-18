const path = require("node:path");
const { defineConfig } = require("@playwright/test");

const webServerUrl = "http://127.0.0.1:41718";

module.exports = defineConfig({
  testDir: __dirname,
  testMatch: "*.spec.js",
  timeout: 60_000,
  expect: {
    timeout: 30_000,
  },
  fullyParallel: false,
  workers: 1,
  reporter: "line",
  outputDir: path.resolve(__dirname, "../../../target/playwright"),
  use: {
    baseURL: webServerUrl,
    browserName: "chromium",
    headless: true,
  },
  webServer: {
    command:
      "trunk serve index.html --release --address 127.0.0.1 --port 41718",
    cwd: path.resolve(__dirname, ".."),
    env: {
      ...process.env,
      NO_COLOR: "true",
    },
    url: webServerUrl,
    reuseExistingServer: false,
    timeout: 120_000,
    stdout: "pipe",
    stderr: "pipe",
  },
});
