# Starbound 🚀

A full-stack e-commerce platform for purchasing rocket components.

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

Users can browse a catalog of rocket components organised by category (propulsion, airframe, avionics, thermal, guidance). Each product has a detailed spec sheet, stock status, and pricing. Users can add items to a cart and complete a multi-step checkout flow, receiving a PDF receipt on order confirmation.

### 👤 Accounts & authentication

Users can sign up, log in, and manage their account. Authentication is handled via Supabase Auth with JWT tokens validated at the API gateway on every protected request.

### 🤖 RAG chatbot

A retrieval-augmented generation chatbot powered by LangChain and OpenAI allows users to query the product catalog in natural language. Example queries include _"build me a rocket under $1 million"_, _"what components can withstand 400°C"_, or _"what can I add to increase my rocket's top speed"_. The chatbot retrieves answers from internal product documentation stored in a Chroma vector database.

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
│                      # Routes to Supabase (database) and microservices
│                      # JWT authentication middleware
│
├── rag-service/       # Python 3.11 / FastAPI
│                      # RAG chatbot microservice
│                      # LangChain + ChromaDB + OpenAI embeddings
│                      # Ingests product PDF documentation as vector embeddings
│
├── cv-service/        # Python 3.11 / FastAPI
│                      # Computer vision refund processing microservice
│                      # OpenCV + Tesseract OCR + pdf2image
│                      # Validates PDF receipts and determines refund eligibility
│
├── electron/          # Electron 28 / Node 24
│                      # Desktop application wrapper
│                      # Serves the compiled Yew WASM bundle as a native app
│
├── tests/             # Playwright (TypeScript)
│                      # End-to-end test suite covering all user flows
│                      # Runs against both browser and Electron targets
│
├── docker/            # Shared Docker configuration
├── docker-compose.yml # Orchestrates all services for local / CI runs
└── .github/workflows/ # GitHub Actions CI — runs tests on every PR
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

Make sure the following are installed before starting:

- [Rust](https://rustup.rs/) + `wasm32-unknown-unknown` target + Trunk
- [Go 1.23+](https://go.dev/dl/)
- [Python 3.11](https://www.python.org/downloads/)
- [Node.js 20+](https://nodejs.org/)
- [Docker Desktop](https://www.docker.com/products/docker-desktop/) _(optional for local dev)_

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
```

### 2 — RAG service

```bash
cd rag-service
source venv/Scripts/activate   # Windows
# source venv/bin/activate     # Mac / Linux
uvicorn app.main:app --reload --port 8001
# Running at http://localhost:8001
# Health check: http://localhost:8001/health
```

### 3 — CV service

```bash
cd cv-service
source venv/Scripts/activate   # Windows
# source venv/bin/activate     # Mac / Linux
uvicorn app.main:app --reload --port 8002
# Running at http://localhost:8002
# Health check: http://localhost:8002/health
```

### 4 — Frontend

```bash
cd frontend
trunk serve
# Running at http://localhost:8080
```

### 5 — Desktop (Electron)

```bash
# Requires the frontend trunk serve to be running first
cd electron
NODE_ENV=development npm start
```

### Running all services with Docker

```bash
# From the project root (requires Docker Desktop running)
docker-compose up --build
```

---

## Running tests

### Go unit tests

```bash
cd gateway
go test ./...
```

### Python unit tests

```bash
# RAG service
cd rag-service
source venv/Scripts/activate
pytest

# CV service
cd cv-service
source venv/Scripts/activate
pytest
```

### Playwright end-to-end tests

```bash
# Requires all services to be running first
cd tests
npx playwright test

# Run with UI mode (recommended for development)
npx playwright test --ui

# View the last test report
npx playwright show-report
```

---

## Status

| Service     | Status         |
| ----------- | -------------- |
| Go gateway  | 🟡 In progress |
| Frontend    | 🟡 In progress |
| RAG service | 🔴 Not started |
| CV service  | 🔴 Not started |
| Electron    | 🔴 Not started |
| E2E tests   | 🔴 Not started |

---

## Licence

MIT
