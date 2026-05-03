#!/bin/bash
# Environment setup for Nook development
# This script sets up all necessary paths and variables for Hermes Agent

export PATH="$HOME/.cargo/bin:$PATH"
export NOOK_HOME="/opt/data/home/.hermes/Nook"
export HERMES_HOME="/opt/data/home/.hermes/Nook/.hermes"

# Add hermes tools to PATH
export PATH="$HERMES_HOME/tools/bin:$PATH"

# Docker compose location
export COMPOSE_FILE="$NOOK_HOME/docker-compose.yml"

# SQLite database location
export DATABASE_URL="sqlite:$NOOK_HOME/backend/nook.db"

echo "✅ Nook development environment loaded"
echo "   NOOK_HOME=$NOOK_HOME"
echo "   HERMES_HOME=$HERMES_HOME"
echo "   wasm-pack: $(which wasm-pack 2>/dev/null || echo 'NOT FOUND')"
