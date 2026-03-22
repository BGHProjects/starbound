# CV Service — technical documentation

The CV (Computer Vision) service processes PDF receipt uploads to validate refund requests. When a user submits a refund, they upload their Starbound order receipt as a PDF. The service extracts text from the document using OCR and applies validation rules to determine whether the refund is legitimate.

---

## Technology

|                  |                         |
| ---------------- | ----------------------- |
| Language         | Python 3.11             |
| Framework        | FastAPI                 |
| PDF processing   | pdf2image + Poppler     |
| OCR              | Tesseract + pytesseract |
| Image processing | OpenCV + Pillow         |

---

## System dependencies

Two system-level dependencies must be installed separately from the Python packages:

**Tesseract OCR**

- Windows: Download from https://github.com/UB-Mannheim/tesseract/wiki
- Linux/Docker: `apt-get install tesseract-ocr`
- Set path via `TESSERACT_CMD` environment variable

**Poppler**

- Windows: Download from https://github.com/oschwartz10612/poppler-windows/releases
- Linux/Docker: `apt-get install poppler-utils`
- Set path via `POPPLER_PATH` environment variable

---

## Running

```bash
cd cv-service
source venv/Scripts/activate   # Windows
# source venv/bin/activate     # Mac / Linux

uvicorn app.main:app --reload --port 8002
# http://localhost:8002
# Health check: GET http://localhost:8002/health
```

### Environment variables

```bash
# cv-service/.env
TESSERACT_CMD=C:/Program Files/Tesseract-OCR/tesseract.exe
POPPLER_PATH=C:/Program Files/poppler/Release-25.12.0-0/poppler-25.12.0/Library/bin
```

On Linux/Docker these default to `/usr/bin/tesseract` and `None` (system PATH) respectively so no `.env` is needed in those environments.

---

## Architecture

```
app/
├── main.py              # FastAPI app — CORS, dotenv, router mount
├── routes/
│   └── refund.py        # POST /api/refund/validate
└── cv/
    ├── processor.py     # Full CV pipeline — PDF → images → OCR → validation
    └── security.py      # Upload validation — size, extension, content type
```

---

## CV pipeline

### 1 — Upload validation (`security.py`)

Before any processing begins, the uploaded file is validated:

- **Extension:** Must be `.pdf`
- **Content type:** Must be `application/pdf` or `application/octet-stream`
- **Size:** Maximum 10MB

Invalid files are rejected immediately with a descriptive error message — no OCR processing occurs.

### 2 — PDF to images (`processor.py`)

The PDF bytes are passed to `pdf2image.convert_from_bytes()` which uses Poppler's `pdftoppm` tool to render each page as a PIL image at 200 DPI. Higher DPI improves OCR accuracy at the cost of memory and processing time.

### 3 — OCR (`processor.py`)

Each page image is converted to grayscale (improving Tesseract accuracy) and passed to `pytesseract.image_to_string()` with `--psm 6` (assumes uniform block of text). The text from all pages is concatenated.

### 4 — Validation rules

The extracted text is checked against four rules in order:

**Rule 1 — Starbound markers**
The text must contain both "starbound" and "order" (case insensitive). Failure indicates the document is not a Starbound receipt.

**Rule 2 — Product type**
The text must contain at least one recognisable Starbound product type: liquid engine, propellant tank, rocket nozzle, rocket frame, panels fuselage, control fins, flight computer, nav sensors, control actuation, telemetry, nose cone, crewed cabin, or cargo module.

**Rule 3 — Order ID**
The text must contain a UUID matching the pattern `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`. The regex is space-tolerant within segments to account for OCR misreads where a space is inserted mid-segment.

**Rule 4 — Price**
The text must contain a dollar amount (e.g. `$4,200,000.00`).

If all four rules pass, the refund is approved with a success message including the extracted order ID and total. If any rule fails, the refund is declined with a specific reason explaining what was missing.

### 5 — Response

```json
{
  "valid": true,
  "order_id": "550e8400-e29b-41d4-a716-446655440000",
  "reason": "Receipt verified successfully. Order 550e8400... for $4,200,000.00 has been approved for refund. Your refund will be processed within 5-7 business days."
}
```

Or on failure:

```json
{
  "valid": false,
  "order_id": null,
  "reason": "Could not find a valid order ID in this receipt. Please ensure the document is complete and not cropped."
}
```

---

## Gateway integration

The CV service is not called directly by the frontend. All requests are proxied through the Go gateway at `POST /api/refund/validate`. The gateway forwards the multipart form upload to `http://localhost:8002/api/refund/validate` and returns the response. This keeps the microservice internal and ensures all traffic flows through the single gateway entry point.

---

## CORS

The service only accepts cross-origin requests from the frontend (`http://localhost:8080`) and the gateway (`http://localhost:8000`).

---

## Limitations and future improvements

- **OCR accuracy:** Tesseract performs best on clean, high-contrast PDFs. Scanned documents, handwriting, or stylised fonts may produce inaccurate text extraction. In production, a cloud OCR service (Google Document AI, AWS Textract) would provide significantly better accuracy.
- **Receipt forgery:** The current validation checks for marker presence only — it does not cryptographically verify that the receipt was generated by Starbound. In production, receipts would be digitally signed at generation time and the signature verified during refund processing.
- **Order ID cross-reference:** The service currently extracts the order ID from the receipt but does not verify it against the order database. A production implementation would call the gateway to confirm the order exists, belongs to the requesting user, and is in a refundable status.
