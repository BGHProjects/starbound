# Starbound 🚀

A full-stack e-commerce platform for purchasing rocket components, built as a portfolio project to demonstrate proficiency across a modern, polyglot microservices architecture.

Starbound allows users to browse a catalog of rocket parts, create accounts, complete purchases, query an AI assistant for product recommendations, and submit photo-verified refund requests — all from a native desktop application or the browser.

This project was pretty ambitious upskilling exercise, where I tried to learn and implement a whole suite of languages, frameworks and paradigms that I hadn't used before (Rust, Yew, Golang, Electron, Playwright, Tesseract, Poppler and many more). Some parts don't work completely (e.g. the Desktop app made with Electron still needs the gateway and its services to be running to be functional), but overall it was a lot of fun and I learned a lot 👍

---

## Overview

| Part of Stack   | Framework / Language           |
| --------------- | ------------------------------ |
| **Frontend**    | Yew (Rust/WASM) + Tailwind CSS |
| **Gateway**     | Go                             |
| **RAG Service** | Python / FastAPI               |
| **CV Service**  | Python / FastAPI               |
| **Desktop**     | Electron                       |
| **Testing**     | Playwright + testify + pytest  |
| **Containers**  | Docker + Docker Compose        |

---

## Functionality

### 🛒 E-commerce catalog

Users can browse a catalog of rocket components organised by four groups — Structural, Guidance, Payload, and Propulsion — each containing multiple product types. Each product has a full spec sheet with typed attributes, stock status, and pricing. Users can filter by group, type, price range, and stock availability, add items to a cart, and complete a multi-step checkout flow receiving an order confirmation on purchase.

### 🔍 Product comparison

Any product can be compared side by side against similar products of the same type. The comparison page uses a vertical card layout styled after SaaS pricing pages, with numeric attributes highlighted green/red for best/worst values across the compared products.

### 👤 Accounts & authentication

Users can register, log in, and manage their account. Authentication is handled via JWT tokens issued by the Go gateway and validated on every protected request. The profile page shows account stats, total spend, and recent order history.

### 📦 Orders

Authenticated users can place orders, view their full order history, inspect individual order details including shipping address and line items, cancel orders that are still in a cancellable status, and download a PDF receipt for any order.

### 🤖 RAG chatbot

A retrieval-augmented generation chatbot powered by LangChain and OpenAI allows users to query the product catalog in natural language. The chatbot is available as a floating widget on key pages and as a full LLM-style chat page. Product mentions in responses are rendered as clickable links. Prompt injection protection is implemented both at the input validation layer and in the system prompt.

### 📄 CV refund service

When a user submits a refund request, they upload their PDF receipt downloaded from the order detail page. A computer vision pipeline (OpenCV + Tesseract OCR + pdf2image) processes the document, extracts the order ID, and cross-references it against the order being refunded — preventing fraudulent submissions of receipts from different orders.

### 🖥️ Desktop application

The entire application is wrapped in an Electron shell, producing a native desktop application for Windows and Linux. Electron bundles a lightweight HTTP server to serve the compiled WASM frontend, allowing Yew's BrowserRouter to work correctly without the file:// URL limitations that affect WASM applications.

---

## Project structure

```
starbound/
├── frontend/          # Yew (Rust 1.81 / WASM) + Tailwind CSS 3
├── gateway/           # Go 1.23 — API gateway, JWT auth, PDF receipts
├── rag-service/       # Python 3.11 / FastAPI / LangChain / ChromaDB
├── cv-service/        # Python 3.11 / FastAPI / OpenCV / Tesseract
├── electron/          # Electron 28 — desktop wrapper
├── tests/             # Playwright (TypeScript) — E2E tests
├── docker/            # Shared Docker configuration
├── docker-compose.yml
├── docs/              # Technical documentation per service
└── .github/workflows/ # GitHub Actions CI
```

---

## Technology versions

