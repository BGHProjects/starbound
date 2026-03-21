import { test, expect } from "@playwright/test";

test.describe("Product comparison", () => {
    test.beforeEach(async ({ page }) => {
        await page.goto("/catalog");
        await expect(page.locator(".card").first()).toBeVisible({ timeout: 10000 });
        await page.locator(".card").first().click();
        await expect(page).toHaveURL(/\/product\//, { timeout: 5000 });
        await page.getByRole("button", { name: "Compare" }).click();
        await expect(page).toHaveURL(/\/compare/, { timeout: 5000 });
    });

    test("compare page loads with product cards", async ({ page }) => {
        await expect(page.getByText("Compare components")).toBeVisible();
    });

    test("back to product button navigates back", async ({ page }) => {
        await page.getByRole("button", { name: /back to product/i }).click();
        await expect(page).toHaveURL(/\/product\//, { timeout: 5000 });
    });

    test("breadcrumb shows correct navigation", async ({ page }) => {
        await expect(page.locator("nav, .flex").getByText("Catalog").first()).toBeVisible();
        await expect(page.locator("span.text-white").filter({ hasText: "Compare" })).toBeVisible();
    });
});
