# Docker — technical documentation

The project includes a full Docker Compose setup that runs all services in isolated containers. This document covers each Dockerfile, how they connect, and how to run the full stack.

---

## Overview

| Service     | Dockerfile                   | Port | Base image         |
| ----------- | ---------------------------- | ---- | ------------------ |
| Frontend    | `docker/frontend.Dockerfile` | 80   | nginx:alpine       |
| Gateway     | `docker/gateway.Dockerfile`  | 8000 | golang:1.23-alpine |
| RAG service | `docker/rag.Dockerfile`      | 8001 | python:3.11-slim   |
| CV service  | `docker/cv.Dockerfile`       | 8002 | python:3.11-slim   |

---

## Prerequisites

- Docker Desktop installed and running
- A `.env` file at the project root (copy from `.env.example`):

```bash
cp .env.example .env
# Fill in JWT_SECRET and OPENAI_API_KEY
```

---

## Running the full stack

```bash
docker-compose up --build
```

This builds all images and starts all services. On first run, the frontend image will take several minutes due to Rust/WASM compilation.

Once running:

- Frontend: http://localhost
- Gateway: http://localhost:8000
- Swagger UI: http://localhost:8000/swagger/index.html
- RAG service: http://localhost:8001
- CV service: http://localhost:8002

To run in the background:

```bash
docker-compose up --build -d
```

To stop:

```bash
docker-compose down
```

To stop and remove all data volumes:

```bash
docker-compose down -v
```

---

## Individual Dockerfiles

### Frontend (`docker/frontend.Dockerfile`)

Multi-stage build:

**Stage 1 — Builder**

- Base: `rust:1.81`
- Installs the `wasm32-unknown-unknown` Rust target
- Installs `trunk` and `wasm-bindgen-cli`
- Installs Node.js dependencies (Tailwind CLI)
- Runs `trunk build --release` with `public_url = "./"` for relative asset paths
- Output: compiled WASM bundle in `frontend/dist/`

**Stage 2 — Nginx**

- Base: `nginx:alpine`
- Copies the compiled `dist/` folder into the nginx web root
- Uses a custom `nginx.conf` that handles SPA routing

The nginx config is critical for the Yew frontend — without it, refreshing the page on any route other than `/` would return a 404 from nginx. The `try_files` directive serves `index.html` for any path that doesn't match a real file, letting Yew's router handle it client-side:

```nginx
location / {
    try_files $uri $uri/ /index.html;
}
```

### Gateway (`docker/gateway.Dockerfile`)

Multi-stage build:

**Stage 1 — Builder**

- Base: `golang:1.23-alpine`
- Downloads Go modules
- Compiles a single static binary: `go build -o gateway ./cmd/main.go`

**Stage 2 — Runtime**

- Base: `alpine:latest`
- Copies only the compiled binary and the product seed JSON
- Minimal image — no Go toolchain in production

The `gateway-data` Docker volume mounts to `/app/internal/db/` and persists `users.json` and `orders.json` across container restarts. Without this volume, all user accounts and orders would be lost every time the container restarts.

### RAG service (`docker/rag.Dockerfile`)

- Base: `python:3.11-slim`
- Installs Python dependencies from `requirements.txt`
- Copies the product seed JSON from the gateway directory so the ingest pipeline can find it at startup
- On startup, automatically ingests products into the in-memory ChromaDB vector store if empty

Note: The `OPENAI_API_KEY` environment variable must be set in `.env` — the service will start but all chat requests will fail without it.

### CV service (`docker/cv.Dockerfile`)

- Base: `python:3.11-slim`
- Installs system dependencies via `apt-get`:
  - `tesseract-ocr` — OCR engine (replaces the manual Windows installation)
  - `poppler-utils` — PDF rendering backend (replaces the manual Windows installation)
  - `libgl1` — required by OpenCV
- Installs Python dependencies from `requirements.txt`

The key advantage of Docker for the CV service is that Tesseract and Poppler install automatically — no manual downloads or PATH configuration needed. The `TESSERACT_CMD` and `POPPLER_PATH` environment variables default to the correct Linux paths (`/usr/bin/tesseract` and system PATH respectively) so no `.env` is needed for these in Docker.

---

## Service dependencies and healthchecks

Each service has a healthcheck that polls its `/health` endpoint. The `depends_on` configuration uses `condition: service_healthy` to ensure services start in the correct order:

```
frontend → waits for → gateway
rag-service → waits for → gateway
cv-service → starts independently
```

The gateway must be healthy before the frontend or RAG service start, because:

- The frontend makes immediate API calls to the gateway on load
- The RAG service reads the product seed file which lives in the gateway directory

---

## Data persistence

User accounts and orders are stored in JSON files inside the gateway container. The `gateway-data` volume persists these files:

```yaml
volumes:
  gateway-data:

services:
  gateway:
    volumes:
      - gateway-data:/app/internal/db
```

Without this volume, all data is lost when the container stops. The product seed file (`products_seed.json`) is baked into the image at build time and is not affected by the volume.

---

## Environment variables

| Variable         | Service     | Required | Description                                        |
| ---------------- | ----------- | -------- | -------------------------------------------------- |
| `JWT_SECRET`     | gateway     | Yes      | Secret for signing JWTs — use a long random string |
| `OPENAI_API_KEY` | rag-service | Yes      | OpenAI API key for embeddings and GPT-4o           |

All other environment variables (Tesseract path, Poppler path, ports) are configured automatically in the Docker environment and do not need to be set manually.

---

## Building individual services

To rebuild a single service without restarting everything:

```bash
docker-compose build gateway
docker-compose up -d --no-deps gateway
```

To view logs for a specific service:

```bash
docker-compose logs -f gateway
docker-compose logs -f rag-service
```

To open a shell inside a running container:

```bash
docker-compose exec gateway sh
docker-compose exec cv-service bash
```

---

## Notes for production deployment

The current Docker setup is designed for local development and portfolio demonstration. For a production deployment, the following would need to be addressed:

- **Secrets management** — `JWT_SECRET` and `OPENAI_API_KEY` should be injected via a secrets manager (AWS Secrets Manager, HashiCorp Vault) rather than a `.env` file
- **Database** — the JSON file store should be replaced with the Supabase/PostgreSQL backend. The `DB` interface in the gateway is designed for this swap
- **HTTPS** — an nginx reverse proxy with Let's Encrypt certificates should sit in front of all services
- **CORS** — the gateway's wildcard `*` CORS policy should be restricted to the production domain
- **RAG persistence** — ChromaDB should be configured with a persistent file store or replaced with a managed vector database
- **Container registry** — images should be pushed to a registry (ECR, Docker Hub) and pulled by the deployment target rather than built on the server
