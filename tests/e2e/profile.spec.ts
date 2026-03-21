import { test, expect, Page } from "@playwright/test";

test.describe.serial("Profile", () => {
    let testEmail: string;
    const testPassword = "playwright123";
    const testName     = "Profile User";

    test.beforeAll(async ({ browser }) => {
        testEmail = `profile-${Date.now()}@playwright.dev`;
        const page = await browser.newPage();
        await page.goto("http://localhost:8080/register");
        await page.waitForLoadState("domcontentloaded");
        await page.locator("input[type='text']").fill(testName);
        await page.locator("input[type='email']").fill(testEmail);
        await page.locator("input[type='password']").fill(testPassword);
        await page.getByRole("button", { name: "Create account" }).click();
        await expect(page).toHaveURL("http://localhost:8080/", { timeout: 15000 });
        await page.close();
    });

    async function login(page: Page) {
        await page.goto("/login");
        await page.waitForLoadState("domcontentloaded");
        await page.locator("input[type='email']").fill(testEmail);
        await page.locator("input[type='password']").fill(testPassword);
        await page.getByRole("button", { name: "Sign in" }).click();
        await expect(page).toHaveURL("http://localhost:8080/", { timeout: 10000 });
    }

    test("profile page shows user name and email", async ({ page }) => {
        await login(page);
        await page.goto("/profile");
        await expect(page.getByText(testName)).toBeVisible();
        await expect(page.getByText(testEmail)).toBeVisible();
    });

    test("profile shows account stats", async ({ page }) => {
        await login(page);
        await page.goto("/profile");
        await expect(page.getByText("Account stats")).toBeVisible();
        await expect(page.getByText("Total orders")).toBeVisible();
        await expect(page.getByText("Total spent")).toBeVisible();
    });

    test("profile shows avatar initials", async ({ page }) => {
        await login(page);
        await page.goto("/profile");
        await expect(page.locator(".rounded-full").filter({ hasText: "P" }).first()).toBeVisible();
    });

    test("sign out redirects to landing and shows sign in", async ({ page }) => {
        await login(page);
        await page.goto("/profile");
        await page.getByRole("button", { name: /sign out/i }).click();
        await expect(page).toHaveURL("http://localhost:8080/", { timeout: 5000 });
        await expect(page.locator("nav").getByText("Sign in")).toBeVisible();
    });

    test("view all orders link navigates to orders page", async ({ page }) => {
        await login(page);
        await page.goto("/profile");
        await page.getByText("View all →").click();
        await expect(page).toHaveURL(/orders/, { timeout: 3000 });
    });
});
