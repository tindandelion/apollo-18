const { test, expect } = require("@playwright/test");

const warmupMilliseconds = 2_000;
const measurementMilliseconds = 8_000;
const minimumFramesPerSecond = 30;

test("release web host sustains the animation frame-rate baseline", async ({
  page,
}) => {
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
  await expect
    .poll(() =>
      page.evaluate(() => window.apollo18CompletedAnimationFrames.length),
    )
    .toBeGreaterThan(1);

  await page.waitForTimeout(warmupMilliseconds);
  await page.evaluate(() => {
    window.apollo18CompletedAnimationFrames = [];
  });
  await page.waitForTimeout(measurementMilliseconds);

  const frameTimestamps = await page.evaluate(() =>
    window.apollo18CompletedAnimationFrames.slice(),
  );
  expect(frameTimestamps.length).toBeGreaterThan(1);

  const elapsedMilliseconds =
    frameTimestamps.at(-1) - frameTimestamps.at(0);
  const measuredFramesPerSecond =
    ((frameTimestamps.length - 1) * 1_000) / elapsedMilliseconds;

  console.log(
    `measured ${measuredFramesPerSecond.toFixed(2)} FPS over ${(
      elapsedMilliseconds / 1_000
    ).toFixed(2)} seconds`,
  );
  expect(measuredFramesPerSecond).toBeGreaterThanOrEqual(
    minimumFramesPerSecond,
  );
});
