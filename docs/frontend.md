# Frontend — technical documentation

The frontend is a single-page application written in Rust, compiled to WebAssembly, and served as a static bundle. It communicates exclusively with the Go gateway over HTTP — it never talks directly to a database or microservice. This document covers the architecture, how state is managed, how routing works, how the API layer is structured, and what each part of the codebase does.

---

## Technology choices

**Rust + Yew** is an unconventional choice for a frontend — most SPAs are written in JavaScript or TypeScript. The decision here is intentional: it demonstrates Rust proficiency in a context where most developers would default to React or Vue, and it produces a highly performant WASM binary with no JavaScript runtime overhead.

**Trunk** is the build tool. It compiles the Rust code to WASM, runs the Tailwind CSS pre-build hook, injects the bundle into `index.html`, and serves the result on a local dev server with hot reload. It is the Yew equivalent of Vite or webpack.

**Tailwind CSS v3** handles all styling. Because Yew has no JavaScript build pipeline, Tailwind is run as a pre-build hook in `Trunk.toml` — it scans all `.rs` files for class names and generates a minified `tailwind.css` before each compilation.

---

## Directory structure

```
frontend/
├── src/
│   ├── main.rs                      # Entry point — mounts the App component
│   ├── route.rs                     # Route enum — all application routes defined here
│   ├── types.rs                     # All shared data types (structs for API shapes, cart, filters)
│   │
│   ├── context/                     # Global state providers
│   │   ├── mod.rs
│   │   ├── auth.rs                  # AuthContext — current user, token, login/logout
│   │   └── cart.rs                  # CartContext — cart items, quantities, totals
│   │
│   ├── services/                    # API call layer — one file per resource
│   │   ├── mod.rs
│   │   ├── api.rs                   # ApiClient — shared GET/POST/PUT methods
│   │   ├── auth.rs                  # AuthService — login, register, me
│   │   ├── products.rs              # ProductService — list, get, get_similar
│   │   └── orders.rs                # OrderService — list, get, create, cancel
│   │
│   ├── pages/                       # One file per route — top-level page components
│   │   ├── mod.rs
│   │   ├── landing.rs               # / — hero, featured products, category rows
│   │   ├── catalog.rs               # /catalog — search results with filters
│   │   ├── product_detail.rs        # /product/:id — full product page
│   │   ├── compare.rs               # /product/:id/compare — side-by-side comparison
│   │   ├── cart.rs                  # /cart — cart contents and totals
│   │   ├── checkout.rs              # /checkout — shipping and payment form
│   │   ├── order_confirmation.rs    # /order-confirmation/:id — post-purchase screen
│   │   ├── orders.rs                # /orders — order history list
│   │   ├── order_detail.rs          # /orders/:id — single order detail
│   │   ├── refund.rs                # /refund/:order_id — receipt upload and processing
│   │   ├── login.rs                 # /login
│   │   ├── register.rs              # /register
│   │   ├── profile.rs               # /profile — user info and store credit
│   │   └── not_found.rs             # /404
│   │
│   ├── components/                  # Reusable components, split by concern
│   │   ├── mod.rs
│   │   ├── layout/                  # App-wide structural components
│   │   │   ├── mod.rs
│   │   │   ├── navbar.rs            # Top navigation bar
│   │   │   ├── search_overlay.rs    # Fullscreen search modal
│   │   │   ├── chatbot_widget.rs    # Floating chat button and chat window
│   │   │   └── protected_route.rs  # Redirects to /login if not authenticated
│   │   ├── ui/                      # Generic UI primitives
│   │   │   ├── mod.rs
│   │   │   ├── button.rs
│   │   │   ├── toast.rs             # Notification toasts
│   │   │   ├── modal.rs             # Generic modal wrapper
│   │   │   ├── spinner.rs           # Loading spinner
│   │   │   └── badge.rs             # Status badges (in stock, low stock, etc.)
│   │   └── product/                 # Product-specific components
│   │       ├── mod.rs
│   │       ├── product_card.rs      # Card used in grids and rows
│   │       ├── product_grid.rs      # Responsive grid of product cards
│   │       └── attribute_table.rs   # Renders a product's JSONB attributes as a table
│   │
│   └── hooks/                       # Custom hooks for shared logic
│       ├── mod.rs
│       ├── use_api.rs               # Generic hook for async API calls with loading/error state
│       ├── use_auth.rs              # Hook that exposes auth context actions
│       └── use_cart.rs              # Hook that exposes cart context actions
│
├── styles/
│   ├── input.css                    # Tailwind directives + custom component classes
│   └── tailwind.css                 # Generated output — do not edit manually
│
├── index.html                       # HTML entry point — Trunk injects WASM bundle here
├── Trunk.toml                       # Build config — Tailwind pre-build hook defined here
├── tailwind.config.js               # Tailwind theme — colours, fonts, keyframes, animations
├── Cargo.toml                       # Rust dependencies
└── package.json                     # Node dependencies (Tailwind CLI only)
```

