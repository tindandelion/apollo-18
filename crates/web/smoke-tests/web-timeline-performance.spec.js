const { test, expect } = require("@playwright/test");

const maximumBackingDimension = 1152;
const measuredFrameCount = 30;

test.use({
  viewport: { width: 1440, height: 900 },
  deviceScaleFactor: 2,
});

test("records the high-density ephemeris-span animation baseline", async ({ page }) => {
  await page.addInitScript(() => {
    const requestAnimationFrame = window.requestAnimationFrame.bind(window);
    const putImageData = CanvasRenderingContext2D.prototype.putImageData;
    window.apollo18TimelineMeasurements = {
      firstAnimationRequestMilliseconds: undefined,
      callbacks: [],
      presentationMilliseconds: [],
    };

    window.requestAnimationFrame = (callback) => {
      window.apollo18TimelineMeasurements.firstAnimationRequestMilliseconds ??=
        performance.now();
      return requestAnimationFrame((timestamp) => {
        const started = performance.now();
        callback(timestamp);
        window.apollo18TimelineMeasurements.callbacks.push(
          performance.now() - started,
        );
      });
    };
    CanvasRenderingContext2D.prototype.putImageData = function (
      imageData,
      ...arguments_
    ) {
      const started = performance.now();
      const result = putImageData.call(this, imageData, ...arguments_);
      window.apollo18TimelineMeasurements.presentationMilliseconds.push(
        performance.now() - started,
      );
      return result;
    };
  });

  await page.goto("/");
  const canvas = page.locator("#apollo18-canvas");
  await expect(canvas).toHaveJSProperty("width", maximumBackingDimension);
  await expect(canvas).toHaveJSProperty("height", maximumBackingDimension);
  await expect
    .poll(() =>
      page.evaluate(
        () => window.apollo18TimelineMeasurements.callbacks.length,
      ),
    )
    .toBeGreaterThanOrEqual(measuredFrameCount);

  const baseline = await page.evaluate((frameCount) => {
    const measurements = window.apollo18TimelineMeasurements;
    const callbacks = measurements.callbacks.slice(0, frameCount);
    const presentations = measurements.presentationMilliseconds.slice(
      0,
      frameCount,
    );
    const mean = (values) =>
      values.reduce((total, value) => total + value, 0) / values.length;
    const meanCompleteMilliseconds = mean(callbacks);
    const meanPresentationMilliseconds = mean(presentations);
    const canvas = document.querySelector("#apollo18-canvas");

    return {
      backingResolution: `${canvas.width}x${canvas.height}`,
      startupPreparationMilliseconds: Number(
        measurements.firstAnimationRequestMilliseconds.toFixed(2),
      ),
      meanRecurringRenderMilliseconds: Number(
        (meanCompleteMilliseconds - meanPresentationMilliseconds).toFixed(2),
      ),
      meanPresentationMilliseconds: Number(
        meanPresentationMilliseconds.toFixed(2),
      ),
      meanCompleteFrameMilliseconds: Number(
        meanCompleteMilliseconds.toFixed(2),
      ),
      completedFramesPerSecond: Number(
        (1_000 / meanCompleteMilliseconds).toFixed(2),
      ),
      measuredFrames: callbacks.length,
      userAgent: navigator.userAgent,
    };
  }, measuredFrameCount);

  console.log(JSON.stringify(baseline, null, 2));
  expect(baseline.backingResolution).toBe("1152x1152");
  expect(baseline.measuredFrames).toBe(measuredFrameCount);
  expect(baseline.startupPreparationMilliseconds).toBeGreaterThan(0);
  expect(baseline.meanRecurringRenderMilliseconds).toBeGreaterThan(0);
  expect(baseline.meanCompleteFrameMilliseconds).toBeGreaterThan(0);
});
