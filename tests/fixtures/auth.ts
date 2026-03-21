import { test as base, expect, Page } from "@playwright/test";

export type AuthFixtures = {
    authenticatedPage: Page;
};

const timestamp    = Date.now();
export const testEmail    = `e2e-${timestamp}@playwright.dev`;
export const testPassword = "playwright123";
export const testName     = "E2E Test User";

export async function registerAndLogin(page: Page, email: string, password: string, name: string) {
    await page.goto("/register");
    await page.waitForLoadState("networkidle");
    await page.locator("input[type='text']").fill(name);
    await page.locator("input[type='email']").fill(email);
    await page.locator("input[type='password']").fill(testPassword);
    await page.getByRole("button", { name: "Create account" }).click();
    await expect(page).toHaveURL("http://localhost:8080/", { timeout: 15000 });
}

export const test = base.extend<AuthFixtures>({
    authenticatedPage: async ({ page }, use) => {
        await registerAndLogin(page, testEmail, testPassword, testName);
        await use(page);
    },
});
