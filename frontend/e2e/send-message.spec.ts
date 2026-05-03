import { test, expect } from "@playwright/test";

const CONV_ID = "33333333-3333-3333-3333-333333333333";
const ASSISTANT_ID = "44444444-4444-4444-4444-444444444444";

test("send message renders streamed assistant bubble", async ({ page }) => {
  const consoleErrors: string[] = [];
  page.on("console", (msg) => {
    if (msg.type() === "error") consoleErrors.push(msg.text());
  });
  page.on("pageerror", (err) => {
    consoleErrors.push(err.message);
  });

  let createdOnce = false;

  await page.route("**/api/conversations", async (route) => {
    if (route.request().method() === "GET") {
      const body = createdOnce
        ? [{ id: CONV_ID, title: "hello", created_at: "2026-05-02T00:00:00Z", updated_at: "2026-05-02T00:00:00Z" }]
        : [];
      await route.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify(body) });
      return;
    }
    if (route.request().method() === "POST") {
      createdOnce = true;
      await route.fulfill({
        status: 201,
        contentType: "application/json",
        body: JSON.stringify({
          id: CONV_ID, title: "hello",
          created_at: "2026-05-02T00:00:00Z", updated_at: "2026-05-02T00:00:00Z",
        }),
      });
      return;
    }
    await route.continue();
  });

  await page.route(`**/api/conversations/${CONV_ID}/messages`, async (route) => {
    await route.fulfill({ status: 200, contentType: "application/json", body: "[]" });
  });

  await page.route(`**/api/conversations/${CONV_ID}/stream`, async (route) => {
    const sse =
      "event: token\ndata: Hello \n\n" +
      "event: token\ndata: world\n\n" +
      `event: done\ndata: {"message_id":"${ASSISTANT_ID}"}\n\n`;
    await route.fulfill({
      status: 200,
      headers: { "content-type": "text/event-stream", "cache-control": "no-cache" },
      body: sse,
    });
  });

  await page.goto("/");
  await page.getByLabel("Message").fill("hello");
  await page.getByLabel("Send").click();

  await expect(page.getByTestId("bubble-assistant")).toBeVisible();
  await expect(page.getByTestId("bubble-assistant")).toContainText("Hello world");
  await expect(page).toHaveURL(new RegExp(`\\?c=${CONV_ID}`));

  expect(consoleErrors, `console errors: ${consoleErrors.join("\n")}`).toEqual([]);
});
