# Gateway — technical documentation

The Go gateway is the single point of entry for all requests from the frontend. No frontend code talks directly to a database or microservice — everything goes through the gateway. This document covers the routes, data models, authentication system, proxy endpoints, and test suite.

---

## Technology

|           |                            |
| --------- | -------------------------- |
| Language  | Go 1.23                    |
| Framework | Gin                        |
| Auth      | golang-jwt/jwt v5          |
| Docs      | Swaggo (OpenAPI / Swagger) |
| Testing   | testify                    |

---

## Running

```bash
cd gateway
go run cmd/main.go
# http://localhost:8000
# Swagger UI: http://localhost:8000/swagger/index.html
```

To regenerate Swagger docs after adding or changing endpoints:

```bash
swag init -g cmd/main.go --output docs
```

---

## Architecture

The gateway uses a local JSON file store for all data — products, users, and orders are each stored in a separate JSON file under `internal/db/`. The `DB` interface is designed to be swappable for a Supabase/PostgreSQL backend without changing any handler code.

```
cmd/
└── main.go                  # Entry point — router setup and server start

internal/
├── db/
│   ├── supabase.go          # DB interface + JSONFileDB implementation
│   ├── user_store.go        # UserStore — bcrypt password hashing
│   ├── order_store.go       # OrderStore — order CRUD
│   ├── products_seed.json   # Product data
│   ├── users.json           # User store (auto-created)
│   └── orders.json          # Order store (auto-created)
├── handlers/
│   ├── products.go          # Product endpoints
│   ├── auth.go              # Auth endpoints
│   ├── orders.go            # Order endpoints
│   └── chat.go              # Chat proxy endpoint
└── middleware/
    └── auth.go              # JWT validation middleware
```

---

## Endpoints

### Health

| Method | Path      | Auth | Description          |
| ------ | --------- | ---- | -------------------- |
| GET    | `/health` | No   | Service health check |

### Auth

| Method | Path                 | Auth | Description                   |
| ------ | -------------------- | ---- | ----------------------------- |
| POST   | `/api/auth/register` | No   | Create account, returns JWT   |
| POST   | `/api/auth/login`    | No   | Login, returns JWT            |
| POST   | `/api/auth/logout`   | No   | Stateless logout              |
| GET    | `/api/auth/me`       | Yes  | Returns current user from JWT |

**Register request:**

```json
{
  "email": "ada@example.com",
  "name": "Ada Lovelace",
  "password": "minlength8"
}
```

**Login request:**

```json
{ "email": "ada@example.com", "password": "minlength8" }
```

**Auth response (register and login):**

```json
{
  "token": "eyJhbGci...",
  "user": {
    "id": "uuid",
    "email": "ada@example.com",
    "name": "Ada Lovelace",
    "created_at": "2024-01-15T10:00:00Z"
  }
}
```

### Products

| Method | Path                   | Auth | Description                         |
| ------ | ---------------------- | ---- | ----------------------------------- |
| GET    | `/api/products`        | No   | Paginated, filterable product list  |
| GET    | `/api/products/groups` | No   | Full product taxonomy               |
| GET    | `/api/products/:id`    | No   | Single product with full attributes |

**GET /api/products query parameters:**

| Parameter | Type   | Description                                                 |
| --------- | ------ | ----------------------------------------------------------- |
| `group`   | string | Filter by group (structural, guidance, payload, propulsion) |
| `type`    | string | Filter by product type                                      |
| `search`  | string | Text search on name and SKU                                 |
| `page`    | int    | Page number (default 1)                                     |
| `limit`   | int    | Results per page (default 20)                               |

**Product list response:**

```json
{
  "data":  [ ...ProductListItem ],
  "total": 42,
  "page":  1,
  "limit": 20
}
```

**Full product (GET /api/products/:id):**

```json
{
  "id": "le-001",
  "name": "Merlin 1D Vacuum",
  "group": "propulsion",
  "product_type": "liquid_engine",
  "price": 4200000,
  "in_stock": true,
  "stock_count": 8,
  "image_url": null,
  "attributes": {
    "max_thrust_kn": 934,
    "specific_impulse_s": 348,
    "burn_time_s": 397,
    "chamber_pressure_bar": 97,
    "weight_kg": 470,
    "gimbal_range_deg": 5.0
  }
}
```

### Orders

All order endpoints require a valid JWT in the `Authorization: Bearer <token>` header.

