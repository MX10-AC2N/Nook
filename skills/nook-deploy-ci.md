---
name: nook-deploy-ci
category: devops
description: Multi-workflow CI for Nook — Backend, Frontend, Turn-server → GHCR multi-arch images → one-command deploy
---

# Nook Deploy CI

## Overview
4-workflow pipeline producing pre-compiled multi-arch Docker images on GHCR.
End user deploys with: `docker compose pull && docker compose up -d`

## Workflow Pipeline

1. **Backend.yml** (workflow_dispatch) — Build backend for both arches
   - amd64 → ubuntu-latest + musl-gcc
   - arm64 → ubuntu-24.04-arm + musl-gcc (native, no cross-compiler)
   - Artifacts: nook-backend-x86_64-unknown-linux-musl, nook-backend-aarch64-unknown-linux-musl

2. **Frontend.yml** (workflow_dispatch) — Build SvelteKit frontend
   - Artifact: nook-frontend

3. **turn.yml** (workflow_dispatch) — Build turn-server for both arches
   - Clones mycrl/turn-rs from GitHub, builds with musl
   - Needs: protobuf-compiler, musl-tools, gcc-aarch64-linux-gnu (for arm64 step)
   - Artifacts: nook-turn-server-amd64, nook-turn-server-arm64

4. **Docker.yml** (workflow_dispatch) — Assemble + Push to GHCR
   - Downloads all artifacts from workflows 1-3
   - Builds two multi-arch images with docker/build-push-action:
     - ghcr.io/mx10-ac2n/nook:latest (backend + frontend)
     - ghcr.io/mx10-ac2n/nook/turn-server:latest
   - Platforms: linux/amd64,linux/arm64 (uses setup-qemu-action)

## Key Patterns

### musl-native approach (preferred over cross-compilation)
```yaml
  - arch: amd64
    target: x86_64-unknown-linux-musl
    runner: ubuntu-latest
  - arch: arm64
    target: aarch64-unknown-linux-musl
    runner: ubuntu-24.04-arm  # Native ARM64 runner, no cross-compiler needed
```

### Linker setup via GITHUB_ENV (not inline)
```yaml
  - name: Set linker
    run: |
      echo "CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=musl-gcc" >> $GITHUB_ENV
```

### Cross-workflow artifact download
```yaml
  - name: Download turn amd64
    uses: dawidd6/action-download-artifact@v6
    with:
      workflow: turn.yml
      branch: ${{ github.ref_name }}
      name: nook-turn-server-amd64
      path: docker-context/turn
```

### Multi-arch Docker build
```yaml
  - uses: docker/setup-qemu-action@v3
  - uses: docker/setup-buildx-action@v3
  - uses: docker/build-push-action@v6
    with:
      push: true
      platforms: linux/amd64,linux/arm64
      tags: |
        ghcr.io/${{ steps.info.outputs.repo }}:latest
      build-args: |
        TARGETARCH
```

### Dockerfile.release pattern (consumes pre-built binaries)
```dockerfile
FROM alpine:3.21 AS runtime
RUN apk add --no-cache ca-certificates
ARG TARGETARCH=amd64
COPY --chmod=755 binary-${TARGETARCH} /usr/local/bin/binary
```

### Docker tag must be lowercase
```yaml
  - run: |
      echo "repo=$(echo ${{ github.repository }} | tr '[:upper:]' '[:lower:]')" >> $GITHUB_OUTPUT
```

## Pitfalls to avoid
- Don't use cross-compilation for aarch64 on x86_64 runner (zig is unstable)
- Don't put shell redirects in COPY instructions (Docker doesn't parse them)
- Don't forget protobuf-compiler for turn-rs build (needed for gRPC protos)
- Workflow names in workflow_run triggers must match exactly (including numbers, spaces, emojis)
- Docker build context paths matter: downloaded artifacts go inside the context dir
- BuildKit cache from x86_64 can corrupt ARM64 builds — use --no-cache or cache busting

## Turn-R specifics
- Source: https://github.com/mycrl/turn-rs (cloned during build, not in repo)
- Needs protobuf-compiler for protoc (gRPC definition compilation)
- Dockerfile.release copies turn-server-${TARGETARCH} from build context
