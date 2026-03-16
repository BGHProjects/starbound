# RAG Service — technical documentation

The RAG (Retrieval-Augmented Generation) service powers the Starbound AI chatbot. It provides a conversational interface over the product catalog — users can ask natural language questions and receive accurate, grounded answers based on real product data. This document covers the architecture, the RAG pipeline, session management, security measures, and how to run and extend the service.

---

## Technology

|                |                                 |
| -------------- | ------------------------------- |
| Language       | Python 3.11                     |
| Framework      | FastAPI                         |
| LLM            | GPT-4o (OpenAI)                 |
| Embeddings     | text-embedding-3-small (OpenAI) |
| Vector store   | ChromaDB (in-memory)            |
| Orchestration  | LangChain                       |
| Env management | python-dotenv                   |

---

## Running

```bash
cd rag-service
source venv/Scripts/activate   # Windows
# source venv/bin/activate     # Mac / Linux

# Add your OpenAI API key to .env first
echo "OPENAI_API_KEY=sk-..." > .env

uvicorn app.main:app --reload --port 8001
# http://localhost:8001
# Health check: GET http://localhost:8001/health
```

On first startup, the service automatically ingests all products from the gateway's seed file into the in-memory vector store. You will see:

```
Vector store is empty — ingesting products...
Ingested N products into vector store.
```

On subsequent startups in the same process, the check is skipped. Because the store is in-memory, it is rebuilt from the seed file every time the service restarts.

---

## Architecture

```
app/
├── main.py              # FastAPI app — CORS, lifespan hook, router mount
├── routes/
│   └── chat.py          # POST /api/chat — validates input, runs chain
└── rag/
    ├── embeddings.py    # OpenAI embeddings — cached singleton
    ├── vectorstore.py   # ChromaDB in-memory store — singleton
    ├── ingest.py        # Product ingestion — seed file → documents → vectors
    ├── chain.py         # RAG chain — retrieval, prompt, LLM, session history
    └── security.py      # Input validation and prompt injection protection
```

---

## RAG pipeline

### 1 — Ingest

On startup, `ingest.py` reads `gateway/internal/db/products_seed.json` and converts each product into a rich text document:

```
Product: Merlin 1D Vacuum
ID: le-001
Type: liquid_engine
Category: propulsion
Price: $4,200,000.00 USD
In stock: Yes (8 units available)
Specifications:
  - Max Thrust Kn: 934
  - Specific Impulse S: 348
  - Burn Time S: 397
  - Chamber Pressure Bar: 97
  - Weight Kg: 470
  - Gimbal Range Deg: 5.0
```

Each document is embedded using `text-embedding-3-small` and stored in ChromaDB alongside its metadata (`product_id`, `name`, `product_type`, `group`, `price`, `in_stock`).

### 2 — Retrieval

On each query, the query text is embedded and the top 5 most semantically similar product documents are retrieved from ChromaDB using cosine similarity.

### 3 — Augmentation

The 5 retrieved documents are concatenated and injected into the system prompt as the "source of truth" context. This grounds the LLM's response in real catalog data.

### 4 — Generation

GPT-4o receives the system prompt (with context), the session conversation history, and the user query. It generates a response at `temperature=0.3` for consistent, factual answers.

### 5 — Product linking

The LLM is instructed via the system prompt to wrap all product name mentions in `[[Product Name|product_id]]` markers. The frontend `ChatMessageContent` component parses these markers and renders them as clickable links to the product detail page.

Example LLM output:

```
The [[Merlin 1D Vacuum|le-001]] is our most popular engine at $4.2M,
with a max thrust of 934 kN and specific impulse of 348s.
```

The frontend renders "Merlin 1D Vacuum" as an orange underlined link to `/product/le-001`.

---

## Session memory

Conversation history is maintained per `session_id` in an in-memory Python dict. The `session_id` is generated client-side and passed in every request. This allows multi-turn conversations where the LLM can reference earlier messages.

```python
_session_store: dict[str, list] = {}
```

Each session stores a list of `HumanMessage` and `AIMessage` objects from LangChain. These are injected into the prompt via `MessagesPlaceholder` on every request.

**Session limits:** History is trimmed to the last 10 turns (20 messages) per session to prevent unbounded context window growth and runaway API costs.

