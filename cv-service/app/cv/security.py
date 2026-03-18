MAX_FILE_SIZE = 10 * 1024 * 1024  # 10MB
ALLOWED_CONTENT_TYPES = {"application/pdf", "application/octet-stream"}
ALLOWED_EXTENSIONS    = {".pdf"}


def validate_upload(filename: str, content_type: str, size: int) -> str | None:
    """
    Validate an uploaded file before processing.
    Returns an error string if invalid, None if valid.
    """
    import os
    ext = os.path.splitext(filename.lower())[1]

    if ext not in ALLOWED_EXTENSIONS:
        return "Only PDF files are accepted."

    if content_type not in ALLOWED_CONTENT_TYPES:
        return f"Invalid file type: {content_type}. Only PDF files are accepted."

    if size > MAX_FILE_SIZE:
        mb = MAX_FILE_SIZE // (1024 * 1024)
        return f"File too large. Maximum size is {mb}MB."

    return None