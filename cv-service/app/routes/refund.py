import traceback
from fastapi import APIRouter, UploadFile, File, Form, HTTPException
from pydantic import BaseModel
from app.cv.processor import process_receipt
from app.cv.security import validate_upload

router = APIRouter()


class RefundResponse(BaseModel):
    valid:    bool
    order_id: str | None = None
    reason:   str


@router.post("/refund/validate", response_model=RefundResponse)
async def validate_refund(
    file:     UploadFile = File(...),
    order_id: str        = Form(...),
):
    contents = await file.read()

    error = validate_upload(
        filename=file.filename or "",
        content_type=file.content_type or "",
        size=len(contents),
    )
    if error:
        return RefundResponse(valid=False, reason=error)

    try:
        valid, reason, extracted_id = process_receipt(contents, expected_order_id=order_id)
        return RefundResponse(valid=valid, reason=reason, order_id=extracted_id)
    except Exception as e:
        print("=== CV PROCESSING ERROR ===")
        traceback.print_exc()
        raise HTTPException(
            status_code=500,
            detail=f"CV processing failed: {str(e)}"
        )
    
@router.post("/refund/debug")
async def debug_refund(file: UploadFile = File(...)):
    contents = await file.read()
    from app.cv.processor import extract_text_from_pdf
    text = extract_text_from_pdf(contents)
    return {"text": text}