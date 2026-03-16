import re

# Patterns that indicate prompt injection attempts
INJECTION_PATTERNS = [
    r"ignore\s+(all\s+)?(previous|prior|above)\s+instructions?",
    r"forget\s+(all\s+)?(previous|prior|above)\s+instructions?",
    r"you\s+are\s+now\s+a",
    r"act\s+as\s+(if\s+you\s+are\s+)?a",
    r"pretend\s+(you\s+are|to\s+be)",
    r"your\s+(new\s+)?instructions?\s+(are|is)\s*:",
    r"disregard\s+(all\s+)?(previous|prior|above)",
    r"override\s+(your\s+)?(previous\s+)?(instructions?|prompt|rules?)",
    r"system\s*prompt\s*:",
    r"<\s*system\s*>",
    r"\[\s*system\s*\]",
    r"jailbreak",
    r"do\s+anything\s+now",
    r"dan\s+mode",
]

# Compiled for performance
_compiled = [re.compile(p, re.IGNORECASE) for p in INJECTION_PATTERNS]

MAX_QUERY_LENGTH = 1000

def sanitise_query(query: str) -> str:
    """
    Strip control characters and excessive whitespace from the query.
    """
    # Remove null bytes and other control characters
    query = re.sub(r"[\x00-\x08\x0b\x0c\x0e-\x1f\x7f]", "", query)
    # Collapse excessive whitespace
    query = re.sub(r"\s{3,}", "  ", query)
    return query.strip()

def check_injection(query: str) -> str | None:
    """
    Check for prompt injection patterns.
    Returns the matched pattern string if found, None if clean.
    """
    for pattern in _compiled:
        if pattern.search(query):
            return pattern.pattern
    return None

def validate_query(query: str) -> tuple[str, str | None]:
    """
    Validate and sanitise a query.
    Returns (sanitised_query, error_message).
    error_message is None if the query is valid.
    """
    if not query or not query.strip():
        return "", "Query cannot be empty."

    if len(query) > MAX_QUERY_LENGTH:
        return "", f"Query too long. Maximum {MAX_QUERY_LENGTH} characters allowed."

    query = sanitise_query(query)

    injection = check_injection(query)
    if injection:
        return "", "I can only help with questions about Starbound rocket components."

    return query, None