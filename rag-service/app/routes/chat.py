from fastapi import APIRouter, HTTPException
from pydantic import BaseModel
from app.rag.chain import run_chain
from app.rag.security import validate_query

router = APIRouter()

class ChatRequest(BaseModel):
    query:      str
    session_id: str | None = None

class ChatResponse(BaseModel):
    answer:  str
    sources: list[str] = []

@router.post("/chat", response_model=ChatResponse)
async def chat(request: ChatRequest):
    query, error = validate_query(request.query)
    if error:
        return ChatResponse(answer=error, sources=[])

    try:
        answer, sources = await run_chain(
            query=query,
            session_id=request.session_id,
        )
        return ChatResponse(answer=answer, sources=sources)
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))