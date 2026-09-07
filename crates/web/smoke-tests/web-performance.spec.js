const { test, expect } = require("@playwright/test");

const warmupMilliseconds = 2_000;
const measurementMilliseconds = 8_000;
const maximumBackingDimension = 1152;
const minimumFramesPerSecond = 30;

test.use({
  viewport: { width: 1440, height: 900 },
  deviceScaleFactor: 2,
});

test("release web host sustains the high-density animation performance target", async ({ page }) => {
  await page.addInitScript(() => {
    const requestAnimationFrame = window.requestAnimationFrame.bind(window);
    window.apollo18CompletedAnimationFrames = [];

    window.requestAnimationFrame = (callback) =>
      requestAnimationFrame((timestamp) => {
        callback(timestamp);
        window.apollo18CompletedAnimationFrames.push(timestamp);
      });
  });

  await page.goto("/");
  const canvas = page.locator("#apollo18-canvas");
  await expect(canvas).toHaveJSProperty("width", maximumBackingDimension);
  await expect(canvas).toHaveJSProperty("height", maximumBackingDimension);
  await expect.poll(() => page.evaluate(() => window.apollo18CompletedAnimationFrames.length)).toBeGreaterThan(1);

  await page.waitForTimeout(warmupMilliseconds);
  await page.evaluate(() => {
    window.apollo18CompletedAnimationFrames = [];
  });
  await page.waitForTimeout(measurementMilliseconds);

  const frameTimestamps = await page.evaluate(() => window.apollo18CompletedAnimationFrames.slice());
  expect(frameTimestamps.length).toBeGreaterThan(1);

  const elapsedMilliseconds = frameTimestamps.at(-1) - frameTimestamps.at(0);
  const measuredFramesPerSecond = ((frameTimestamps.length - 1) * 1_000) / elapsedMilliseconds;

  const environment = await page.evaluate(() => {
    const canvas = document.querySelector("#apollo18-canvas");
    const bounds = canvas.getBoundingClientRect();
    return {
      userAgent: navigator.userAgent,
      viewport: `${window.innerWidth}x${window.innerHeight} CSS pixels`,
      devicePixelRatio: window.devicePixelRatio,
      canvasCssDimensions: `${bounds.width}x${bounds.height} CSS pixels`,
      backingResolution: `${canvas.width}x${canvas.height}`,
    };
  });

  console.log(
    JSON.stringify(
      {
        ...environment,
        measuredFramesPerSecond: Number(measuredFramesPerSecond.toFixed(2)),
        measurementSeconds: Number((elapsedMilliseconds / 1_000).toFixed(2)),
      },
      null,
      2,
    ),
  );
  expect(measuredFramesPerSecond).toBeGreaterThanOrEqual(minimumFramesPerSecond);
});
