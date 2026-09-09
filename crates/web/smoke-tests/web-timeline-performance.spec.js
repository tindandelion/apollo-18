const os = require("node:os");
const { test, expect } = require("@playwright/test");

const warmupMilliseconds = 2_000;
const measurementMilliseconds = 8_000;
const maximumBackingDimension = 1152;
const minimumMeasuredFrames = 60;

test.use({
  viewport: { width: 1440, height: 900 },
  deviceScaleFactor: 2,
});

test("records completed high-density rendering stage timings", async ({
  page,
  browser,
}) => {
  await page.addInitScript(() => {
    const requestAnimationFrame = window.requestAnimationFrame.bind(window);
    const ImageDataConstructor = window.ImageData;
    const putImageData = CanvasRenderingContext2D.prototype.putImageData;
    const measurements = {
      active: false,
      currentFrame: undefined,
      frames: [],
    };
    window.apollo18PerformanceMeasurements = measurements;

    window.ImageData = new Proxy(ImageDataConstructor, {
      construct(target, argumentsList, newTarget) {
        const started = performance.now();
        const imageData = Reflect.construct(target, argumentsList, newTarget);
        const elapsed = performance.now() - started;
        const frame = measurements.currentFrame;
        if (frame) {
          frame.imageDataConstructions += 1;
          frame.imageDataMilliseconds += elapsed;
          frame.imageDataDimensions.push(`${imageData.width}x${imageData.height}`);
        }
        return imageData;
      },
    });

    CanvasRenderingContext2D.prototype.putImageData = function (
      imageData,
      ...arguments_
    ) {
      const started = performance.now();
      try {
        return putImageData.call(this, imageData, ...arguments_);
      } finally {
        const elapsed = performance.now() - started;
        const frame = measurements.currentFrame;
        if (frame) {
          frame.presentations += 1;
          frame.presentationMilliseconds += elapsed;
          frame.presentationDimensions.push(
            `${imageData.width}x${imageData.height}`,
          );
        }
      }
    };

    window.requestAnimationFrame = (callback) =>
      requestAnimationFrame((timestamp) => {
        const frame = {
          startedMilliseconds: performance.now(),
          imageDataConstructions: 0,
          imageDataMilliseconds: 0,
          imageDataDimensions: [],
          presentations: 0,
          presentationMilliseconds: 0,
          presentationDimensions: [],
        };
        measurements.currentFrame = frame;
        try {
          callback(timestamp);
        } finally {
          frame.completedMilliseconds = performance.now();
          frame.completeMilliseconds =
            frame.completedMilliseconds - frame.startedMilliseconds;
          frame.softwareRenderingMilliseconds =
            frame.completeMilliseconds -
            frame.imageDataMilliseconds -
            frame.presentationMilliseconds;
          measurements.currentFrame = undefined;
          if (measurements.active) {
            measurements.frames.push(frame);
          }
        }
      });
  });

  await page.goto("/");
  const canvas = page.locator("#apollo18-canvas");
  await expect(canvas).toHaveJSProperty("width", maximumBackingDimension);
  await expect(canvas).toHaveJSProperty("height", maximumBackingDimension);

  await page.waitForTimeout(warmupMilliseconds);
  await page.evaluate(() => {
    window.apollo18PerformanceMeasurements.frames = [];
    window.apollo18PerformanceMeasurements.active = true;
  });
  await page.waitForTimeout(measurementMilliseconds);
  const configuration = {
    expectedDimensions: `${maximumBackingDimension}x${maximumBackingDimension}`,
    warmupMilliseconds,
    measurementMilliseconds,
  };
  const result = await page.evaluate((configuration) => {
    const measurements = window.apollo18PerformanceMeasurements;
    measurements.active = false;
    const observedFrames = measurements.frames.slice();
    const isCompletedFrame = (frame) =>
      frame.imageDataConstructions === 1 &&
      frame.presentations === 1 &&
      frame.imageDataDimensions[0] === configuration.expectedDimensions &&
      frame.presentationDimensions[0] === configuration.expectedDimensions &&
      frame.softwareRenderingMilliseconds > 0;
    const frames = observedFrames.filter(isCompletedFrame);
    const median = (values) => {
      const sorted = values.slice().sort((left, right) => left - right);
      const middle = Math.floor(sorted.length / 2);
      return sorted.length % 2 === 0
        ? (sorted[middle - 1] + sorted[middle]) / 2
        : sorted[middle];
    };
    const rounded = (value) => Number(value.toFixed(2));
    const firstCompletion = frames.at(0)?.completedMilliseconds;
    const lastCompletion = frames.at(-1)?.completedMilliseconds;
    const elapsedMilliseconds = lastCompletion - firstCompletion;
    const canvas = document.querySelector("#apollo18-canvas");
    const bounds = canvas.getBoundingClientRect();

    return {
      environment: {
        userAgent: navigator.userAgent,
        viewport: `${window.innerWidth}x${window.innerHeight} CSS pixels`,
        devicePixelRatio: window.devicePixelRatio,
        canvasCssDimensions: `${bounds.width}x${bounds.height} CSS pixels`,
        backingResolution: `${canvas.width}x${canvas.height}`,
      },
      measurement: {
        warmupMilliseconds: configuration.warmupMilliseconds,
        requestedMeasurementMilliseconds:
          configuration.measurementMilliseconds,
        elapsedMilliseconds: rounded(elapsedMilliseconds),
        completedFrames: frames.length,
        completedFramesPerSecond: rounded(
          ((frames.length - 1) * 1_000) / elapsedMilliseconds,
        ),
      },
      medianStageMilliseconds: {
        softwareRendering: rounded(
          median(frames.map((frame) => frame.softwareRenderingMilliseconds)),
        ),
        imageDataConstruction: rounded(
          median(frames.map((frame) => frame.imageDataMilliseconds)),
        ),
        canvasPresentation: rounded(
          median(frames.map((frame) => frame.presentationMilliseconds)),
        ),
        completeFrame: rounded(
          median(frames.map((frame) => frame.completeMilliseconds)),
        ),
      },
      incompleteFrames: observedFrames.length - frames.length,
    };
  }, configuration);

  const diagnostic = {
    host: {
      hostname: os.hostname(),
      operatingSystem: `${os.type()} ${os.release()}`,
      architecture: os.arch(),
      processor: os.cpus()[0]?.model ?? "unknown",
    },
    browser: {
      name: "Chromium",
      version: browser.version(),
      userAgent: result.environment.userAgent,
    },
    viewport: result.environment.viewport,
    devicePixelRatio: result.environment.devicePixelRatio,
    canvasCssDimensions: result.environment.canvasCssDimensions,
    backingResolution: result.environment.backingResolution,
    measurement: result.measurement,
    medianStageMilliseconds: result.medianStageMilliseconds,
  };

  console.log(JSON.stringify(diagnostic, null, 2));
  expect(result.environment.viewport).toBe("1440x900 CSS pixels");
  expect(result.environment.devicePixelRatio).toBe(2);
  expect(result.environment.backingResolution).toBe("1152x1152");
  expect(result.incompleteFrames).toBe(0);
  expect(result.measurement.completedFrames).toBeGreaterThanOrEqual(
    minimumMeasuredFrames,
  );
  expect(result.medianStageMilliseconds.softwareRendering).toBeGreaterThan(0);
  expect(
    result.medianStageMilliseconds.imageDataConstruction,
  ).toBeGreaterThanOrEqual(0);
  expect(
    result.medianStageMilliseconds.canvasPresentation,
  ).toBeGreaterThanOrEqual(0);
  expect(result.medianStageMilliseconds.completeFrame).toBeGreaterThan(0);
});
