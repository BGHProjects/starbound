import { test, expect } from "@playwright/test";

test("homepage loads", async ({ page }) => {
  await page.goto("/");
  await expect(page).toHaveTitle(/Starbound/);
});

test("catalog page is reachable", async ({ page }) => {
  await page.goto("/catalog");
  await expect(page.locator("main")).toBeVisible();
});
