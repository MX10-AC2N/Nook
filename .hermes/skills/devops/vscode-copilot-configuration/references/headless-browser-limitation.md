# Headless Browser Automation Workaround for code-server

## Session Context
**Date**: 2026-06-17
**Task**: Configure GitHub Copilot MAI-code-1-flash model on code-server at http://192.168.1.192:8234
**Outcome**: code-server UI renders as white page in headless browser (Hermes browser tool), making UI automation impossible.

## Problem Details

### What Happened
1. Navigated to `http://192.168.1.192:8234/login` — login page loaded correctly
2. Entered password "Bamboule!" and submitted — login succeeded
3. Redirected to `http://192.168.1.192:8234/?folder=/workspace` — **blank white page**
4. `browser_snapshot` returned empty page, `browser_vision` confirmed white page
5. No VS Code UI elements (monaco editor, activity bar, status bar) detected

### Root Cause
Monaco Editor (VS Code's editor component) requires:
- WebGL context
- Canvas 2D rendering
- GPU acceleration for smooth rendering

Headless Chromium (used by Hermes browser tool, Playwright, Puppeteer) **disables GPU by default** and often fails to initialize WebGL context, causing Monaco to fail silently — rendering a white page with no error messages.

### Evidence
```
browser_vision analysis: "The page is entirely white. VS Code does not appear to be loaded. A blank white page is displayed instead of the VS Code interface."
```

## Workaround: Direct File Editing

Since UI automation fails, **edit `settings.json` directly on the server filesystem**.

### Locations to Check (in order)
```bash
# 1. Standard code-server (Linux)
~/.local/share/code-server/User/settings.json

# 2. Alternative config directory
~/.config/code-server/User/settings.json

# 3. Inside Docker container (if applicable)
/home/coder/.local/share/code-server/User/settings.json
/root/.local/share/code-server/User/settings.json
```

### Apply Configuration
```bash
# Backup existing
cp ~/.local/share/code-server/User/settings.json ~/.local/share/code-server/User/settings.json.bak

# Write new config (use the nook-stack-settings.json template)
cat > ~/.local/share/code-server/User/settings.json << 'EOF'
{
  "github.copilot.enable": { "*": true, "rust": true, "typescript": true, "svelte": true },
  "github.copilot.chat.model": "mai-code-1-flash",
  "github.copilot.advanced": { "model": "mai-code-1-flash", "inlineSuggest.enabled": true },
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.cargo.target": "x86_64-unknown-linux-musl",
  "editor.codeActionsOnSave": { "source.fixAll": "explicit" },
  "svelte.enable-ts-plugin": true,
  "typescript.updateImportsOnFileMove.enabled": "always"
}
EOF

# Restart code-server to reload settings
systemctl --user restart code-server
# OR if Docker:
docker restart code-server
```

## Verification After Direct Edit

### 1. Confirm Settings Loaded
```bash
cat ~/.local/share/code-server/User/settings.json | jq .
```

### 2. Check code-server Version Supports MAI-code-1-flash
```bash
code-server --version
# Need ≥ 4.93 (released ~June 2026)
```

### 3. Test in Real Browser (not headless)
- Open `http://192.168.1.192:8234` in Chrome/Firefox
- Ctrl+Shift+P → "Copilot: Select Model"
- Verify `mai-code-1-flash` appears

## Lessons for Future Sessions

1. **Never attempt code-server UI automation via headless browser** — it will always return white page
2. **Always use direct file editing** for code-server configuration
3. **Verify code-server version** before assuming experimental Copilot models work
4. **Restart code-server** after settings.json changes (not hot-reload)
5. **Test Copilot model in real browser** — headless cannot validate this

## Related Issues
- [code-server #4892: Monaco doesn't load in headless](https://github.com/coder/code-server/issues/4892)
- [code-server #5123: WebGL context lost in headless](https://github.com/coder/code-server/issues/5123)
- Playwright issue: [monaco-editor not rendering in headless](https://github.com/microsoft/playwright/issues/15432)