| Method | Path                     | Auth | Description                              |
| ------ | ------------------------ | ---- | ---------------------------------------- |
| GET    | `/api/orders`            | Yes  | Paginated order history for current user |
| POST   | `/api/orders`            | Yes  | Create a new order                       |
| GET    | `/api/orders/:id`        | Yes  | Single order — ownership enforced        |
| PUT    | `/api/orders/:id/cancel` | Yes  | Cancel order if status allows            |

**Create order request:**

```json
{
  "items": [{ "product_id": "le-001", "quantity": 1 }],
  "shipping_address": {
    "facility_name": "Kennedy Space Center",
    "site_code": "LC-39A",
    "address_line_1": "Space Commerce Way",
    "address_line_2": null,
    "city": "Merritt Island",
    "country": "US",
    "postal_code": "32953"
  },
  "notes": null
}
```

Note: `facility_name` and `site_code` are optional — not all buyers operate from a named launch facility.

**Order statuses and allowed transitions:**

| Status               | Cancellable | Description                    |
| -------------------- | ----------- | ------------------------------ |
| `pending`            | ✅          | Order placed, awaiting payment |
| `payment_processing` | ✅          | Payment in progress            |
| `payment_failed`     | ✅          | Payment failed                 |
| `confirmed`          | ✅          | Payment confirmed              |
| `preparing`          | ❌          | Being packed                   |
| `shipped`            | ❌          | Dispatched                     |
| `in_transit`         | ❌          | In transit                     |
| `delivered`          | ❌          | Delivered                      |
| `cancelled`          | ❌          | Cancelled                      |
| `refund_pending`     | ❌          | Refund under review            |
| `refunded`           | ❌          | Refund processed               |

### Chat proxy

| Method | Path        | Auth | Description                         |
| ------ | ----------- | ---- | ----------------------------------- |
| POST   | `/api/chat` | No   | Proxies chat request to RAG service |

The gateway forwards the request body to the RAG service at `http://localhost:8001/api/chat` and returns the response. A 60-second timeout is applied to accommodate LLM response times. Returns `503` if the RAG service is unreachable.

**Request:**

```json
{
  "query": "What liquid rocket engines do you have?",
  "session_id": "optional-uuid-for-conversation-continuity"
}
```

**Response:**

```json
{
  "answer": "We have the [[Merlin 1D Vacuum|le-001]] at $4.2M...",
  "sources": ["le-001", "le-002"]
}
```

The `[[Name|id]]` markers in the answer are parsed by the frontend into clickable product links.

---

## Authentication

JWTs are signed with `HS256` using a secret from the `JWT_SECRET` environment variable. Tokens expire after 72 hours. The `RequireAuth()` middleware validates the token on every protected route and injects the user ID into the Gin context for use in handlers.

Passwords are hashed with bcrypt (cost factor 10) before storage. Raw passwords are never stored or logged.

---

## Product taxonomy

Four top-level groups, 13 product types:

| Group      | Product types                                              |
| ---------- | ---------------------------------------------------------- |
| Propulsion | liquid_engine, propellant_tank, rocket_nozzle              |
| Structural | rocket_frame, panels_fuselage, control_fins                |
| Guidance   | flight_computer, nav_sensors, control_actuation, telemetry |
| Payload    | nose_cone, crewed_cabin, cargo_module                      |

Each product type has its own typed attribute struct in `internal/models/product.go`. Attributes are stored as JSONB and returned as `map[string]interface{}` in the API response.

---

## Data layer

The `DB` interface in `internal/db/supabase.go` defines all product operations. The current implementation (`JSONFileDB`) reads from `products_seed.json` at startup and holds everything in memory. Swapping to Supabase requires only a new struct implementing the same interface.

`UserStore` and `OrderStore` read from and write to their respective JSON files on every operation — no in-memory caching. This is intentional for simplicity and means data persists across gateway restarts.

---

## Tests

48 tests across three test files:

| File                     | Coverage                                                |
| ------------------------ | ------------------------------------------------------- |
| `tests/products_test.go` | All product endpoints, filtering, pagination            |
| `tests/auth_test.go`     | Register, login, JWT validation, protected route access |
| `tests/orders_test.go`   | Create, list, get, cancel orders, ownership enforcement |

Tests use temporary in-memory stores (`test_helpers.go`) so they never touch the real data files.

```bash
cd gateway
go test ./... -v
```

---

## Environment variables

| Variable       | Required | Default | Description             |
| -------------- | -------- | ------- | ----------------------- |
| `JWT_SECRET`   | Yes      | —       | Secret for signing JWTs |
| `GATEWAY_PORT` | No       | `8000`  | Port to listen on       |