---

## How the app boots

When the browser loads `index.html`, it downloads and executes the compiled WASM bundle. The entry point is `src/main.rs`:

```rust
fn main() {
    yew::Renderer::<App>::new().render();
}
```

The `App` component wraps everything in three providers — the router, the auth context, and the cart context — then renders the navbar and a route switcher:

```rust
#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <AuthProvider>
                <CartProvider>
                    <div class="min-h-screen bg-navy">
                        <components::layout::navbar::Navbar />
                        <main>
                            <Switch<Route> render={switch} />
                        </main>
                    </div>
                </CartProvider>
            </AuthProvider>
        </BrowserRouter>
    }
}
```

The order of providers matters. `AuthProvider` wraps `CartProvider` because in future the cart may need to read auth state (e.g. to sync a server-side cart for logged-in users).

---

## Routing

All routes are defined in `src/route.rs` as a single Rust enum. Every variant maps to a URL pattern, and some carry parameters:

```rust
#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Landing,
    #[at("/product/:id")]
    ProductDetail { id: String },
    #[at("/orders/:id")]
    OrderDetail { id: String },
    #[at("/refund/:order_id")]
    Refund { order_id: String },
    // ... all other routes
}
```

The `switch` function in `main.rs` matches each variant to a page component. Route parameters are passed directly as component props:

```rust
fn switch(route: Route) -> Html {
    match route {
        Route::ProductDetail { id } =>
            html! { <pages::product_detail::ProductDetail {id} /> },
        Route::Refund { order_id } =>
            html! { <pages::refund::Refund {order_id} /> },
        // ...
    }
}
```

To navigate programmatically inside a component, use the `use_navigator` hook:

```rust
let navigator = use_navigator().unwrap();
navigator.push(&Route::OrderConfirmation { id: order.id.clone() });
```

To render a link in HTML, use the `Link` component:

```rust
html! {
    <Link<Route> to={Route::Catalog}>
        <span>{"Browse catalog"}</span>
    </Link<Route>>
}
```

### Protected routes

Pages that require authentication use the `ProtectedRoute` component, which checks the auth context and redirects to `/login` if the user is not logged in:

```rust
// In switch():
Route::Checkout => html! {
    <ProtectedRoute>
        <pages::checkout::Checkout />
    </ProtectedRoute>
},
```

The following routes require authentication:

- `/checkout`
- `/order-confirmation/:id`
- `/orders`
- `/orders/:id`
- `/refund/:order_id`
- `/profile`

---

## Global state

The app has two global state stores, both implemented using Yew's `use_reducer` pattern. This is Yew's equivalent of React's `useReducer` + Context API.

### AuthContext

Defined in `src/context/auth.rs`. Tracks the current user and JWT token.

**State shape:**

```rust
pub struct AuthState {
    pub user:       Option<User>,
    pub token:      Option<String>,
    pub is_loading: bool,
}
```

**Available actions:**

```rust
pub enum AuthAction {
    Login(AuthResponse),   // stores user + token, persists to localStorage
    Logout,                // clears user + token, removes from localStorage
    SetLoading(bool),      // toggles loading state during async auth calls
}
```

