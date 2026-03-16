# Frontend — technical documentation

The frontend is a single-page application written in Rust, compiled to WebAssembly, and served as a static bundle. It communicates exclusively with the Go gateway over HTTP — it never talks directly to a database or microservice. This document covers the architecture, how state is managed, how routing works, how the API layer is structured, and what each part of the codebase does.

---

## Technology choices

**Rust + Yew** is an unconventional choice for a frontend — most SPAs are written in JavaScript or TypeScript. The decision here is intentional: it demonstrates Rust proficiency in a context where most developers would default to React or Vue, and it produces a highly performant WASM binary with no JavaScript runtime overhead.

**Trunk** is the build tool. It compiles the Rust code to WASM, runs the Tailwind CSS pre-build hook, injects the bundle into `index.html`, and serves the result on a local dev server with hot reload.

**Tailwind CSS v3** handles all styling. Because Yew has no JavaScript build pipeline, Tailwind is run as a pre-build hook in `Trunk.toml` — it scans all `.rs` files for class names and generates a minified `tailwind.css` before each compilation.

---

## Directory structure

```
frontend/
├── src/
│   ├── main.rs                      # Entry point — mounts App, wraps providers
│   ├── route.rs                     # Route enum — all application routes
│   ├── types.rs                     # All shared data types
│   │
│   ├── context/
│   │   ├── auth.rs                  # AuthContext — user, token, login/logout
│   │   └── cart.rs                  # CartContext — items, quantities, totals
│   │
│   ├── services/
│   │   ├── api.rs                   # ApiClient — GET/POST/PUT with optional auth
│   │   ├── auth.rs                  # AuthService — login, register, me
│   │   ├── products.rs              # ProductService — list, get, get_similar
│   │   └── orders.rs                # OrderService — list, get, create, cancel
│   │
│   ├── pages/
│   │   ├── landing.rs               # / — hero, category pills, product rows
│   │   ├── catalog.rs               # /catalog — filters, search, product grid
│   │   ├── product_detail.rs        # /product/:id — specs, add to cart, compare
│   │   ├── compare.rs               # /product/:id/compare — side by side cards
│   │   ├── cart.rs                  # /cart — items, quantities, order summary
│   │   ├── checkout.rs              # /checkout — shipping form, payment, summary
│   │   ├── order_confirmation.rs    # /order-confirmation/:id — post-purchase
│   │   ├── orders.rs                # /orders — paginated order history
│   │   ├── order_detail.rs          # /orders/:id — full order, cancel, refund
│   │   ├── refund.rs                # /refund/:order_id — PDF upload, CV result
│   │   ├── login.rs                 # /login
│   │   ├── register.rs              # /register
│   │   ├── profile.rs               # /profile — stats, recent orders, sign out
│   │   └── not_found.rs             # /404
│   │
│   ├── components/
│   │   ├── layout/
│   │   │   ├── navbar.rs            # Top navigation bar
│   │   │   ├── chatbot_widget.rs    # Floating chat window (stub)
│   │   │   └── protected_route.rs  # Redirects to /login if not authenticated
│   │   ├── ui/
│   │   │   ├── spinner.rs           # Loading spinner — sm/md/lg sizes
│   │   │   ├── tooltip.rs           # Hover tooltip with optional external link
│   │   │   ├── button.rs            # (stub)
│   │   │   ├── toast.rs             # (stub)
│   │   │   ├── modal.rs             # (stub)
│   │   │   └── badge.rs             # (stub)
│   │   └── product/
│   │       ├── product_card.rs      # Card used in all grids and rows
│   │       ├── product_grid.rs      # (stub)
│   │       └── attribute_table.rs   # (stub)
│   │
│   └── hooks/
│       ├── use_api.rs               # (stub)
│       ├── use_auth.rs              # (stub)
│       └── use_cart.rs              # (stub)
│
├── styles/
│   ├── input.css                    # Tailwind directives + component classes
│   └── tailwind.css                 # Generated — do not edit manually
│
├── index.html
├── Trunk.toml
├── tailwind.config.js
├── Cargo.toml
└── package.json
```

---

## How the app boots

```rust
fn main() {
    yew::Renderer::<App>::new().render();
}
```

The `App` component wraps everything in the router, auth context, and cart context:

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

---

## Routing

All routes are defined in `src/route.rs`:

```rust
#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]                        Landing,
    #[at("/catalog/:group")]          CatalogFiltered { group: String },
    #[at("/catalog")]                 Catalog,
    #[at("/product/:id")]             ProductDetail { id: String },
    #[at("/product/:id/compare")]     Compare { id: String },
    #[at("/cart")]                    Cart,
    #[at("/checkout")]                Checkout,
    #[at("/order-confirmation/:id")]  OrderConfirmation { id: String },
    #[at("/orders")]                  Orders,
    #[at("/orders/:id")]              OrderDetail { id: String },
    #[at("/refund/:order_id")]        Refund { order_id: String },
    #[at("/login")]                   Login,
    #[at("/register")]                Register,
    #[at("/profile")]                 Profile,
    #[not_found] #[at("/404")]        NotFound,
}
```

