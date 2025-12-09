# --- Build Frontend ---
FROM node:20 AS frontend-builder
WORKDIR /app
COPY frontend/ .
RUN npm ci && npm run build
RUN mkdir -p /output/static && cp -r dist/* /output/static/

# --- Build Backend ---
FROM rust:1.83 AS backend-builder
WORKDIR /app
COPY backend/ .
RUN cargo build --release --locked

# --- Runtime ---
FROM gcr.io/distroless/cc-debian12
COPY --from=frontend-builder /output/static /static
COPY --from=backend-builder /app/target/release/nook-backend /nook-backend
EXPOSE 3000
CMD ["/nook-backend"]