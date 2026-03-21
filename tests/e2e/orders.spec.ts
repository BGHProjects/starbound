import { test, expect, Page } from "@playwright/test";

test.describe.serial("Orders", () => {
    let testEmail: string;
    const testPassword = "playwright123";
    const testName     = "Orders User";

    test.beforeAll(async ({ browser }) => {
        testEmail = `orders-${Date.now()}@playwright.dev`;
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

    // Complete order flow in a single page session to preserve WASM cart state
    async function placeOrder(page: Page) {
        // Step 1: Login
        await login(page);

        // Step 2: Add to cart — use Yew router navigation, not page.goto
        // Click the catalog link in navbar to stay in same WASM session
        await page.locator("nav").getByText("Cart").click();
        await expect(page).toHaveURL(/cart/, { timeout: 3000 });
        await page.getByRole("button", { name: /browse catalog/i }).click();
        await expect(page).toHaveURL(/catalog/, { timeout: 3000 });
        await expect(page.locator(".card").first()).toBeVisible({ timeout: 10000 });
        const firstCard = page.locator(".card").first();
        await firstCard.getByRole("button", { name: "Add to cart" }).click();
        await expect(firstCard.getByText(/Added/)).toBeVisible({ timeout: 5000 });

        // Step 3: Navigate to checkout via cart page using Yew router links
        await page.locator("nav").getByText("Cart").click();
        await expect(page).toHaveURL(/cart/, { timeout: 3000 });
        await expect(page.locator("nav").getByText("1")).toBeVisible({ timeout: 5000 });

        // Step 4: Click proceed to checkout
        await page.getByRole("button", { name: /proceed to checkout/i }).click();
        await expect(page).toHaveURL(/checkout/, { timeout: 5000 });

        // Step 5: Fill checkout form
        await expect(page.locator("input[placeholder='e.g. Kennedy Space Center']")).toBeVisible({ timeout: 8000 });
        await page.locator("input[placeholder='e.g. Kennedy Space Center']").fill("Test Facility");
        await page.locator("input[placeholder='e.g. LC-39A']").fill("TC-01");
        await page.locator("input[placeholder='Street address']").fill("123 Test Street");
        await page.locator("input[placeholder='City']").fill("London");
        await page.locator("input[placeholder='e.g. US']").fill("UK");
        await page.locator("input[placeholder='Postal / ZIP code']").fill("SW1A 1AA");
        await page.getByRole("button", { name: "Place order" }).click();
        await expect(page).toHaveURL(/order-confirmation/, { timeout: 15000 });
    }

    test("orders list page shows empty state for new user", async ({ page }) => {
        await login(page);
        await page.locator("nav").getByText("Cart").click();
        await page.goto("/orders");
        await expect(page.getByText("No orders yet")).toBeVisible({ timeout: 5000 });
    });

    test("full order flow — add to cart, checkout, confirmation", async ({ page }) => {
        await placeOrder(page);
        await expect(page.getByText("Order confirmed")).toBeVisible();
        await expect(page.getByText("Order ID")).toBeVisible();
    });

    test("order appears in order history after purchase", async ({ page }) => {
        await placeOrder(page);
        await page.getByRole("button", { name: /view all orders/i }).click();
        await expect(page).toHaveURL(/\/orders$/, { timeout: 5000 });
        await expect(page.locator(".card-static").first()).toBeVisible();
    });

    test("order detail page loads from order history", async ({ page }) => {
        await placeOrder(page);
        await page.getByRole("button", { name: /view all orders/i }).click();
        await page.locator(".card-static").first().click();
        await expect(page).toHaveURL(/orders\//, { timeout: 5000 });
        await expect(page.getByText("Items")).toBeVisible();
        await expect(page.getByText("Shipping address")).toBeVisible();
    });

    test("cancel order button is visible for pending order", async ({ page }) => {
        await placeOrder(page);
        await page.getByRole("button", { name: /view all orders/i }).click();
        await page.locator(".card-static").first().click();
        await expect(page.getByRole("button", { name: /cancel order/i })).toBeVisible();
    });

    test("download receipt button is visible on order detail", async ({ page }) => {
        await placeOrder(page);
        await page.getByRole("button", { name: /view all orders/i }).click();
        await page.locator(".card-static").first().click();
        await expect(page.getByRole("button", { name: /download receipt/i })).toBeVisible();
    });
});
