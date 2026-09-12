const { test, expect } = require("@playwright/test");

const backgroundChannel = 0x18;
const maximumBackingDimension = 1152;

test.use({ deviceScaleFactor: 2 });

function selectedBackingResolution(cssWidth, cssHeight, devicePixelRatio) {
  const desiredWidth = cssWidth * devicePixelRatio;
  const desiredHeight = cssHeight * devicePixelRatio;
  const scale = Math.min(
    1,
    maximumBackingDimension / Math.max(desiredWidth, desiredHeight),
  );

  return {
    width: Math.max(1, Math.round(desiredWidth * scale)),
    height: Math.max(1, Math.round(desiredHeight * scale)),
  };
}

test("release web host fits its square presentation within the viewport", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1280, height: 720 });

  await page.goto("/");

  const layout = await page.evaluate(() => {
    const canvasBounds = document
      .querySelector("#apollo18-canvas")
      .getBoundingClientRect();

    return {
      canvasWidth: canvasBounds.width,
      canvasHeight: canvasBounds.height,
      viewportWidth: window.innerWidth,
      viewportHeight: window.innerHeight,
      pageWidth: document.documentElement.scrollWidth,
      pageHeight: document.documentElement.scrollHeight,
    };
  });
  expect(layout.pageWidth).toBeLessThanOrEqual(layout.viewportWidth);
  expect(layout.pageHeight).toBeLessThanOrEqual(layout.viewportHeight);
  expect(layout.canvasWidth).toBe(layout.canvasHeight);
});

test("footer aligns credits across wide screens and stacks them on small screens", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1280, height: 720 });
  await page.goto("/");

  const wideFooter = await page.locator("footer p").evaluateAll((paragraphs) =>
    paragraphs.map((paragraph) => {
      const bounds = paragraph.getBoundingClientRect();
      return {
        left: bounds.left,
        right: bounds.right,
        bottom: bounds.bottom,
        textAlign: getComputedStyle(paragraph).textAlign,
      };
    }),
  );

  expect(wideFooter).toHaveLength(2);
  expect(wideFooter[0].left).toBe(16);
  expect(wideFooter[1].right).toBe(1264);
  expect(wideFooter[0].bottom).toBe(wideFooter[1].bottom);
  expect(wideFooter[0].textAlign).toBe("left");
  expect(wideFooter[1].textAlign).toBe("right");

  await page.setViewportSize({ width: 900, height: 1000 });

  const narrowFooter = await page.locator("footer p").evaluateAll((paragraphs) =>
    paragraphs.map((paragraph) => {
      const bounds = paragraph.getBoundingClientRect();
      return {
        top: bounds.top,
        bottom: bounds.bottom,
        textAlign: getComputedStyle(paragraph).textAlign,
      };
    }),
  );

  expect(narrowFooter).toHaveLength(2);
  expect(narrowFooter[0].bottom).toBeLessThanOrEqual(narrowFooter[1].top);
  expect(narrowFooter[0].textAlign).toBe("center");
  expect(narrowFooter[1].textAlign).toBe("center");
});