| Technology     | Version | Purpose                             |
| -------------- | ------- | ----------------------------------- |
| Rust           | 1.81.0  | Frontend language                   |
| Yew            | 0.21    | Rust/WASM frontend framework        |
| Trunk          | 0.21    | Yew build tool and dev server       |
| Tailwind CSS   | 3.x     | Utility-first CSS framework         |
| Go             | 1.23.1  | API gateway                         |
| Gin            | latest  | Go HTTP router                      |
| golang-jwt     | v5      | JWT authentication                  |
| go-pdf/fpdf    | latest  | PDF receipt generation              |
| Swaggo         | latest  | OpenAPI/Swagger doc generation      |
| Python         | 3.11.5  | RAG and CV services                 |
| FastAPI        | latest  | Python web framework                |
| LangChain      | latest  | RAG chain orchestration             |
| ChromaDB       | latest  | Vector database for embeddings      |
| OpenAI         | latest  | Embeddings + LLM (GPT-4o)           |
| OpenCV         | latest  | Computer vision processing          |
| Tesseract      | latest  | OCR engine                          |
| pdf2image      | latest  | PDF to image conversion             |
| Poppler        | latest  | PDF rendering backend for pdf2image |
| Electron       | 28      | Desktop application wrapper         |
| Playwright     | latest  | End-to-end testing                  |
| Node.js        | 24.14.0 | Electron + tooling runtime          |
| Docker         | 29.2.1  | Containerisation                    |
| Docker Compose | v3.9    | Multi-service orchestration         |

---

## Running locally

Each service runs independently. Open a separate terminal for each.

### Prerequisites

```bash
# Rust WASM target + Trunk
rustup target add wasm32-unknown-unknown
cargo install trunk
cargo install wasm-bindgen-cli

# Tesseract OCR (Windows)
# Download from https://github.com/UB-Mannheim/tesseract/wiki

# Poppler (Windows — required for CV service PDF processing)
# Download from https://github.com/oschwartz10612/poppler-windows/releases
```

### Environment variables

```bash
# gateway/.env
JWT_SECRET=your-secret-here

# rag-service/.env
OPENAI_API_KEY=sk-...

# cv-service/.env
TESSERACT_CMD=C:/Program Files/Tesseract-OCR/tesseract.exe
POPPLER_PATH=C:/Program Files/poppler/Release-x.x.x/poppler-x.x.x/Library/bin
```

### 1 — Go gateway

```bash
cd gateway
go run cmd/main.go
# http://localhost:8000
# Swagger UI: http://localhost:8000/swagger/index.html
# Static assets: http://localhost:8000/static/
```

Note: The gateway serves the `frontend/static/` folder at `/static/`. This is required for the hero video to load on the landing page.

### 2 — RAG service

```bash
cd rag-service
source venv/Scripts/activate   # Windows
uvicorn app.main:app --reload --port 8001
# http://localhost:8001
# Products are ingested into the vector store automatically on first startup
```

### 3 — CV service

```bash
cd cv-service
source venv/Scripts/activate   # Windows
uvicorn app.main:app --reload --port 8002
# http://localhost:8002
```

### 4 — Frontend

```bash
cd frontend
trunk serve
# http://localhost:8080
```

### 5 — Desktop (Electron — development)

```bash
# Requires trunk serve to be running first
cd electron
NODE_ENV=development npm start
```

### 6 — Desktop (Electron — production build)

```bash
# Build the frontend first
cd frontend && trunk build --release

# Then build and run Electron
cd electron
npm run build:win
dist/win-unpacked/Starbound.exe
```

## Running with Docker

Copy the environment template and fill in your credentials:

```bash
cp .env.example .env
```

Then start all services:

```bash
docker-compose up --build
```

| URL                                      | Service     |
| ---------------------------------------- | ----------- |
| http://localhost                         | Frontend    |
| http://localhost:8000                    | Gateway     |
| http://localhost:8000/swagger/index.html | Swagger UI  |
| http://localhost:8001                    | RAG service |
| http://localhost:8002                    | CV service  |

See `docs/docker.md` for full documentation including individual service details, data persistence, and production deployment notes.

---

## Running tests

### Go unit tests

```bash
cd gateway
go test ./... -v
```

### Python unit tests

```bash
cd rag-service && source venv/Scripts/activate && pytest
cd cv-service  && source venv/Scripts/activate && pytest
```

### Playwright end-to-end tests

```bash
cd tests
npx playwright test
npx playwright test --ui
npx playwright show-report
```

---

## API documentation

```
http://localhost:8000/swagger/index.html
```

---

## PDF receipts and refunds

The refund flow requires a PDF receipt generated by the app:

1. Place an order through the checkout flow
2. Go to the order detail page and click "Download receipt"
3. Change the order status to `delivered` in `gateway/internal/db/orders.json`
4. Go to the refund page for that order
5. Upload the downloaded receipt PDF
6. The CV service extracts the order ID and cross-references it against the order being refunded

---

## Other Notes

The Go gateway serves the video used on the landing page, so the backend has to be running in order to see this video on that page

---

## Licence

MIT
