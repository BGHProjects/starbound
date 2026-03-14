from fastapi import APIRouter, UploadFile, File
from pydantic import BaseModel

router = APIRouter()

class RefundResponse(BaseModel):
    valid: bool
    order_id: str | None = None
    reason: str

@router.post("/refund/validate", response_model=RefundResponse)
async def validate_refund(file: UploadFile = File(...)):
    return RefundResponse(valid=False, reason="CV pipeline not yet initialised")