test("release web host updates its high-density backing resolution", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.addInitScript(() => {
    const putImageData = CanvasRenderingContext2D.prototype.putImageData;
    CanvasRenderingContext2D.prototype.putImageData = function (
      imageData,
      ...arguments_
    ) {
      window.apollo18PresentedResolution = {
        width: imageData.width,
        height: imageData.height,
      };
      return putImageData.call(this, imageData, ...arguments_);
    };
  });

  await page.goto("/");
  const canvas = page.locator("#apollo18-canvas");
  await expect(canvas).toHaveJSProperty("width", maximumBackingDimension);
  await expect(canvas).toHaveJSProperty("height", maximumBackingDimension);

  const cappedLayout = await canvas.evaluate((element) => {
    const bounds = element.getBoundingClientRect();
    return {
      cssWidth: bounds.width,
      cssHeight: bounds.height,
      backingWidth: element.width,
      backingHeight: element.height,
      devicePixelRatio: window.devicePixelRatio,
      imageRendering: getComputedStyle(element).imageRendering,
      presentedResolution: window.apollo18PresentedResolution,
    };
  });
  expect(cappedLayout.cssWidth).toBe(cappedLayout.cssHeight);
  expect(cappedLayout.devicePixelRatio).toBe(2);
  expect(cappedLayout.imageRendering).not.toBe("pixelated");
  expect(cappedLayout.cssWidth * cappedLayout.devicePixelRatio).toBeGreaterThan(
    maximumBackingDimension,
  );
  expect(cappedLayout.presentedResolution).toEqual({
    width: cappedLayout.backingWidth,
    height: cappedLayout.backingHeight,
  });

  await page.setViewportSize({ width: 500, height: 700 });
  await expect
    .poll(() =>
      canvas.evaluate((element, maximumBackingDimension) => {
        const bounds = element.getBoundingClientRect();
        const desiredWidth = bounds.width * window.devicePixelRatio;
        const desiredHeight = bounds.height * window.devicePixelRatio;
        const scale = Math.min(
          1,
          maximumBackingDimension / Math.max(desiredWidth, desiredHeight),
        );
        return (
          element.width === Math.max(1, Math.round(desiredWidth * scale)) &&
          element.height === Math.max(1, Math.round(desiredHeight * scale))
        );
      }, maximumBackingDimension),
    )
    .toBe(true);

  const resizedLayout = await canvas.evaluate(
    (element, maximumBackingDimension) => {
      const bounds = element.getBoundingClientRect();
      const desiredWidth = bounds.width * window.devicePixelRatio;
      const desiredHeight = bounds.height * window.devicePixelRatio;
      const scale = Math.min(
        1,
        maximumBackingDimension / Math.max(desiredWidth, desiredHeight),
      );
      return {
        actual: { width: element.width, height: element.height },
        expected: {
          width: Math.max(1, Math.round(desiredWidth * scale)),
          height: Math.max(1, Math.round(desiredHeight * scale)),
        },
      };
    },
    maximumBackingDimension,
  );
  expect(resizedLayout.actual).toEqual(resizedLayout.expected);
  expect(resizedLayout.actual.width).toBeLessThan(maximumBackingDimension);

  await page.evaluate(() => {
    Object.defineProperty(window, "devicePixelRatio", {
      configurable: true,
      value: 1,
    });
  });
  await expect
    .poll(() =>
      canvas.evaluate((element) => {
        const bounds = element.getBoundingClientRect();
        return (
          element.width ===
            Math.max(1, Math.round(bounds.width * window.devicePixelRatio)) &&
          element.height ===
            Math.max(1, Math.round(bounds.height * window.devicePixelRatio))
        );
      }),
    )
    .toBe(true);
  const changedDensity = await canvas.evaluate((element) => {
    const bounds = element.getBoundingClientRect();
    return {
      actual: { width: element.width, height: element.height },
      expected: {
        width: Math.max(1, Math.round(bounds.width * window.devicePixelRatio)),
        height: Math.max(1, Math.round(bounds.height * window.devicePixelRatio)),
      },
    };
  });
  expect(changedDensity.actual).toEqual(changedDensity.expected);
  await expect
    .poll(() =>
      canvas.evaluate((element) => ({
        actual: { width: element.width, height: element.height },
        presented: window.apollo18PresentedResolution,
      })),
    )
    .toEqual({
      actual: changedDensity.actual,
      presented: changedDensity.actual,
    });
});

test("release web host follows controlled monotonic ephemeris-span time", async ({
  page,
}) => {
  await page.setViewportSize({ width: 360, height: 640 });
  await page.addInitScript(() => {
    const callbacks = [];
    window.apollo18PresentedFrameHashes = [];
    window.requestAnimationFrame = (callback) => {
      callbacks.push(callback);
      return callbacks.length;
    };
    window.apollo18RunAnimationFrame = (timestamp) => {
      const callback = callbacks.shift();
      if (!callback) throw new Error("no animation callback is ready");
      callback(timestamp);
    };

    const putImageData = CanvasRenderingContext2D.prototype.putImageData;
    CanvasRenderingContext2D.prototype.putImageData = function (
      imageData,
      ...arguments_
    ) {
      let hash = 2166136261;
      for (let offset = 0; offset < imageData.data.length; offset += 97) {
        hash = Math.imul(hash ^ imageData.data[offset], 16777619);
      }
      window.apollo18PresentedFrameHashes.push(hash >>> 0);
      return putImageData.call(this, imageData, ...arguments_);
    };
  });
  await page.goto("/");
  await expect.poll(() => page.evaluate(() => typeof window.apollo18RunAnimationFrame)).toBe("function");

  await page.evaluate(() => window.apollo18RunAnimationFrame(1_000));
  await page.evaluate(() => window.apollo18RunAnimationFrame(61_000));
  await page.evaluate(() => window.apollo18RunAnimationFrame(121_000));
  const hashes = await page.evaluate(() => window.apollo18PresentedFrameHashes.slice());

  expect(hashes).toHaveLength(3);
  expect(hashes[1]).not.toBe(hashes[0]);
  expect(hashes[2]).toBe(hashes[0]);
});

