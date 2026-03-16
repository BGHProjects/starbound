from contextlib import asynccontextmanager
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from dotenv import load_dotenv
import os

load_dotenv()

from app.routes import chat

@asynccontextmanager
async def lifespan(app: FastAPI):
    try:
        from app.rag.ingest import maybe_ingest
        maybe_ingest()
    except Exception as e:
        print(f"Warning: could not ingest products on startup: {e}")
    yield

app = FastAPI(
    title="Starbound RAG Service",
    version="0.1.0",
    lifespan=lifespan,
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:8080", "http://localhost:8000"],
    allow_methods=["*"],
    allow_headers=["*"],
)

app.include_router(chat.router, prefix="/api")

@app.get("/health")
def health():
    return {"status": "ok", "service": "starbound-rag"}