`CatalogFiltered` vs `Catalog` — when a user clicks a category on the landing page they are routed to `/catalog/propulsion` etc., which pre-fills the group filter. The unfiltered `/catalog` route passes `initial_group: None`.

### Protected routes

The `ProtectedRoute` component checks the auth context and redirects to `/login` if the user is not authenticated. The following pages require auth:

- `/checkout`
- `/order-confirmation/:id`
- `/orders`
- `/orders/:id`
- `/refund/:order_id`
- `/profile`

---

## Global state

### AuthContext (`src/context/auth.rs`)

Tracks the current user and JWT token. Persists to `localStorage` on login so the session survives a page refresh.

```rust
pub struct AuthState {
    pub user:       Option<User>,
    pub token:      Option<String>,
    pub is_loading: bool,
}

pub enum AuthAction {
    Login(AuthResponse),   // stores user + token, writes to localStorage
    Logout,                // clears user + token, removes from localStorage
    SetLoading(bool),
}
```

Usage in a component:

```rust
let auth = use_context::<AuthContext>().expect("AuthContext not found");

auth.is_authenticated()                    // bool
auth.token.clone()                         // Option<String>
auth.user.as_ref().map(|u| u.name.clone()) // Option<String>
auth.dispatch(AuthAction::Logout);
```

### CartContext (`src/context/cart.rs`)

In-memory cart — resets on page refresh. No localStorage persistence.

```rust
pub struct CartState {
    pub items: Vec<CartItem>,  // ProductListItem + quantity
}

pub enum CartAction {
    AddItem(ProductListItem),       // adds 1, or increments if already present
    RemoveItem(String),             // remove by product_id
    UpdateQuantity(String, i32),    // set quantity; removes if qty <= 0
    Clear,
}
```

Computed helpers: `cart.total()`, `cart.item_count()`, `cart.contains(id)`.

---

## API service layer

All HTTP calls go through `src/services/`. No component ever calls `gloo_net` directly.

### ApiClient

```rust
ApiClient::get::<T>("/products?group=propulsion", None).await
ApiClient::get::<T>("/orders/abc", Some(&token)).await
ApiClient::post::<Body, T>("/orders", &req, Some(&token)).await
ApiClient::put::<T>("/orders/abc/cancel", Some(&token)).await
```

All return `Result<T, String>`. Non-2xx responses return `Err("HTTP 404: ...")`.

### Services

```rust
// Products
ProductService::list(&filters).await
ProductService::get("le-001").await
ProductService::get_similar("liquid_engine", "le-001").await

// Auth
AuthService::login(LoginRequest { email, password }).await
AuthService::register(RegisterRequest { email, name, password }).await
AuthService::me(&token).await

// Orders
OrderService::list(&token, page).await
OrderService::get(&order_id, &token).await
OrderService::create(&req, &token).await
OrderService::cancel(&order_id, &token).await
```

---

## Pages

### Landing (`/`)

Fetches products from the gateway on mount — four separate requests, one per group, plus a featured request. Uses `use_effect_with((), ...)` to fire once. Each category row passes `group` to `CatalogFiltered` when the user clicks "View all".

### Catalog (`/catalog` and `/catalog/:group`)

Accepts an `initial_group: Option<String>` prop from the router. Filter state is held in component state — group, type, in-stock toggle, min/max price. Price sliders use a two-level debounce pattern:

- `min_price` / `max_price` — update immediately on every slider event (display value)
- `committed_min` / `committed_max` — only update 400ms after the slider stops moving (triggers API call)
- Each slider has its own `Rc<RefCell<Option<Timeout>>>` debounce handle so pending timeouts are cancelled before scheduling a new one

Grid transitions use a three-phase animation state (`GridPhase::Idle`, `Exiting`, `Entering`) — products fade out over 250ms, then the new set fades in with a staggered delay of 50ms per card.

### Product detail (`/product/:id`)

Fetches the full product (including attributes) on mount. Attributes are rendered as a key-value list with label formatting — underscores replaced with spaces, unit suffixes stripped and placed in brackets. Add to cart button shows a 1.5s "Added ✓" confirmation state.

### Compare (`/product/:id/compare`)

Fetches the current product, then calls `get_similar` to find up to two products of the same type. Renders as fixed-width (`w-80`) vertical cards — image, name in a fixed `h-12` container (prevents misalignment from long names), stock badge, CTA button, then all specs listed vertically with label above value. Numeric attributes are highlighted green (highest) or red (lowest) across the compared products. On mobile, cards are horizontally scrollable with snap points and prev/next navigation buttons.

### Cart (`/cart`)

Reads directly from `CartContext` — no API call needed. Quantity controls dispatch `UpdateQuantity` (removes item if quantity reaches zero). Shows auth-aware checkout CTA: "Sign in to checkout" for guests, "Proceed to checkout" for authenticated users.

