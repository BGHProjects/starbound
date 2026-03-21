import { test, expect } from "@playwright/test";

test.describe("Health checks", () => {
    test("frontend loads with correct title", async ({ page }) => {
        await page.goto("/");
        await expect(page).toHaveTitle(/Starbound/);
    });

    test("gateway health endpoint returns ok", async ({ request }) => {
        const resp = await request.get("http://localhost:8000/health");
        expect(resp.ok()).toBeTruthy();
        const body = await resp.json();
        expect(body.status).toBe("ok");
    });
});
