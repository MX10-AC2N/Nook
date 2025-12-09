# --- Build Frontend ---
FROM node:20 AS frontend-builder
WORKDIR /app
COPY frontend/ .
RUN npm install && npm run build
# → génère ./static

# --- Build Backend ---
FROM rust:1.83 AS backend-builder
WORKDIR /app
# Copier le frontend buildé pour que le backend le voie (si besoin plus tard)
COPY --from=frontend-builder /app/static ./static
COPY backend/ .
RUN cargo build --release --locked

# --- Runtime ---
FROM gcr.io/distroless/cc-debian12
COPY --from=frontend-builder /app/static /static
COPY --from=backend-builder /app/target/release/nook-backend /nook-backend
EXPOSE 3000
CMD ["/nook-backend"]