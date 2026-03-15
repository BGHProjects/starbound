# Starbound 🚀

A full-stack e-commerce platform for purchasing rocket components, built as a portfolio project to demonstrate proficiency across a modern, polyglot microservices architecture.

Starbound allows users to browse a catalog of rocket parts, create accounts, complete purchases, query an AI assistant for product recommendations, and submit photo-verified refund requests — all from a native desktop application or the browser.

---

## Overview

|                 |                                |
| --------------- | ------------------------------ |
| **Frontend**    | Yew (Rust/WASM) + Tailwind CSS |
| **Gateway**     | Go                             |
| **RAG Service** | Python / FastAPI               |
| **CV Service**  | Python / FastAPI               |
| **Desktop**     | Electron                       |
| **Database**    | Supabase (PostgreSQL)          |
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

Authenticated users can place orders, view their full order history, inspect individual order details including shipping address and line items, and cancel orders that are still in a cancellable status.

### 🤖 RAG chatbot

A retrieval-augmented generation chatbot powered by LangChain and OpenAI allows users to query the product catalog in natural language. Example queries include "build me a rocket under $1 million", "what components can withstand 400°C", or "what can I add to increase my rocket's top speed". The chatbot retrieves answers from internal product documentation stored in a Chroma vector database.

### 📄 CV refund service

When a user submits a refund request, they upload their PDF receipt. A computer vision pipeline (OpenCV + Tesseract OCR) processes the document, extracts the relevant order information, and determines whether the refund is valid — returning a decision to the user automatically.

### 🖥️ Desktop application

The entire application is wrapped in an Electron shell, producing a native desktop application for Windows and Linux from the same Yew/WASM codebase with no code changes required.

---

## Project structure

```
starbound/
├── frontend/          # Yew (Rust 1.81 / WASM) + Tailwind CSS 3
│                      # Client-side SPA compiled to WebAssembly
│                      # Trunk used as the build tool
│
├── gateway/           # Go 1.23
│                      # API gateway — handles all requests from the frontend
│                      # JWT authentication middleware
│                      # Local JSON file stores (swappable for Supabase)
│
├── rag-service/       # Python 3.11 / FastAPI
│                      # RAG chatbot microservice
│                      # LangChain + ChromaDB + OpenAI embeddings
│
├── cv-service/        # Python 3.11 / FastAPI
│                      # Computer vision refund processing microservice
│                      # OpenCV + Tesseract OCR + pdf2image
│
├── electron/          # Electron 28 / Node 24
│                      # Desktop application wrapper
│
├── tests/             # Playwright (TypeScript)
│                      # End-to-end test suite
│
├── docker/            # Shared Docker configuration
├── docker-compose.yml # Orchestrates all services
└── .github/workflows/ # GitHub Actions CI
```

---

## Technology versions

| Technology     | Version | Purpose                        |
| -------------- | ------- | ------------------------------ |
| Rust           | 1.81.0  | Frontend language              |
| Yew            | 0.21    | Rust/WASM frontend framework   |
| Trunk          | 0.21    | Yew build tool and dev server  |
| Tailwind CSS   | 3.x     | Utility-first CSS framework    |
| Go             | 1.23.1  | API gateway                    |
| Gin            | latest  | Go HTTP router                 |
| golang-jwt     | v5      | JWT authentication             |
| Swaggo         | latest  | OpenAPI/Swagger doc generation |
| Python         | 3.11.5  | RAG and CV services            |
| FastAPI        | latest  | Python web framework           |
| LangChain      | latest  | RAG chain orchestration        |
| ChromaDB       | latest  | Vector database for embeddings |
| OpenAI         | latest  | Embeddings + LLM (GPT-4o)      |
| OpenCV         | latest  | Computer vision processing     |
| Tesseract      | latest  | OCR engine                     |
| Supabase       | latest  | PostgreSQL database + auth     |
| Electron       | 28      | Desktop application wrapper    |
| Playwright     | latest  | End-to-end testing             |
| Node.js        | 24.14.0 | Electron + tooling runtime     |
| Docker         | 29.2.1  | Containerisation               |
| Docker Compose | v3.9    | Multi-service orchestration    |

---

## Running locally

Each service runs independently. Open a separate terminal for each.

### Prerequisites

```bash
# Rust WASM target + Trunk (one-time setup)
rustup target add wasm32-unknown-unknown
cargo install trunk
cargo install wasm-bindgen-cli
```

### Environment variables

```bash
cp .env.example .env
# Fill in your Supabase and OpenAI credentials in .env
```

### 1 — Go gateway

```bash
cd gateway
go run cmd/main.go
# Running at http://localhost:8000
# Health check: http://localhost:8000/health
# Swagger UI:   http://localhost:8000/swagger/index.html
```

### 2 — RAG service

```bash
cd rag-service
source venv/Scripts/activate   # Windows
# source venv/bin/activate     # Mac / Linux
uvicorn app.main:app --reload --port 8001
# Running at http://localhost:8001
```

### 3 — CV service

```bash
cd cv-service
source venv/Scripts/activate   # Windows
# source venv/bin/activate     # Mac / Linux
uvicorn app.main:app --reload --port 8002
# Running at http://localhost:8002
```

### 4 — Frontend

```bash
cd frontend
trunk serve
# Running at http://localhost:8080
```

### 5 — Desktop (Electron)

```bash
# Requires trunk serve to be running first
cd electron
NODE_ENV=development npm start
```

### Running all services with Docker

```bash
docker-compose up --build
```

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
npx playwright test --ui       # UI mode
npx playwright show-report     # View last report
```

---

## API documentation

The Go gateway serves a live interactive Swagger UI when running locally:

```
http://localhost:8000/swagger/index.html
```

---

## Status

| Area                          | Status         |
| ----------------------------- | -------------- |
| Go gateway — products         | ✅ Complete    |
| Go gateway — auth             | ✅ Complete    |
| Go gateway — orders           | ✅ Complete    |
| Go gateway — chat proxy       | 🔴 Not started |
| Go gateway — refund proxy     | 🔴 Not started |
| Go gateway — Swagger docs     | ✅ Complete    |
| Frontend — landing page       | ✅ Complete    |
| Frontend — catalog            | ✅ Complete    |
| Frontend — product detail     | ✅ Complete    |
| Frontend — compare            | ✅ Complete    |
| Frontend — cart               | ✅ Complete    |
| Frontend — checkout           | ✅ Complete    |
| Frontend — order confirmation | ✅ Complete    |
| Frontend — orders list        | ✅ Complete    |
| Frontend — order detail       | ✅ Complete    |
| Frontend — refund             | ✅ Complete    |
| Frontend — login / register   | ✅ Complete    |
| Frontend — profile            | ✅ Complete    |
| Frontend — search overlay     | 🔴 Not started |
| Frontend — chatbot widget     | 🔴 Not started |
| RAG service                   | 🔴 Not started |
| CV service                    | 🔴 Not started |
| Electron wrapper              | 🔴 Not started |
| E2E tests                     | 🔴 Not started |

---

## Licence

MIT
