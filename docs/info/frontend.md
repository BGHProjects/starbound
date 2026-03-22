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
│   │   ├── cart.rs                  # CartContext — items, quantities, totals
│   │   └── chat.rs                  # ChatContext — shared chat history and loading state
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
│   │   ├── chat.rs                  # /chat — full LLM-style chat page
│   │   └── not_found.rs             # /404
│   │
│   ├── components/
│   │   ├── layout/
│   │   │   ├── navbar.rs            # Top navigation bar
│   │   │   ├── chatbot_widget.rs    # Floating chat button and compact window
│   │   │   └── protected_route.rs  # Redirects to /login if not authenticated
│   │   ├── ui/
│   │   │   ├── spinner.rs           # Loading spinner — sm/md/lg sizes
│   │   │   ├── tooltip.rs           # Hover tooltip with optional external link
│   │   │   ├── chat_message.rs      # Parses [[Name|id]] markers into product links
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

Note: `search_overlay.rs` was removed — search functionality is handled entirely by the catalog page search bar.

---

## How the app boots

```rust
fn main() {
    yew::Renderer::<App>::new().render();
}
```

The `App` component wraps everything in the router, auth context, cart context, and chat context:

```rust
#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <AuthProvider>
                <CartProvider>
                    <ChatProvider>
                        <div class="min-h-screen bg-navy">
                            <components::layout::navbar::Navbar />
                            <main>
                                <Switch<Route> render={switch} />
                            </main>
                        </div>
                    </ChatProvider>
                </CartProvider>
            </AuthProvider>
        </BrowserRouter>
    }
}
```

Provider order matters — `AuthProvider` wraps `CartProvider` which wraps `ChatProvider`. `ChatProvider` is at the app level so the chat history is shared between the floating widget and the full chat page.

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
    #[at("/chat")]                    Chat,
    #[not_found] #[at("/404")]        NotFound,
}
```

`CatalogFiltered` vs `Catalog` — when a user clicks a category on the landing page they are routed to `/catalog/propulsion` etc., which pre-fills the group filter. The unfiltered `/catalog` route passes `initial_group: None`.

### Protected routes

The following pages require authentication and redirect to `/login` if the user is not logged in:

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

### CartContext (`src/context/cart.rs`)

In-memory cart — resets on page refresh. No localStorage persistence.

```rust
pub struct CartState {
    pub items: Vec<CartItem>,
}

pub enum CartAction {
    AddItem(ProductListItem),
    RemoveItem(String),
    UpdateQuantity(String, i32),
    Clear,
}
```

Computed helpers: `cart.total()`, `cart.item_count()`, `cart.contains(id)`.

### ChatContext (`src/context/chat.rs`)

Shared between the floating `ChatbotWidget` and the full `/chat` page. Because it lives at the app level, conversation history carries over seamlessly when a user clicks "Expand" in the widget to open the full chat page.

```rust
pub struct ChatState {
    pub messages:   Vec<ChatMessage>,
    pub is_loading: bool,
    pub session_id: Option<String>,
}

pub enum ChatAction {
    AddMessage(ChatMessage),
    SetLoading(bool),
    SetSessionId(String),
    Clear,
}
```

The `session_id` is passed to the RAG service on every request so the backend can maintain per-session conversation history.

---

## API service layer

All HTTP calls go through `src/services/`. No component ever calls `gloo_net` directly.

### ApiClient

```rust
ApiClient::get::<T>("/products?group=propulsion", None).await
ApiClient::get::<T>("/orders/abc", Some(&token)).await
ApiClient::post::<Body, T>("/orders", &req, Some(&token)).await
ApiClient::post::<Body, T>("/chat", &req, None).await
ApiClient::put::<T>("/orders/abc/cancel", Some(&token)).await
```

All return `Result<T, String>`. Non-2xx responses return `Err("HTTP 404: ...")`. All requests go to the Go gateway at `http://localhost:8000/api` — the frontend never talks directly to the RAG or CV microservices.

---

## Pages

### Landing (`/`)

Fetches products from the gateway on mount — four separate requests, one per group, plus a featured request. `ChatbotWidget` rendered at the bottom of the page.

### Catalog (`/catalog` and `/catalog/:group`)

Accepts an `initial_group: Option<String>` prop from the router. Price sliders use a two-level debounce pattern with `Rc<RefCell<Option<Timeout>>>` — each slider has its own debounce handle so pending timeouts are cancelled before scheduling a new one. Grid transitions use a three-phase animation state (`GridPhase::Idle`, `Exiting`, `Entering`). `ChatbotWidget` rendered at the bottom.

### Product detail (`/product/:id`)

