const { test, expect } = require("@playwright/test");

const backgroundChannel = 0x18;

function isFaviconRequest(url) {
  return new URL(url).pathname === "/favicon.ico";
}

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