**Persistence:** When a user logs in, the token and user object are written to `localStorage` under the keys `starbound_token` and `starbound_user`. On app load, the provider reads these keys back and rehydrates the state — so a logged-in user stays logged in across browser refreshes without needing to re-authenticate.

**Using it in a component:**

```rust
let auth = use_context::<AuthContext>().expect("AuthContext not found");

// Check if logged in
if auth.is_authenticated() { ... }

// Get the current user
if let Some(user) = &auth.user { ... }

// Get the token for API calls
if let Some(token) = &auth.token {
    OrderService::list(token, 1).await
}

// Dispatch an action
auth.dispatch(AuthAction::Logout);
```

### CartContext

Defined in `src/context/cart.rs`. Tracks cart items and quantities in memory — resets when the app is closed or refreshed.

**State shape:**

```rust
pub struct CartState {
    pub items: Vec<CartItem>,  // each item is a ProductListItem + quantity
}
```

**Available actions:**

```rust
pub enum CartAction {
    AddItem(ProductListItem),          // adds 1 unit, or increments if already present
    RemoveItem(String),                // removes by product_id
    UpdateQuantity(String, i32),       // sets quantity; removes item if qty <= 0
    Clear,                             // empties the cart
}
```

**Computed values available on CartState:**

```rust
cart.total()        // sum of all line totals
cart.item_count()   // sum of all quantities
cart.contains(id)   // whether a product is already in the cart
```

**Using it in a component:**

```rust
let cart = use_context::<CartContext>().expect("CartContext not found");

// Add a product
cart.dispatch(CartAction::AddItem(product.clone()));

// Remove a product
cart.dispatch(CartAction::RemoveItem(product_id.clone()));

// Update quantity
cart.dispatch(CartAction::UpdateQuantity(product_id.clone(), new_qty));

// Display cart count in the navbar
html! { <span>{ cart.item_count() }</span> }
```

---

## API service layer

All HTTP communication lives in `src/services/`. No page or component ever calls `gloo_net` directly — they always go through a service.

### ApiClient (`src/services/api.rs`)

A low-level client with three methods: `get`, `post`, and `put`. Every method takes a path relative to `http://localhost:8000/api` and an optional JWT token. The token is automatically formatted as a `Bearer` header if provided.

```rust
// GET with no auth
ApiClient::get::<ProductListResponse>("/products?group=propulsion", None).await

// GET with auth
ApiClient::get::<Order>("/orders/abc-123", Some(&token)).await

// POST with body and auth
ApiClient::post::<CreateOrderRequest, Order>("/orders", &req, Some(&token)).await

// PUT with auth (no body — used for cancel)
ApiClient::put::<Order>("/orders/abc-123/cancel", Some(&token)).await
```

All methods return `Result<T, String>`. On a non-2xx response, the error contains the HTTP status code and response body as a string.

### ProductService (`src/services/products.rs`)

```rust
// Fetch a filtered, paginated list
ProductService::list(&filters).await

// Fetch a single product with full attributes
ProductService::get("le-001").await

// Fetch two similar products for the comparison page
// Returns up to 2 products of the same type, excluding the current product
ProductService::get_similar("liquid_engine", "le-001").await
```

### AuthService (`src/services/auth.rs`)

```rust
// Login
AuthService::login(LoginRequest { email, password }).await

// Register
AuthService::register(RegisterRequest { email, name, password }).await

// Get current user from token (used to verify a stored token is still valid)
AuthService::me(&token).await
```

### OrderService (`src/services/orders.rs`)

```rust
// List current user's orders (paginated)
OrderService::list(&token, page).await

// Get a single order
OrderService::get(&order_id, &token).await

// Create a new order
OrderService::create(&create_order_request, &token).await

// Cancel an order
OrderService::cancel(&order_id, &token).await
```

---

## Type system

All data shapes are defined in `src/types.rs`. The types mirror the Go gateway's JSON responses, so adding `#[serde(rename = "field_name")]` attributes ensures field names match exactly even when Go uses snake_case.

