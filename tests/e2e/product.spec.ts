import { test, expect } from "@playwright/test";

test.describe("Product detail", () => {
    test.beforeEach(async ({ page }) => {
        // Navigate via catalog to get a real product
        await page.goto("/catalog");
        await expect(page.locator(".card").first()).toBeVisible({ timeout: 10000 });
        await page.locator(".card").first().click();
        await expect(page).toHaveURL(/\/product\//, { timeout: 5000 });
    });

    test("product detail page loads correctly", async ({ page }) => {
        await expect(page.getByText("Specifications")).toBeVisible();
        await expect(page.getByText("Add to cart")).toBeVisible();
        await expect(page.getByText("Compare")).toBeVisible();
    });

    test("breadcrumb navigation is visible", async ({ page }) => {
        await expect(page.getByText("Catalog")).toBeVisible();
        await expect(page.getByText("Home")).toBeVisible();
    });

    test("add to cart button works", async ({ page }) => {
        await page.getByRole("button", { name: "Add to cart" }).click();
        await expect(page.getByText(/Added/)).toBeVisible({ timeout: 3000 });
    });

    test("cart count updates in navbar after adding item", async ({ page }) => {
        await page.getByRole("button", { name: "Add to cart" }).click();
        // Cart count badge should appear
        await expect(page.locator("nav").getByText("1")).toBeVisible({ timeout: 3000 });
    });

    test("compare button navigates to compare page", async ({ page }) => {
        await page.getByRole("button", { name: "Compare" }).click();
        await expect(page).toHaveURL(/\/compare/, { timeout: 5000 });
    });

    test("catalog breadcrumb navigates back", async ({ page }) => {
        await page.getByText("Catalog").click();
        await expect(page).toHaveURL(/catalog/, { timeout: 3000 });
    });
});
