FROM rust:1.81 AS builder

RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk wasm-bindgen-cli

WORKDIR /app
COPY frontend/ ./frontend/
COPY frontend/package.json ./frontend/
RUN cd frontend && npm install

WORKDIR /app/frontend
RUN trunk build --release

FROM nginx:alpine
COPY --from=builder /app/frontend/dist /usr/share/nginx/html
COPY docker/nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
