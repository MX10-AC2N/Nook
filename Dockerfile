# --- Build Frontend ---
FROM node:20 AS frontend-builder
WORKDIR /app
COPY frontend/ .
RUN npm install && npm run build

# --- Build Backend ---
FROM rust:1.83 AS backend-builder
WORKDIR /app
COPY backend/ .
RUN cargo build --release

# --- Runtime ---
FROM gcr.io/distroless/cc-debian12

# 1. Définir l'utilisateur non-root AVANT WORKDIR
USER nonroot

# 2. Définir le répertoire de travail (il sera créé avec les bonnes permissions)
WORKDIR /app

# 3. Copier les fichiers de l'application
COPY --from=frontend-builder --chown=nonroot:nonroot /app/build ./static
COPY --from=backend-builder --chown=nonroot:nonroot /app/target/release/nook-backend ./

EXPOSE 3000
CMD ["./nook-backend"]