Key types and what they represent:

| Type                           | Purpose                                                  |
| ------------------------------ | -------------------------------------------------------- |
| `Product`                      | Full product record including `attributes: HashMap`      |
| `ProductListItem`              | Lightweight product for catalog listings (no attributes) |
| `ProductListResponse`          | Paginated wrapper: `data`, `total`, `page`, `limit`      |
| `ProductFilters`               | Filter parameters with `to_query_string()` helper        |
| `User`                         | Public user record (no password)                         |
| `AuthResponse`                 | Login/register response: `token` + `user`                |
| `Order`                        | Full order record with items and shipping address        |
| `CartItem`                     | In-memory cart entry: `ProductListItem` + `quantity`     |
| `CreateOrderRequest`           | Body for POST /api/orders                                |
| `ShippingAddress`              | Embedded in orders and checkout form                     |
| `ChatRequest` / `ChatResponse` | RAG chatbot message shapes                               |
| `RefundResponse`               | CV service response after processing a receipt           |

Product attributes are typed as `Option<HashMap<String, serde_json::Value>>` rather than a fixed struct. This reflects the fact that each of the 13 product types has completely different attributes — rendering them in the UI is handled dynamically by the `attribute_table` component, which iterates the key-value pairs and formats them for display.

---

## Styling system

All styles use Tailwind utility classes applied directly in Rust `html!` macros:

```rust
html! {
    <div class="bg-navy2 border border-border rounded-2xl p-6 hover:border-orange transition-all duration-200">
        <h2 class="font-orbitron text-lg font-bold text-white">{ &product.name }</h2>
        <p class="font-exo text-muted text-sm mt-1">{ &product.product_type }</p>
    </div>
}
```

### Design tokens

These custom values are defined in `tailwind.config.js` and available as utility classes:

**Colours:**
| Class | Hex | Usage |
|---|---|---|
| `bg-navy` / `text-navy` | `#0a0f1e` | Page background |
| `bg-navy2` | `#0d1526` | Card and navbar background |
| `bg-navy3` | `#111d35` | Input backgrounds, hover states |
| `bg-navy4` | `#162040` | Active states, selected items |
| `border-border` | `#1e2e50` | Default border colour |
| `text-orange` / `bg-orange` | `#f4681a` | Primary accent — prices, CTAs, active states |
| `bg-orange2` | `#e05510` | Orange hover state |
| `text-muted` | `#7a8aaa` | Secondary text |
| `text-dim` | `#3a4e70` | Placeholder text, disabled states |

**Fonts:**

- `font-orbitron` — display font for headings, logos, prices, SKUs
- `font-exo` — body font for all readable text

### Reusable component classes

Defined in `styles/input.css` using Tailwind's `@layer components`:

```css
.btn-primary    /* orange filled button */
.btn-ghost      /* bordered transparent button */
.btn-outline    /* orange bordered button */
.card           /* hoverable navy card with orange border on hover */
.card-static    /* non-hoverable card */
.input-field    /* dark input with orange focus ring */
.select-field   /* dark select with orange focus ring */
.label-mono     /* small uppercase orbitron label */
.price-text     /* orange orbitron price */
.badge-stock    /* green "In stock" badge */
.badge-low      /* orange "X left" badge */
.badge-pre      /* indigo "Pre-order" badge */
.skeleton       /* shimmer loading placeholder */
```

### Animations

All animations are defined as Tailwind keyframes in `tailwind.config.js` and applied as utility classes:

| Class                    | Effect                        | Usage                |
| ------------------------ | ----------------------------- | -------------------- |
| `animate-fade-up`        | Fades in while moving up 20px | Page content on load |
| `animate-fade-in`        | Simple opacity fade           | Overlays, modals     |
| `animate-slide-in-right` | Slides in from the right      | Cart panel, drawers  |
| `animate-slide-in-left`  | Slides in from the left       | Filter panel         |
| `animate-scale-in`       | Scales from 95% to 100%       | Modals, dropdowns    |
| `animate-pulse-glow`     | Orange glow pulse             | CTAs, featured items |
| `animate-float`          | Gentle up/down bob            | Hero elements        |
| `animate-shimmer`        | Moving highlight              | Skeleton loaders     |

