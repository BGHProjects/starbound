FROM golang:1.23-alpine AS builder

WORKDIR /app
COPY gateway/ .
RUN go mod download
RUN go build -o gateway ./cmd/main.go

FROM alpine:latest
RUN apk --no-cache add ca-certificates
WORKDIR /app
COPY --from=builder /app/gateway .
COPY gateway/internal/db/products_seed.json ./internal/db/products_seed.json
EXPOSE 8000
CMD ["./gateway"]