test("release web host replaces the canvas when ephemeris validation fails", async ({
  page,
}) => {
  const diagnostics = [];
  page.on("console", (message) => {
    if (message.type() === "error") diagnostics.push(message.text());
  });
  await page.addInitScript(() => {
    window.__apollo18EphemerisJson = "not valid JSON";
  });

  await page.goto("/");
  const canvas = page.locator("#apollo18-canvas");
  const failure = page.locator("#apollo18-render-error");

  await expect(canvas).toBeHidden();
  await expect(failure).toBeVisible();
  await expect(failure).toContainText("could not load its ephemeris data");
  expect(diagnostics.some((message) => message.includes("invalid NASA lunar ephemeris JSON"))).toBe(true);
});

test("release web host presents the software-rendered framebuffer", async ({
  page,
}) => {
  const runtimeErrors = [];

  page.on("pageerror", (error) => runtimeErrors.push(error.message));
  page.on("console", (message) => {
    if (message.type() === "error") {
      runtimeErrors.push(message.text());
    }
  });
  page.on("requestfailed", (request) => {
    runtimeErrors.push(
      `${request.method()} ${request.url()} failed: ${request.failure()?.errorText}`,
    );
  });
  page.on("response", (response) => {
    if (response.status() >= 400) {
      runtimeErrors.push(`${response.status()} ${response.url()}`);
    }
  });

  await page.addInitScript(() => {
    window.apollo18RequestedCanvasContexts = [];
    const getContext = HTMLCanvasElement.prototype.getContext;
    HTMLCanvasElement.prototype.getContext = function (...arguments_) {
      window.apollo18RequestedCanvasContexts.push(arguments_[0]);
      return getContext.apply(this, arguments_);
    };
  });

  const response = await page.goto("/");
  expect(response).not.toBeNull();
  expect(response.ok()).toBe(true);

  const canvas = page.locator("#apollo18-canvas");
  await expect(canvas).toHaveCount(1);
  await expect
    .poll(() =>
      page.evaluate(() => window.apollo18RequestedCanvasContexts.slice()),
    )
    .toContain("2d");

  const backingResolution = await canvas.evaluate((element) => {
    const bounds = element.getBoundingClientRect();
    return {
      actual: { width: element.width, height: element.height },
      cssWidth: bounds.width,
      cssHeight: bounds.height,
      devicePixelRatio: window.devicePixelRatio,
    };
  });
  expect(backingResolution.actual).toEqual(
    selectedBackingResolution(
      backingResolution.cssWidth,
      backingResolution.cssHeight,
      backingResolution.devicePixelRatio,
    ),
  );
  expect(backingResolution.actual.width).toBeGreaterThan(800);
  expect(backingResolution.actual.height).toBeGreaterThan(800);

  await expect
    .poll(() =>
      canvas.evaluate((element, expectedBackgroundChannel) => {
        const context = element.getContext("2d", { willReadFrequently: true });
        const pixels = context.getImageData(
          0,
          0,
          element.width,
          element.height,
        ).data;

        for (let offset = 0; offset < pixels.length; offset += 4) {
          if (
            pixels[offset] !== expectedBackgroundChannel ||
            pixels[offset + 1] !== expectedBackgroundChannel ||
            pixels[offset + 2] !== expectedBackgroundChannel
          ) {
            return true;
          }
        }
        return false;
      }, backgroundChannel),
    )
    .toBe(true);

  const requestedContexts = await page.evaluate(() =>
    window.apollo18RequestedCanvasContexts.slice(),
  );
  expect(requestedContexts).not.toContain("webgl");
  expect(requestedContexts).not.toContain("webgl2");
  expect(requestedContexts).not.toContain("webgpu");
  expect(runtimeErrors).toEqual([]);
});