Fetches full product including attributes. Attributes rendered as key-value list with label formatting. `ChatbotWidget` rendered at the bottom.

### Compare (`/product/:id/compare`)

Fetches current product then calls `get_similar` for up to two products of the same type. Fixed-width `w-80` vertical cards. Fixed `h-12` name container prevents card misalignment from long names. Numeric attributes highlighted green/red for best/worst values. Horizontal snap scroll on mobile with nav buttons. `ChatbotWidget` rendered at the bottom.

### Cart (`/cart`)

Reads from `CartContext` — no API call. Auth-aware checkout CTA. `ChatbotWidget` rendered at the bottom.

### Checkout (`/checkout`)

Redirects to `/login` if not authenticated, `/cart` if cart empty. Facility name and site code are optional fields with `Tooltip` components. On submit calls `OrderService::create`, clears cart, navigates to order confirmation.

### Refund (`/refund/:order_id`)

Four-stage UI via `RefundStage` enum: `Upload`, `Processing`, `Success { valid, reason }`, `Error(String)`. POSTs PDF as `multipart/form-data` to the gateway which proxies to the CV service.

### Chat (`/chat`)

Full LLM-style chat interface. Fixed layout — scrollable message history with pinned input at the bottom. Height set to `calc(100vh - 64px)` to fill below the navbar. Suggested prompts shown when empty. Typing indicator during loading. Enter to send, Shift+Enter for new line. Calls `POST /api/chat` on the gateway which proxies to the RAG service. Product mentions in responses rendered as clickable links via `ChatMessageContent`.

### Orders and Order detail

Order detail shows cancel button for cancellable statuses. Shows "Request refund" for `shipped` or `delivered` orders. Status badges colour-coded via shared `status_color()` helper used consistently across all pages.

---

## Components

### Navbar

Responsive. Logo text hidden on very small screens. Cart icon always visible, text on `sm+`. Auth state: logged out shows Sign in + Sign up buttons; logged in shows avatar circle with initial and first name (truncated, links to profile).

### ChatbotWidget

Floating bottom-right on Landing, Catalog, Product detail, Compare, and Cart pages. Toggle button opens a compact chat window. Shows last 6 messages from `ChatContext`. Suggested prompts shown when no messages. "Expand" button navigates to `/chat` — conversation history carries over because both the widget and the full page share the same `ChatContext`. Calls `POST /api/chat` through the gateway.

### ProductCard

Entire card wrapped in a `Link` to the product detail page. Add to cart button uses `e.stop_propagation()` to prevent navigation. Price formatted as `$40K`, `$1.5M` etc.

### Spinner

Three sizes: `SpinnerSize::Sm`, `Md`, `Lg`.

### Tooltip

Hover-triggered bubble with optional external link. Hover zone covers both the icon and bubble so moving the cursor to click the link doesn't dismiss it. Font set inline on the bubble to prevent inheriting `font-orbitron` from parent label elements.

### ChatMessageContent

Parses `[[Product Name|product_id]]` markers in assistant messages and renders them as clickable orange links navigating to `/product/:id`. Plain text segments rendered with `white-space: pre-wrap`. Used in both the full chat page and the floating widget.

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

| Class                    | Effect                                                         |
| ------------------------ | -------------------------------------------------------------- |
| `animate-fade-up`        | Fade in + rise 20px — fill mode `both` for staggered entrances |
| `animate-fade-in`        | Opacity fade                                                   |
| `animate-slide-in-right` | Slide from right                                               |
| `animate-slide-in-left`  | Slide from left                                                |
| `animate-scale-in`       | Scale from 95%                                                 |
| `animate-pulse-glow`     | Orange glow pulse — used on chat toggle button                 |
| `animate-float`          | Gentle bob                                                     |
| `animate-shimmer`        | Skeleton shimmer                                               |

---

## Build and development

```bash
cd frontend
trunk serve            # dev server at http://localhost:8080
trunk build --release  # production build → frontend/dist/
```

### Adding a new page

1. Create `src/pages/your_page.rs`
2. Add `#[function_component]`
3. Add variant to `Route` in `src/route.rs`
4. Add match arm to `switch` in `src/main.rs`
5. Add module to `src/pages/mod.rs`
6. Wrap in `<ProtectedRoute>` in switch if auth required

---

## What is not yet implemented

- `use_api`, `use_auth`, `use_cart` custom hooks
- `button.rs`, `toast.rs`, `modal.rs`, `badge.rs` UI stubs
- `product_grid.rs`, `attribute_table.rs` product component stubs
- `ProtectedRoute` not yet applied in the switch function — auth redirects handled inline per page
