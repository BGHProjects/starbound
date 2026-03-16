from langchain_chroma import Chroma
from app.rag.embeddings import get_embeddings

_vectorstore: Chroma | None = None

def get_vectorstore() -> Chroma:
    global _vectorstore
    if _vectorstore is None:
        _vectorstore = Chroma(
            collection_name="starbound_products",
            embedding_function=get_embeddings(),
        )
    return _vectorstore

def is_empty() -> bool:
    vs = get_vectorstore()
    return vs._collection.count() == 0