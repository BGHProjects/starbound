import traceback
from fastapi import APIRouter, UploadFile, File, HTTPException
from pydantic import BaseModel
from app.cv.processor import process_receipt
from app.cv.security import validate_upload

router = APIRouter()


class RefundResponse(BaseModel):
    valid:    bool
    order_id: str | None = None
    reason:   str


@router.post("/refund/validate", response_model=RefundResponse)
async def validate_refund(file: UploadFile = File(...)):
    contents = await file.read()

    error = validate_upload(
        filename=file.filename or "",
        content_type=file.content_type or "",
        size=len(contents),
    )
    if error:
        return RefundResponse(valid=False, reason=error)

    try:
        valid, reason, order_id = process_receipt(contents)
        return RefundResponse(valid=valid, reason=reason, order_id=order_id)
    except Exception as e:
        print("=== CV PROCESSING ERROR ===")
        traceback.print_exc()
        raise HTTPException(
            status_code=500,
            detail=f"CV processing failed: {str(e)}"
        )