from fastapi import FastAPI
from app.routes import chat

app = FastAPI(title="Starbound RAG Service", version="0.1.0")
app.include_router(chat.router, prefix="/api")

@app.get("/health")
def health():
    return {"status": "ok", "service": "starbound-rag"}