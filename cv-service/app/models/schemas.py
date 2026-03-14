from pydantic import BaseModel

class RefundResponse(BaseModel):
    valid: bool
    order_id: str | None = None
    reason: str
    extracted_data: dict = {}