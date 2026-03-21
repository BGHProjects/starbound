import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
    testDir: "./e2e",
    fullyParallel: false,
    retries: process.env.CI ? 2 : 0,
    reporter: "html",
    use: {
        baseURL: "http://localhost:8080",
        trace:   "on-first-retry",
        screenshot: "only-on-failure",
    },
    projects: [
        { name: "chromium", use: { ...devices["Desktop Chrome"] } },
    ],
    webServer: [
        {
            command:             "go run cmd/main.go",
            url:                 "http://localhost:8000/health",
            cwd:                 "../gateway",
            reuseExistingServer: true,
            timeout:             30000,
        },
        {
            command:             "trunk serve",
            url:                 "http://localhost:8080",
            cwd:                 "../frontend",
            reuseExistingServer: true,
            timeout:             60000,
        },
    ],
});
