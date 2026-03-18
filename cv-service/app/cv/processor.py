import os
import re
import pytesseract
from pdf2image import convert_from_bytes
from PIL import Image

# Set Tesseract path from environment — defaults to Linux path for Docker
tesseract_cmd = os.getenv("TESSERACT_CMD", "/usr/bin/tesseract")
pytesseract.pytesseract.tesseract_cmd = tesseract_cmd

# Markers we expect to find in a legitimate Starbound receipt
STARBOUND_MARKERS = [
    "starbound",
    "order",
]

# Known product types from the catalog
KNOWN_PRODUCT_TYPES = [
    "liquid engine",
    "liquid_engine",
    "propellant tank",
    "propellant_tank",
    "rocket nozzle",
    "rocket_nozzle",
    "rocket frame",
    "rocket_frame",
    "panels fuselage",
    "panels_fuselage",
    "control fins",
    "control_fins",
    "flight computer",
    "flight_computer",
    "nav sensors",
    "nav_sensors",
    "control actuation",
    "control_actuation",
    "telemetry",
    "nose cone",
    "nose_cone",
    "crewed cabin",
    "crewed_cabin",
    "cargo module",
    "cargo_module",
]

# Max PDF size — 10MB
MAX_FILE_SIZE = 10 * 1024 * 1024


def extract_text_from_pdf(pdf_bytes: bytes) -> str:
    """
    Convert PDF pages to images and run OCR on each page.
    Returns concatenated text from all pages.
    """

    poppler_path = os.getenv("POPPLER_PATH", None)

    images = convert_from_bytes(pdf_bytes, dpi=200, poppler_path=poppler_path)
    pages  = []

    for image in images:
        # Convert to grayscale — improves OCR accuracy
        gray = image.convert("L")

        # Run OCR
        text = pytesseract.image_to_string(gray, config="--psm 6")
        pages.append(text)

    return "\n\n".join(pages)


def extract_order_id(text: str) -> str | None:
    """
    Try to extract an order ID from the receipt text.
    Order IDs are UUIDs — 8-4-4-4-12 hex format.
    Handles common OCR misreads like © -> c, o -> 0, O -> 0.
    """
    # Normalise common OCR substitutions before matching
    cleaned = text
    cleaned = cleaned.replace("©", "c")
    cleaned = cleaned.replace("\u00a9", "c")
    pattern = r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{1,4}\s?[0-9a-f]{1,4}-[0-9a-f]{12}"
    match   = re.search(pattern, cleaned, re.IGNORECASE)
    if not match:
        return None
    # Remove any spaces OCR inserted within segments
    return re.sub(r"\s", "", match.group(0)).lower()


def extract_total(text: str) -> float | None:
    """
    Try to extract a dollar total from the receipt text.
    Handles formats like $1,234,567.00 / $1.5M / $40K
    """
    # Full dollar amount
    match = re.search(r"\$[\d,]+\.?\d*", text)
    if match:
        raw = match.group(0).replace("$", "").replace(",", "")
        try:
            return float(raw)
        except ValueError:
            pass
    return None


def validate_receipt(text: str, expected_order_id: str | None = None) -> tuple[bool, str]:
    """
    Apply validation rules to extracted receipt text.
    Returns (valid, reason).
    """
    text_lower = text.lower()

    # Rule 1 — must contain Starbound branding and order reference
    missing_markers = [m for m in STARBOUND_MARKERS if m not in text_lower]
    if missing_markers:
        return False, (
            "This does not appear to be a valid Starbound receipt. "
            "Please upload the PDF receipt generated after placing your order."
        )

    # Rule 2 — must contain at least one recognisable product type
    has_product = any(p in text_lower for p in KNOWN_PRODUCT_TYPES)
    if not has_product:
        return False, (
            "No recognisable Starbound products were found in this receipt. "
            "Please ensure you are uploading the correct document."
        )

    # Rule 3 — must contain an order ID
    order_id = extract_order_id(text)
    if not order_id:
        return False, (
            "Could not find a valid order ID in this receipt. "
            "Please ensure the document is complete and not cropped."
        )

    # Rule 4 — must contain a price
    total = extract_total(text)
    if not total:
        return False, (
            "Could not find a valid total amount in this receipt. "
            "Please ensure the document is complete."
        )

    # Rule 5 — extracted order ID must match the order being refunded
    if expected_order_id and order_id:
        if order_id.lower() != expected_order_id.lower():
            return False, (
                "The order ID on this receipt does not match the order you are requesting a refund for. "
                "Please upload the correct receipt."
            )
    return True, (
        f"Receipt verified successfully. "
        f"Order {order_id} for ${total:,.2f} has been approved for refund. "
        f"Your refund will be processed within 5-7 business days."
    )


def process_receipt(pdf_bytes: bytes, expected_order_id: str | None = None) -> tuple[bool, str, str | None]:
    """
    Full pipeline — extract text, validate, return result.
    Returns (valid, reason, order_id).
    """
    text = extract_text_from_pdf(pdf_bytes)

    valid, reason = validate_receipt(text, expected_order_id=expected_order_id)
    order_id = extract_order_id(text) if valid else None
    return valid, reason, order_id