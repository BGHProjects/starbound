# Testing — technical documentation

Starbound has two layers of testing: Go unit tests covering the gateway's handlers and business logic, and Playwright end-to-end tests covering the full user-facing application from the browser.

---

## Go unit tests

### Overview

The gateway has 48 unit tests across three test files covering all endpoints, authentication flows, and order business logic. Tests use temporary in-memory stores so they never touch real data files.

### Running
```bash
cd gateway
go test ./... -v
```

To run a specific test file:
```bash
go test ./tests/ -run TestProducts -v
go test ./tests/ -run TestAuth -v
go test ./tests/ -run TestOrders -v
```

### Coverage

**`tests/products_test.go`**
- GET /api/products — returns paginated list
- GET /api/products — filter by group
- GET /api/products — filter by product type
- GET /api/products — text search
- GET /api/products/groups — returns full taxonomy
- GET /api/products/:id — returns full product with attributes
- GET /api/products/:id — 404 for unknown ID

**`tests/auth_test.go`**
- POST /api/auth/register — creates user and returns JWT
- POST /api/auth/register — rejects duplicate email with 409
- POST /api/auth/register — rejects short password
- POST /api/auth/login — returns JWT for valid credentials
- POST /api/auth/login — rejects wrong password with 401
- POST /api/auth/login — rejects unknown email with 401
- GET /api/auth/me — returns user for valid token
- GET /api/auth/me — rejects missing token with 401
- GET /api/auth/me — rejects invalid token with 401
- Protected routes reject requests without auth header

**`tests/orders_test.go`**
- POST /api/orders — creates order for authenticated user
- POST /api/orders — rejects out of stock items
- POST /api/orders — rejects missing required shipping fields
- GET /api/orders — returns paginated order history for user
- GET /api/orders — only returns orders belonging to the requesting user
- GET /api/orders/:id — returns full order detail
- GET /api/orders/:id — returns 403 for orders belonging to another user
- GET /api/orders/:id — returns 404 for unknown order
- PUT /api/orders/:id/cancel — cancels a pending order
- PUT /api/orders/:id/cancel — rejects cancellation of shipped order
- PUT /api/orders/:id/cancel — rejects cancellation by non-owner

---

## Playwright end-to-end tests

### Overview

49 end-to-end tests across 7 spec files covering the full user journey through the browser. Tests run against the live frontend and gateway — both must be running before the test suite is executed.

### Prerequisites
```bash
# Install Playwright browsers (one-time setup)
cd tests
npx playwright install chromium
```

Both the gateway and frontend dev server must be running. The Playwright config starts them automatically if they are not already running:
```bash
# Terminal 1
cd gateway && go run cmd/main.go

# Terminal 2
cd frontend && trunk serve
```

### Running
```bash
cd tests

# Run all tests
npx playwright test

# Run with list reporter (shows each test as it runs)
npx playwright test --reporter=list

# Run a specific spec file
npx playwright test e2e/auth.spec.ts

# Run in UI mode (visual test runner)
npx playwright test --ui

# View the last HTML report
npx playwright show-report
```

### Important note on cart state

The Starbound frontend is a Yew/WASM application where cart state lives in memory. This state does not survive a full page reload (`page.goto()`). The E2E tests account for this by navigating within the same WASM session using in-app links rather than triggering full reloads. Tests that require cart state use Playwright's navigation within the running app rather than direct URL navigation.

### Test files

**`e2e/health.spec.ts`** — 2 tests

Basic smoke tests verifying services are reachable:
- Frontend loads with correct title
- Gateway health endpoint returns `{ status: "ok" }`

**`e2e/landing.spec.ts`** — 6 tests

Landing page content and navigation:
- Hero section text is visible
- Browse Catalog button navigates to catalog
- Create Account button navigates to register
- Featured products section loads from API
- All four category row headings load
- Navbar shows Sign in and Sign up links when logged out

**`e2e/auth.spec.ts`** — 7 tests

Authentication flows:
- Register page renders correctly
- Login page renders correctly
- Successful registration redirects to landing page
- Navbar shows user's first name after login
- Wrong password shows error message
- Duplicate email shows error message
- Protected route redirects unauthenticated user to login

**`e2e/catalog.spec.ts`** — 7 tests

Product catalog and filtering:
- Catalog loads with product cards
- Filter sidebar is visible on desktop
- Clicking a category filter updates results
- In-stock toggle filters products
- Search bar filters products
- Clicking a product card navigates to product detail
- Clear filters button resets all active filters

**`e2e/product.spec.ts`** — 6 tests

Product detail page:
- Product detail page loads with specifications
- Breadcrumb navigation is visible
- Add to cart button shows confirmation state
- Cart count in navbar increments after adding item
- Compare button navigates to compare page
- Catalog breadcrumb navigates back to catalog

**`e2e/compare.spec.ts`** — 3 tests

Product comparison page:
- Compare page loads with product cards
- Back to product button navigates to product detail
- Breadcrumb shows correct navigation path

**`e2e/cart.spec.ts`** — 7 tests

Cart functionality:
- Empty cart shows browse catalog button
- Cart count increments when item added from catalog
- Cart icon in navbar navigates to cart page
- Cart page shows empty state initially
- Browse catalog button on empty cart navigates to catalog
- Add to cart button shows confirmation state
- Multiple items increment cart count correctly

**`e2e/orders.spec.ts`** — 6 tests (serial)

Full order flow — runs serially to avoid email conflicts and maintain WASM session state:
- Orders list shows empty state for new user
- Full order flow: add to cart → checkout → confirmation
- Order appears in order history after purchase
- Order detail page loads from order history
- Cancel order button visible for pending order
- Download receipt button visible on order detail

**`e2e/profile.spec.ts`** — 5 tests (serial)

User profile:
- Profile page shows user name and email
- Profile shows account stats section
- Profile shows avatar initials
- Sign out redirects to landing and shows Sign in
- View all orders link navigates to orders page

### Serial test suites

The `orders` and `profile` suites use `test.describe.serial` instead of running in parallel. This is necessary because:

- Each test registers a new user account — parallel execution can cause email timestamp collisions
- The orders suite places real orders, and the tests depend on order state from previous tests in the suite
- Checkout requires an active WASM session with cart state, which is built up across test steps

### Test data

Tests create their own user accounts using timestamped emails (`orders-1234567890@playwright.dev`) to avoid conflicts across runs. These accounts persist in `gateway/internal/db/users.json` — you may want to periodically clear this file if running tests frequently.

---

## CI configuration

The Playwright config is set up for CI with `retries: 2` on the `CI` environment variable. In CI, the web servers are started fresh rather than reusing existing ones:
```yaml
# .github/workflows example
- name: Run E2E tests
  env:
    CI: true
  run: |
    cd gateway && go run cmd/main.go &
    cd frontend && trunk serve &
    cd tests && npx playwright test
```

---

## What is not covered

The following areas are not currently covered by automated tests:

- RAG service chat responses (requires live OpenAI API — not suitable for automated testing without mocking)
- CV service PDF processing (requires real PDF files and Tesseract/Poppler)
- Electron desktop app (Playwright can test Electron apps but is not configured here)
- Refund flow end-to-end (requires CV service running)
- Chatbot widget interactions
- Mobile responsive layouts