---

## Pages overview

| Page               | Route                     | Auth | Key functionality                                             |
| ------------------ | ------------------------- | ---- | ------------------------------------------------------------- |
| Landing            | `/`                       | No   | Hero, category nav, featured products, category rows, chatbot |
| Catalog            | `/catalog`                | No   | Search results, filter sidebar, product grid                  |
| Product detail     | `/product/:id`            | No   | Image, specs, add to cart, compare link                       |
| Compare            | `/product/:id/compare`    | No   | Side-by-side comparison of 3 products                         |
| Cart               | `/cart`                   | No   | Item list, quantities, totals, checkout CTA                   |
| Checkout           | `/checkout`               | Yes  | Shipping form, payment method, order summary                  |
| Order confirmation | `/order-confirmation/:id` | Yes  | Order ID, items, total, receipt download                      |
| Orders             | `/orders`                 | Yes  | Paginated order history                                       |
| Order detail       | `/orders/:id`             | Yes  | Full order with status, items, shipping                       |
| Refund             | `/refund/:order_id`       | Yes  | PDF upload, CV processing, refund decision                    |
| Login              | `/login`                  | No   | Email + password, link to register                            |
| Register           | `/register`               | No   | Email + name + password, link to login                        |
| Profile            | `/profile`                | Yes  | Name, store credit, order history summary                     |
| 404                | `/404`                    | No   | Not found page                                                |

---

## Build and development

### Running the dev server

```bash
cd frontend
trunk serve
# Available at http://localhost:8080
# Hot reload on file save
```

### Production build

```bash
trunk build --release
# Output in frontend/dist/
# This is what Electron loads in production
```

### How Tailwind is integrated

Trunk runs Tailwind as a pre-build hook before each compilation, defined in `Trunk.toml`:

```toml
[[hooks]]
stage = "pre_build"
command = "node_modules/.bin/tailwindcss.cmd"
command_arguments = [
    "-i", "./styles/input.css",
    "-o", "./styles/tailwind.css",
    "--minify"
]
```

Tailwind scans all `.rs` files for class names. This means class names must appear as complete strings in the source — they cannot be dynamically constructed, as Tailwind's scanner is a static text search, not a runtime evaluator.

```rust
// CORRECT — full class name present in source
let active_class = if is_active { "bg-orange text-white" } else { "bg-navy3 text-muted" };

// WRONG — Tailwind will not detect these
let colour = "orange";
let class = format!("bg-{} text-white", colour);  // bg-orange never appears as a string
```

### Adding a new page

1. Create the file in `src/pages/your_page.rs`
2. Add the component function with `#[function_component]`
3. Add a variant to `Route` in `src/route.rs`
4. Add a match arm to the `switch` function in `src/main.rs`
5. Add the module to `src/pages/mod.rs`
6. If auth is required, wrap in `<ProtectedRoute>` in the switch arm

---

## Electron integration

In development, Electron points at the Trunk dev server:

```javascript
// electron/main.js
if (isDev) {
  win.loadURL("http://localhost:8080");
}
```

In production, Electron loads the compiled `dist/` folder directly as a local file:

```javascript
win.loadFile(path.join(__dirname, "../frontend/dist/index.html"));
```

This means the frontend WASM bundle needs to be built with `trunk build --release` before packaging the Electron app. No code changes to the frontend are required for desktop vs browser — they are identical builds.

---

## What is not yet implemented

As of the current state of the project, the following are stubbed and pending:

- All page content (pages render placeholder headings only)
- `SearchOverlay` component
- `ChatbotWidget` component
- All `ui/` components (button, toast, modal, spinner, badge)
- All `product/` components (product_card, product_grid, attribute_table)
- All custom hooks (use_api, use_auth, use_cart)
- `ProtectedRoute` is wired but not yet applied to protected pages in the switch function

These will be built out page by page in subsequent development sessions.