**Persistence:** Session history is in-memory only — it is lost when the service restarts. This is intentional for simplicity. For production, sessions would be persisted to Redis or a database.

---

## Security

### Input validation (`security.py`)

Every query passes through `validate_query()` before reaching the RAG chain:

**Length limit:** Queries over 1000 characters are rejected with an error message. This prevents excessively long inputs that could inflate embedding costs or attempt to overwhelm the context window.

**Sanitisation:** Control characters (null bytes, non-printable characters) are stripped, and excessive whitespace is collapsed. This prevents certain encoding-based injection techniques.

**Injection detection:** A set of regex patterns detects common prompt injection attempts:

```python
INJECTION_PATTERNS = [
    r"ignore\s+(all\s+)?(previous|prior|above)\s+instructions?",
    r"forget\s+(all\s+)?(previous|prior|above)\s+instructions?",
    r"you\s+are\s+now\s+a",
    r"act\s+as\s+(if\s+you\s+are\s+)?a",
    r"pretend\s+(you\s+are|to\s+be)",
    r"your\s+(new\s+)?(instructions?)\s+(are|is)\s*:",
    r"disregard\s+(all\s+)?(previous|prior|above)",
    r"override\s+(your\s+)?(previous\s+)?(instructions?|prompt|rules?)",
    r"system\s*prompt\s*:",
    r"<\s*system\s*>",
    r"\[\s*system\s*\]",
    r"jailbreak",
    r"do\s+anything\s+now",
    r"dan\s+mode",
]
```

Detected injections return the message: `"I can only help with questions about Starbound rocket components."` — no error details are leaked.

### System prompt hardening (`chain.py`)

The LLM system prompt enforces strict behavioural boundaries regardless of what the user sends:

- **Topic restriction:** The LLM is explicitly instructed to only discuss rocket components and aerospace engineering.
- **Instruction immunity:** The prompt instructs the LLM to never follow instructions embedded in user messages that attempt to change its role or behaviour.
- **Identity protection:** The LLM is instructed to never claim to be a different AI, person, or system.
- **Prompt confidentiality:** The LLM is instructed never to reveal, repeat, or summarise its own system prompt.
- **Consistent fallback:** Any manipulation attempt is met with: `"I can only help with questions about Starbound rocket components."`

### CORS

The service only accepts cross-origin requests from the frontend (`http://localhost:8080`) and the gateway (`http://localhost:8000`). All other origins are blocked.

---

## Endpoints

### Health

```
GET /health
→ { "status": "ok", "service": "starbound-rag" }
```

### Chat

```
POST /api/chat
```

**Request:**

```json
{
  "query": "What liquid rocket engines do you have under $5M?",
  "session_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

`session_id` is optional. If omitted, no conversation history is maintained for that request.

**Response:**

```json
{
  "answer": "We have the [[Merlin 1D Vacuum|le-001]] priced at $4,200,000...",
  "sources": ["le-001", "le-002", "rf-001"]
}
```

`sources` contains the product IDs of the top retrieved documents, regardless of whether the LLM mentioned them in its answer.

---

## Extending the knowledge base

To add more documents to the vector store beyond the product seed file, create additional ingest functions in `ingest.py`. Candidates include:

- Product installation manuals (PDF → text → chunks → embed)
- Engineering specification sheets
- FAQ documents
- Compatibility guides (e.g. which nozzles fit which engines)

Each additional document type would be loaded, chunked, and added to the same ChromaDB collection via `vs.add_documents(documents)`.

---

## Environment variables

| Variable         | Required | Description                              |
| ---------------- | -------- | ---------------------------------------- |
| `OPENAI_API_KEY` | Yes      | OpenAI API key for embeddings and GPT-4o |

---

## Cost considerations

Every request to the chat endpoint makes two OpenAI API calls:

1. **Embedding call** — embeds the user query using `text-embedding-3-small` (~$0.00002 per query)
2. **Completion call** — GPT-4o generates the answer (cost depends on context length, roughly $0.01–0.05 per query depending on how much product context is retrieved)

Session history increases the completion cost per turn as the conversation grows. The 10-turn limit caps this at a predictable maximum per session.
