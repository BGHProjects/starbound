FROM python:3.11-slim

WORKDIR /app
COPY rag-service/requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY rag-service/ .
COPY gateway/internal/db/products_seed.json /app/gateway/internal/db/products_seed.json

EXPOSE 8001
CMD ["uvicorn", "app.main:app", "--host", "0.0.0.0", "--port", "8001"]
