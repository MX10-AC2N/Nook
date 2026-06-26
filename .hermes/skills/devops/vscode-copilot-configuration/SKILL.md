---
name: vscode-copilot-configuration
description: Configure VS Code/code-server and GitHub Copilot including experimental models (MAI-code-1-flash, GPT-4o, Claude). Covers settings.json management, model selection, version requirements, and headless automation limitations.
category: devops
tags:
  - vscode
  - code-server
  - github-copilot
  - mai-code-1-flash
  - experimental-models
  - remote-development
  - settings-json
---

# VS Code / code-server + GitHub Copilot Configuration

## Purpose
Configure VS Code or code-server (browser-based VS Code) for optimal GitHub Copilot usage, including experimental/preview models like MAI-code-1-flash.

## When to Use
- Setting up a new code-server instance for a project
- Switching Copilot models (especially experimental ones)
- Configuring language-specific Copilot settings (Rust, TypeScript, Svelte)
- Troubleshooting Copilot not showing expected models

## Key Concepts

### code-server vs VS Code Desktop
| Aspect | VS Code Desktop | code-server |
|--------|----------------|-------------|
| Settings location | `~/Library/Application Support/Code/User/settings.json` (macOS)<br>`~/.config/Code/User/settings.json` (Linux) | `~/.local/share/code-server/User/settings.json`<br>`~/.config/code-server/User/settings.json` |
| Extensions | Local marketplace | OpenVSX / local marketplace |
| Copilot auth | Built-in | Requires `code-server --link` or manual token |

### Experimental Models (MAI-code-1-flash, etc.)
- **MAI-code-1-flash**: Announced June 2, 2026. Requires:
  - VS Code ≥ 1.96 (or Insiders)
  - GitHub Copilot Nightly / pre-release
  - code-server ≥ 4.93 (check `code-server --version`)
- If model doesn't appear in picker: version mismatch. Fallback to `gpt-4o` or `claude-3.5-sonnet`.

## Configuration: settings.json

### Minimal Recommended Config for Nook Stack (Rust + Svelte 5 + TS)
```json
{
  "github.copilot.enable": {
    "*": true,
    "rust": true,
    "typescript": true,
    "svelte": true
  },
  "github.copilot.chat.model": "mai-code-1-flash",
  "github.copilot.advanced": {
    "model": "mai-code-1-flash",
    "inlineSuggest.enabled": true
  },
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.cargo.target": "x86_64-unknown-linux-musl",
  "editor.codeActionsOnSave": {
    "source.fixAll": "explicit"
  },
  "svelte.enable-ts-plugin": true,
  "typescript.updateImportsOnFileMove.enabled": "always"
}
```

### Model Selection Priority
1. `mai-code-1-flash` (experimental, fastest, code-specialized)
2. `gpt-4o` (general purpose, reliable)
3. `claude-3.5-sonnet` (strong reasoning, good for architecture)
4. `gpt-4-turbo` (legacy fallback)

## Headless Browser Automation Limitation

### Problem
code-server's Monaco editor **does not render in headless browsers** (Playwright, Puppeteer, Hermes browser tool). Result: white page, no interactive elements.

### Root Cause
Monaco requires WebGL/canvas rendering that headless Chromium disables or fails to initialize without GPU.

### Workaround: Direct File Editing
**Do not attempt UI automation.** Instead, edit `settings.json` directly on the server:

```bash
# Locate settings.json
ls ~/.local/share/code-server/User/settings.json
ls ~/.config/code-server/User/settings.json

# Edit directly (vim, nano, or cat heredoc)
cat > ~/.local/share/code-server/User/settings.json << 'EOF'
{
  "github.copilot.chat.model": "mai-code-1-flash",
  "github.copilot.advanced": { "model": "mai-code-1-flash" }
  // ... rest of config
}
EOF

# Restart code-server to pick up changes
systemctl --user restart code-server
# or: docker restart code-server
```

## Verification Steps

### 1. Check code-server Version
```bash
code-server --version
# Should be ≥ 4.93 for latest Copilot support
```

### 2. Verify Settings Applied
```bash
cat ~/.local/share/code-server/User/settings.json | jq .
```

### 3. Test Copilot Model in UI
- Open code-server in browser
- Ctrl+Shift+P → "Copilot: Select Model"
- Verify `mai-code-1-flash` appears in list

### 4. Test Inline Suggestions
- Open a `.rs` or `.ts` file
- Type a function signature
- Wait for ghost text suggestion (Tab to accept)

## Common Pitfalls

| Pitfall | Symptom | Fix |
|---------|---------|-----|
| Old code-server | Model not in picker | Update Docker image: `codercom/code-server:latest` |
| Wrong settings path | Config ignored | Check both `~/.local/share/code-server/` and `~/.config/code-server/` |
| Copilot not authenticated | "Sign in" prompt | Run `code-server --link` or set `GITHUB_COPILOT_TOKEN` env |
| musl target missing | Rust analyzer errors | Install: `rustup target add x86_64-unknown-linux-musl` |
| Svelte 5 runes not recognized | False TS errors | Ensure `svelte.enable-ts-plugin: true` + TypeScript 5.5+ |

## References
- [code-server config docs](https://github.com/coder/code-server/blob/main/docs/config.md)
- [GitHub Copilot model selection](https://docs.github.com/en/copilot/using-github-copilot/changing-the-language-model-for-copilot-chat)
- [MAI-code-1-flash announcement](https://github.blog/changelog/2026-06-02-mai-code-1-flash-is-now-available-for-github-copilot/)
- [Rust analyzer config](https://rust-analyzer.github.io/manual.html#configuration)

## Related Skills
- `nook-rust-backend` — Rust toolchain + clippy config for Nook
- `nook-svelte-frontend` — Svelte 5 + TypeScript config for Nook
- `docker-management` — code-server container deployment
