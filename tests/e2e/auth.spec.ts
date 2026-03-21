import { test, expect } from "@playwright/test";

const timestamp    = Date.now();
const testEmail    = `auth-test-${timestamp}@playwright.dev`;
const testPassword = "playwright123";
const testName     = "Auth Test User";

test.describe("Authentication", () => {
    test("register page loads correctly", async ({ page }) => {
        await page.goto("/register");
        await expect(page.getByText("Create your account")).toBeVisible();
        await expect(page.locator("input[type='text']")).toBeVisible();
        await expect(page.locator("input[type='email']")).toBeVisible();
        await expect(page.locator("input[type='password']")).toBeVisible();
    });

    test("login page loads correctly", async ({ page }) => {
        await page.goto("/login");
        await expect(page.getByText("Welcome back")).toBeVisible();
        await expect(page.locator("input[type='email']")).toBeVisible();
        await expect(page.locator("input[type='password']")).toBeVisible();
    });

    test("register with valid credentials redirects to landing", async ({ page }) => {
        await page.goto("/register");
        await page.waitForLoadState("networkidle");
        await page.locator("input[type='text']").fill(testName);
        await page.locator("input[type='email']").fill(testEmail);
        await page.locator("input[type='password']").fill(testPassword);
        await page.getByRole("button", { name: "Create account" }).click();
        await expect(page).toHaveURL("http://localhost:8080/", { timeout: 15000 });
    });

    test("navbar shows user first name after login", async ({ page }) => {
        await page.goto("/login");
        await page.locator("input[type='email']").fill(testEmail);
        await page.locator("input[type='password']").fill(testPassword);
        await page.getByRole("button", { name: "Sign in" }).click();
        await expect(page).toHaveURL("http://localhost:8080/", { timeout: 10000 });
        await expect(page.locator("nav").getByText("Auth")).toBeVisible();
    });

    test("login with wrong password shows error", async ({ page }) => {
        await page.goto("/login");
        await page.locator("input[type='email']").fill(testEmail);
        await page.locator("input[type='password']").fill("wrongpassword");
        await page.getByRole("button", { name: "Sign in" }).click();
        await expect(page.getByText(/invalid email or password/i)).toBeVisible({ timeout: 5000 });
    });

    test("register with existing email shows error", async ({ page }) => {
        await page.goto("/register");
        await page.waitForLoadState("networkidle");
        await page.locator("input[type='text']").fill(testName);
        await page.locator("input[type='email']").fill(testEmail);
        await page.locator("input[type='password']").fill(testPassword);
        await page.getByRole("button", { name: "Create account" }).click();
        await expect(page.getByText(/already exists/i)).toBeVisible({ timeout: 5000 });
    });

    test("protected route redirects to login when not authenticated", async ({ page }) => {
        await page.goto("/profile");
        await expect(page).toHaveURL(/login/, { timeout: 5000 });
    });
});
