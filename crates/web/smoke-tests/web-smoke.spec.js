const { test, expect } = require("@playwright/test");

const backgroundChannel = 0x18;

function isFaviconRequest(url) {
  return new URL(url).pathname === "/favicon.ico";
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

test("release web host presents the software-rendered framebuffer", async ({
  page,
}) => {
  const runtimeErrors = [];

  page.on("pageerror", (error) => runtimeErrors.push(error.message));
  page.on("console", (message) => {
    if (message.type() === "error" && !message.text().includes("favicon.ico")) {
      runtimeErrors.push(message.text());
    }
  });
  page.on("requestfailed", (request) => {
    if (!isFaviconRequest(request.url())) {
      runtimeErrors.push(
        `${request.method()} ${request.url()} failed: ${request.failure()?.errorText}`,
      );
    }
  });
  page.on("response", (response) => {
    if (response.status() >= 400 && !isFaviconRequest(response.url())) {
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

  await expect(canvas).toHaveJSProperty("width", 800);
  await expect(canvas).toHaveJSProperty("height", 800);

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
