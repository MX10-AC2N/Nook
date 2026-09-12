# Nook Dependency Audit Report
Generated: 2026-04-28 21:11 UTC
Repo: https://github.com/MX10-AC2N/Nook (branch: develop)

## Summary
- Backend (Rust): Automated audit limited due to Rust version constraints
- Frontend (Node.js): 3 vulnerabilities found, multiple outdated packages
- CI Workflows: Healthy, use up-to-date actions, no deprecated components found

---

## 1. Backend Audit (Rust/Cargo)
### Dependencies (from Cargo.toml)
| Package             | Version Spec | Notes                          |
|---------------------|--------------|--------------------------------|
| dotenvy             | 0.15         | Environment loading            |
| hostname            | 0.4          | System hostname                |
| axum                | 0.8          | Web framework (ws, multipart)  |
| tokio               | 1.0          | Async runtime                  |
| sqlx                | 0.8.6        | Database (SQLite, migrations)  |
| reqwest             | 0.13         | HTTP client (rustls)           |
| rand                | 0.9          | Random number generation       |
| argon2              | 0.5          | Password hashing               |
| chacha20poly1305    | ^0.10        | Encryption                     |
| ring                | 0.17         | Cryptography (VAPID)           |
| governor            | 0.10         | Rate limiting                  |
| tracing-subscriber  | 0.3          | Logging                        |
| rustrtc             | 0.3.39       | SFU for group calls            |

### Audit Limitations
- Installed Rust version: 1.85.0
- cargo-audit requires Rust >=1.88, install failed
- cargo-outdated requires Rust >=1.88, install failed
- Cargo.lock exists (4780 lines) with exact dependency versions
- No automated CVE scan completed for backend

---

## 2. Frontend Audit (Node.js/npm)
### Vulnerabilities Found (npm audit)
Total: 3 vulnerabilities (1 low, 1 moderate, 1 high)

| Severity | Package         | Advisory URL                                                                 |
|----------|-----------------|-----------------------------------------------------------------------------|
| High     | @sveltejs/kit   | https://github.com/advisories/GHSA-3f6h-2hrp-w5wx (Unvalidated redirect, DoS) |
| High     | @sveltejs/kit   | https://github.com/advisories/GHSA-2crg-3p73-43xp (BODY_SIZE_LIMIT bypass)   |
| Moderate | postcss         | https://github.com/advisories/GHSA-qx2v-qp2m-jg93 (XSS via unescaped </style>) |
| Low      | cookie          | https://github.com/advisories/GHSA-pxg6-pf52-xh8x (Out of bounds chars)      |

Fix available: `npm audit fix` (run in /Nook/frontend)

### Outdated Packages (npm outdated)
| Package                       | Current | Latest | Type        |
|-------------------------------|---------|--------|-------------|
| @playwright/test              | 1.59.0  | 1.59.1 | devDependency |
| @sveltejs/kit                 | 2.55.0  | 2.58.0 | devDependency |
| @sveltejs/vite-plugin-svelte  | 6.2.4   | 7.0.0  | devDependency |
| @types/dompurify              | 3.0.5   | 3.2.0  | devDependency |
| eslint                        | 9.39.4  | 10.2.1 | devDependency |
| svelte                        | 5.55.1  | 5.55.5 | devDependency |
| typescript                    | 5.9.3   | 6.0.3  | devDependency |
| vite                          | 7.3.2   | 8.0.10 | devDependency |
| libsodium-wrappers            | 0.8.2   | 0.8.4  | dependency   |
| postcss                       | 8.5.8   | 8.5.12 | devDependency |
| prettier                      | 3.8.1   | 3.8.3  | devDependency |
| tailwindcss                   | 4.2.2   | 4.2.4  | devDependency |

Update command: `npm update` (run in /Nook/frontend)

---

## 3. CI Workflow Health Check
### Checked Workflows
- Frontend.yml: uses actions/checkout@v4, actions/setup-node@v4, actions/upload-artifact@v4 (all current)
- Backend.yml: uses actions/checkout@v4, dtolnay/rust-toolchain@nightly, Swatinem/rust-cache@v2 (all current)
- security-audit.yml: Comprehensive audit workflow, uses up-to-date actions, includes custom security pattern scans
- test-nook.yml: Main CI pipeline, uses current v4 actions, proper concurrency settings

### Key Findings
- No deprecated actions found (all use v4 or maintained third-party actions)
- Proper permissions set for GITHUB_TOKEN
- Caching configured for Node.js and Rust dependencies
- Security audit workflow includes npm audit, cargo audit (with fallback), clippy, and custom XSS/injection pattern scans
- Workflows have proper error handling and artifact retention set to 1-7 days

### Recommendations
1. Fix frontend vulnerabilities: run `npm audit fix` in /Nook/frontend
2. Update outdated frontend packages regularly
3. Upgrade Rust toolchain to >=1.88 to enable cargo-audit for backend CVE scans
4. Monitor security-audit.yml runs scheduled weekly (Mondays 04:00 UTC)

---

## 4. Action Items
| Priority | Task | Component |
|----------|------|-----------|
| High     | Fix @sveltejs/kit and postcss vulnerabilities | Frontend |
| Medium   | Update outdated frontend packages | Frontend |
| Medium   | Upgrade Rust toolchain to >=1.88 for backend CVE scans | Backend |
| Low      | Review custom security patterns in security-audit.yml | CI |
