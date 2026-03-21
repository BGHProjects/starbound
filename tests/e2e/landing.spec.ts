import { test, expect } from "@playwright/test";

test.describe("Landing page", () => {
    test.beforeEach(async ({ page }) => {
        await page.goto("/");
        await page.waitForLoadState("domcontentloaded");
    });

    test("hero section is visible", async ({ page }) => {
        await expect(page.getByText("Build Your")).toBeVisible({ timeout: 5000 });
        await expect(page.getByText("Launch Your")).toBeVisible({ timeout: 5000 });
    });

    test("Browse Catalog button navigates to catalog", async ({ page }) => {
        await page.getByRole("button", { name: "Browse Catalog" }).click();
        await expect(page).toHaveURL(/catalog/);
    });

    test("Create Account button navigates to register", async ({ page }) => {
        await page.getByRole("button", { name: "Create Account" }).click();
        await expect(page).toHaveURL(/register/);
    });

    test("featured products section loads", async ({ page }) => {
        await expect(page.getByText("Featured Components")).toBeVisible();
        await expect(page.locator(".card").first()).toBeVisible({ timeout: 10000 });
    });

    test("category rows load products", async ({ page }) => {
        await expect(page.getByRole("heading", { name: "Propulsion" })).toBeVisible({ timeout: 10000 });
        await expect(page.getByRole("heading", { name: "Structural" })).toBeVisible();
        await expect(page.getByRole("heading", { name: "Guidance" })).toBeVisible();
        await expect(page.getByRole("heading", { name: "Payload" })).toBeVisible();
    });

    test("navbar shows sign in link when logged out", async ({ page }) => {
        await expect(page.locator("nav").getByText("Sign in")).toBeVisible();
        await expect(page.locator("nav").getByText("Sign up")).toBeVisible();
    });
});
