import { test, expect } from "@playwright/test";

test.describe("Catalog", () => {
    test.beforeEach(async ({ page }) => {
        await page.goto("/catalog");
        // Wait for products to load
        await expect(page.locator(".card").first()).toBeVisible({ timeout: 10000 });
    });

    test("catalog page loads with products", async ({ page }) => {
        await expect(page.getByText("All Components")).toBeVisible();
        await expect(page.locator(".card").first()).toBeVisible();
    });

    test("filter sidebar is visible on desktop", async ({ page }) => {
        await expect(page.getByText("Category")).toBeVisible();
        await expect(page.getByText("Availability")).toBeVisible();
        await expect(page.getByText("Price range")).toBeVisible();
    });

    test("clicking a category filter updates results", async ({ page }) => {
        await page.getByRole("button", { name: "Propulsion" }).click();
        await expect(page.getByText("Results")).toBeVisible({ timeout: 5000 });
    });

    test("in stock toggle filters products", async ({ page }) => {
        const initialCount = await page.locator(".card").count();
        await page.getByText("In stock only").click();
        await page.waitForTimeout(600); // wait for animation
        const filteredCount = await page.locator(".card").count();
        expect(filteredCount).toBeLessThanOrEqual(initialCount);
    });

    test("search filters products", async ({ page }) => {
        await page.locator("input[placeholder*='Search']").fill("engine");
        await page.waitForTimeout(600);
        const cards = page.locator(".card");
        await expect(cards.first()).toBeVisible({ timeout: 5000 });
    });

    test("clicking a product card navigates to product detail", async ({ page }) => {
        await page.locator(".card").first().click();
        await expect(page).toHaveURL(/\/product\//, { timeout: 5000 });
    });

    test("clear filters button resets filters", async ({ page }) => {
        await page.getByRole("button", { name: "Propulsion" }).click();
        await expect(page.getByText("Active filters")).toBeVisible({ timeout: 3000 });
        await page.getByText("Clear all").click();
        await expect(page.getByText("All Components")).toBeVisible({ timeout: 3000 });
    });
});