### Checkout (`/checkout`)

Redirects to `/login` if not authenticated, to `/cart` if cart is empty. On submit, builds a `CreateOrderRequest` from form state and cart items, calls `OrderService::create`, clears the cart on success and navigates to order confirmation. Facility name and site code fields are optional with tooltip components explaining their purpose.

### Refund (`/refund/:order_id`)

Four-stage UI controlled by a `RefundStage` enum: `Upload`, `Processing`, `Success { valid, reason }`, `Error(String)`. File selection via a hidden `<input type="file">` triggered by clicking the upload area. On selection, validates the file is a PDF, then POSTs as `multipart/form-data` to the CV service at `http://localhost:8002/api/refund/validate`.

### Orders and Order detail

Both require auth. Order detail shows cancel button for orders in cancellable statuses (`pending`, `payment_processing`, `payment_failed`, `confirmed`). Shows "Request refund" button for `shipped` or `delivered` orders. Order status badges are colour-coded consistently across all pages using a shared `status_color()` helper.

---

## Components

### Navbar

Responsive. Logo text hidden on very small screens (icon only). Cart shows icon always, text on `sm+` screens. Auth state:

- Logged out: "Sign in" ghost button + "Sign up" primary button
- Logged in: avatar circle with initial + first name (truncated), links to profile

### ProductCard

The entire card is wrapped in a `Link` to the product detail page. The "Add to cart" button uses `e.stop_propagation()` to prevent the click bubbling to the link. Shows stock badge (green / orange / indigo). Price formatted as `$40K`, `$1.5M` etc.

### Spinner

Three sizes: `SpinnerSize::Sm`, `Md`, `Lg`. Used on all loading states across the app.

### Tooltip

Hover-triggered bubble with optional external link. The hover zone covers both the `i` icon and the bubble itself — `onmouseenter`/`onmouseleave` are on the wrapper span so moving the cursor from the icon to the bubble doesn't dismiss it. Font family is set inline on the bubble to prevent inheriting `font-orbitron` from parent label elements.

---

## Styling system

### Design tokens

| Class                       | Value     | Usage                           |
| --------------------------- | --------- | ------------------------------- |
| `bg-navy`                   | `#0a0f1e` | Page background                 |
| `bg-navy2`                  | `#0d1526` | Card and navbar background      |
| `bg-navy3`                  | `#111d35` | Input backgrounds, hover states |
| `bg-navy4`                  | `#162040` | Active states                   |
| `border-border`             | `#1e2e50` | Default border                  |
| `text-orange` / `bg-orange` | `#f4681a` | Primary accent                  |
| `bg-orange2`                | `#e05510` | Orange hover                    |
| `text-muted`                | `#7a8aaa` | Secondary text                  |
| `text-dim`                  | `#3a4e70` | Placeholder, disabled           |

### Fonts

- `font-orbitron` — display font for headings, prices, SKUs, order IDs
- `font-exo` — body font for all readable text, labels, descriptions

### Reusable component classes (`styles/input.css`)

```
.btn-primary    .btn-ghost      .btn-outline
.card           .card-static
.input-field    .select-field
.label-mono     .price-text
.badge-stock    .badge-low      .badge-pre
.skeleton
.scrollbar-hide
.line-clamp-2
```

### Animations

| Class                    | Effect              |
| ------------------------ | ------------------- |
| `animate-fade-up`        | Fade in + rise 20px |
| `animate-fade-in`        | Opacity fade        |
| `animate-slide-in-right` | Slide from right    |
| `animate-slide-in-left`  | Slide from left     |
| `animate-scale-in`       | Scale from 95%      |
| `animate-pulse-glow`     | Orange glow pulse   |
| `animate-float`          | Gentle bob          |
| `animate-shimmer`        | Skeleton shimmer    |

Animation fill mode is `both` — the initial keyframe state (opacity: 0) is applied before the animation starts, which makes staggered entrances work correctly with `animation-delay`.

---

## Build and development

```bash
cd frontend
trunk serve          # dev server at http://localhost:8080
trunk build --release  # production build → frontend/dist/
```

### Tailwind integration

Trunk runs Tailwind as a pre-build hook (`Trunk.toml`). Class names must appear as complete strings in source — dynamic construction like `format!("bg-{}", colour)` will not be detected by Tailwind's scanner.

### Adding a new page

1. Create `src/pages/your_page.rs`
2. Add `#[function_component]`
3. Add variant to `Route` in `src/route.rs`
4. Add match arm to `switch` in `src/main.rs`
5. Add module to `src/pages/mod.rs`
6. Wrap in `<ProtectedRoute>` in switch if auth required

---

## What is not yet implemented

- `ChatbotWidget` component — floating chat button and window
- `use_api`, `use_auth`, `use_cart` custom hooks
- `button.rs`, `toast.rs`, `modal.rs`, `badge.rs` UI components
- `product_grid.rs`, `attribute_table.rs` product components
- `ProtectedRoute` not yet applied in the switch function (redirects handled inline per page)
