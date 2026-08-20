# Onboarding for New Developers

> **Version**: 0.8.0 | **Created**: 2026-08-17 | **Assignee**: docs-writer

> **Goal**: Get a new developer productive with Nook development in under 10 minutes.

---

## 1. Nook Overview

Nook is a self-hosted, end-to-end encrypted family messaging platform. Key features:

- 💬 **Messages**: Real-time chat with emojis, photos, files, groups & private
- 🔐 **E2EE**: X25519 + XChaCha20 encryption — even the admin cannot read
- 📞 **Calls**: WebRTC P2P + SFU audio/video (server never sees the stream)
- 📅 **Calendar**: Family events, birthdays, appointments with drag‑and‑drop
- ♟️ **Chess**: Play against 5 difficulty levels (easy → godlike) or vs members
- 📊 **Polls**: "What should we eat?" in 3 clicks
- 🎨 **Themes**: Jardin Stern 🌿, Space Hub 🚀, Maison 🏠 + dark mode
- 📁 **Files**: Uploads up to 50 MB, encrypted on disk
- 🔔 **Notifications**: Push via VAPID, even if tab is closed
- 📁 **WebRTC / TURN**: STUN/TURN credentials via `/webrtc/ice-config`

All features run in a **single Docker container**.

---

## 2. Technology Stack

| Layer | Language / Framework | Notes |
|-------|---------------------|-------|
| **Backend** | Rust 1.72 + Axum 0.8 + SQLx + SQLite | `backend/src/` — async, typed queries |
| **Frontend** | SvelteKit 5 + Runes + TypeScript + Tailwind | `frontend/src/` — reactive, component‑first |
| **Infra** | Docker single‑container | `docker-compose.yml` — dev & prod |
| **Crypto** | X25519, XChaCha20, HMAC‑SHA256 | `backend/crypto/` |
| **DB** | SQLite (development), Postgres (production‑ready) | Migrations via `sqlx` |

---

## 3. Local Setup

### 3.1 Clone & Init

```bash
# 1. Clone the repo (develop branch only)
git clone https://github.com/MX10-AC2N/Nook.git
cd Nook
git checkout develop

# 2. Copy env and fill secrets
cp .env.example .env
# Required: TURN_SECRET (generate with `openssl rand -hex 32`)
# Optional: DATABASE_URL (defaults to ./nook.db)
```

### 3.2 Backend (Rust)

```bash
# Install if missing: rustup toolchain install 1.72
cargo build  # compiles backend
# Run migrations (creates ./nook.db if using SQLite)
sqlx migrate run
# Start API (listens on 0.0.0.0:6300 by default)
cargo run
```

### 3.3 Frontend (SvelteKit)

```bash
npm install  # or pnpm install
# Dev server (Vite, http://localhost:5173)
npm run dev
```

### 3.4 Environment Variables

Add to `.env` (minimal set):

```
# Required
TURN_SECRET=your-turn-secret-hex-string

# Optional — SQLite dev
DATABASE_URL=./nook.db

# Optional — Frontend
VITE_API_BASE_URL=http://localhost:6300/api
VITE_WS_BASE_URL=http://localhost:6300/ws
```

### 3.5 Test Server (Docker)

```bash
# The repo includes a compose file that runs everything on 192.168.1.192
docker compose -f docker-compose.yml up -d
# Verify:
# - API responds at http://192.168.1.192:6300/api/health
# - Frontend serves at http://192.168.1.192:80
# - WebSocket works at ws://192.168.1.192:6300/ws
```

---

## 4. Development Conventions

| Convention | Rule |
|------------|------|
| **1 engram PLUR per task** | Use `mcp__plur__plur_learn` after any significant change. Tags: `[nook, <profil>, YYYY-MM-DD, <sujet>]` |
| **One Kanban task at a time** | Never multitask across Kanban cards — focus flow is essential |
| **serde e2ee.rs keyVersion** | Keep snake_case (`key_version`), never camelCase. Clippy `--fix` blind runs will break it — review each change |
| **Clippy** | Run `cargo clippy` but **never** pipe directly to `--fix` auto‑apply. Inspect each warning first |
| **Git workflow** | `git checkout -b feat/X` from `develop`; PR against `develop`; squash‑merge only |
| **Rustfmt** | `cargo fmt` is mandatory before commit |
| **TypeScript** | `npm run lint` and `npm run type-check` must pass |
| **Commits** | Conventional Commits (`feat:`, `fix:`, `doc:`, `refactor:`). No "refactored" in CHANGELOG — describe what changed for the user |

---

## 5. Team Tools

| Tool | Purpose |
|------|---------|
| **PLUR** | Shared memory across all 17 Hermes profiles. `mcp__plur__plur_learn` / `mcp__plur__plur_recall` |
| **Kanban** | Task board at `hermes kanban` — one task per card, never split |
| **Swarm Monitor** | Dashboard at `:9090` — 16/16 agents visible. Keep it running! |
| **Hermes Agent CLI** | `hermes config set ...`, `hermes tools` for profile management |
| **GitHub CLI** | `gh pr create`, `gh pr view`, `gh issue` — GITHUB_TOKEN is stripped in terminal, use `gh` locally |

---

## 6. Where to Find Things

| Category | Path |
|----------|------|
| **Profile config** | `.hermes/profiles/docs-writer/SOUL.md`, `.hermes/AGENTS.md` |
| **Roles** | `.hermes/roles/` — 22 role files, all existent (verify) |
| **Backend source** | `backend/src/` — Axum handlers, SQLx queries, crypto |
| **Frontend source** | `frontend/src/` — Svelte 5 Runes, Tailwind components |
| **Docs index** | `docs/` — API.md, onboarding/, ADR, disaster‑recovery, ops‑status |
| **SOUL.md (your profile)** | `.hermes/profiles/docs-writer/SOUL.md` — keep in sync with role changes |
| **CHANGELOG** | `CHANGELOG.md` — Keep a Changelog format, one entry per PR |
| **Architecture diagram** | `.hermes/rules/architecture.md` — stack versions, module diagram |

---

## 7. Models (LLM)

- **Default**: `nemotron-3.5-lightning-free`
- **Orchestrator**: `tencent/hy3:free`
- When asking the orchestrator for help, use the `hermes-agent` skill and reference the model config in `.hermes/`

---

## 8. Verification Checklist

- [ ] File `docs/onboarding/new-developer.md` exists and is committed on `develop`
- [ ] Link added from `README.md` or `docs/index.md` if one exists
- [ ] PLUR engram created: `ENG-2026-0817-001` with tags `[nook, docs-writer, 2026-08-17, onboarding]`
- [ ] No breakage of `serde e2ee.rs` keyVersion (snake_case verified)
- [ ] `cargo clippy` passes without `--fix` blind apply
- [ ] One Kanban task active at a time (no multitasking)

---

## 9. Quick Reference Commands

```bash
# Rust
cargo build
cargo clippy        # inspect, never auto‑fix blindly
cargo fmt           # mandatory before commit

# Frontend
npm install
npm run dev         # Vite dev server
npm run lint        # TypeScript + ESLint
npm run type-check  # full type check

# Docker
docker compose up -d        # start all services
docker compose down         # stop all services
docker compose logs -f      # follow logs

# Git
git checkout develop
git checkout -b feat/X      # new branch from develop
git commit -m "feat: something"
git push origin feat/X

# PLUR (memory)
mcp__plur__plur_learn --tag nook --tag docs-writer --tag 2026-08-17 --tag onboarding --statement "New developer onboarded"
```