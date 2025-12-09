# --- Build Frontend (SvelteKit) ---
FROM node:20 AS frontend-builder
WORKDIR /app
COPY frontend/ .
RUN npm install && npm run build
# → génère ./build/

# --- Build Backend (Rust) ---
FROM rust:1.83 AS backend-builder
WORKDIR /app
COPY backend/ .
RUN cargo build --release

# --- Final Runtime ---
FROM gcr.io/distroless/cc-debian12
WORKDIR /app
COPY --from=frontend-builder /app/build ./static
COPY --from=backend-builder /app/target/release/nook-backend ./
EXPOSE 3000
CMD ["./nook-backend"]