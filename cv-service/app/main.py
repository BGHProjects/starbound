from dotenv import load_dotenv
load_dotenv()

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from app.routes import refund

app = FastAPI(title="Starbound CV Service", version="0.1.0")

app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:8080", "http://localhost:8000"],
    allow_methods=["*"],
    allow_headers=["*"],
)

app.include_router(refund.router, prefix="/api")

@app.get("/health")
def health():
    return {"status": "ok", "service": "starbound-cv"}