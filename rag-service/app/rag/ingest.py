import json
import os
from pathlib import Path
from langchain_core.documents import Document
from app.rag.vectorstore import get_vectorstore, is_empty

# Path to the gateway's product seed file
SEED_PATH = Path(__file__).parents[3] / "gateway" / "internal" / "db" / "products_seed.json"

def product_to_document(product: dict) -> Document:
    """
    Convert a product dict into a rich human-readable document
    that gives the LLM enough context to answer questions about it.
    """
    lines = []

    name         = product.get("name", "Unknown")
    product_type = product.get("product_type", "")
    group        = product.get("group", "")
    price        = product.get("price", 0)
    in_stock     = product.get("in_stock", False)
    stock_count  = product.get("stock_count", 0)
    product_id   = product.get("id", "")

    lines.append(f"Product: {name}")
    lines.append(f"ID: {product_id}")
    lines.append(f"Type: {product_type}")
    lines.append(f"Category: {group}")
    lines.append(f"Price: ${price:,.2f} USD")
    lines.append(f"In stock: {'Yes' if in_stock else 'No'} ({stock_count} units available)")

    attrs = product.get("attributes", {})
    if attrs:
        lines.append("Specifications:")
        for key, value in attrs.items():
            label = key.replace("_", " ").title()
            lines.append(f"  - {label}: {value}")

    return Document(
        page_content="\n".join(lines),
        metadata={
            "product_id":   product_id,
            "name":         name,
            "product_type": product_type,
            "group":        group,
            "price":        price,
            "in_stock":     in_stock,
        }
    )

def ingest_products() -> int:
    """
    Load products from the seed file and ingest into the vector store.
    Returns the number of products ingested.
    """
    if not SEED_PATH.exists():
        raise FileNotFoundError(
            f"Product seed file not found at {SEED_PATH}. "
            f"Make sure the gateway is set up correctly."
        )

    with open(SEED_PATH, "r", encoding="utf-8") as f:
        products = json.load(f)

    documents = [product_to_document(p) for p in products]
    vs        = get_vectorstore()
    vs.add_documents(documents)

    return len(documents)

def maybe_ingest() -> None:
    """
    Ingest products only if the vector store is empty.
    Called on startup.
    """
    if is_empty():
        print("Vector store is empty — ingesting products...")
        count = ingest_products()
        print(f"Ingested {count} products into vector store.")
    else:
        print("Vector store already populated — skipping ingest.")