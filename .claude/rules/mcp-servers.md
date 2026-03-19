# 🔌 MCP Servers — Nook

> Ajouté en session 38. Référence des serveurs MCP disponibles pour les agents.

---

## 🎨 Svelte MCP — Documentation officielle Svelte 5

**URL** : `https://mcp.svelte.dev/mcp`  
**Type** : Remote (HTTP/SSE)  
**Utilisé par** : 🎨 SVELTE

### Configuration (une fois, dans le client MCP)

```json
{
  "mcpServers": {
    "svelte": {
      "url": "https://mcp.svelte.dev/mcp"
    }
  }
}
```

### Outils disponibles

| Outil | Quand l'utiliser |
|-------|-----------------|
| `list-sections` | **En premier** — lister toutes les sections de docs disponibles. Utiliser au début de chaque tâche Svelte. |
| `get-documentation` | Après `list-sections` — récupérer le contenu complet des sections pertinentes (accepte plusieurs sections en une fois). |
| `svelte-autofixer` | **Obligatoire avant livraison** — analyser le code Svelte produit. Relancer jusqu'à 0 problèmes. |
| `playground-link` | Optionnel — générer un lien Svelte Playground pour une démo. Ne jamais utiliser si du code a été écrit dans le projet. |

### Workflow 🎨 SVELTE avec MCP

```
1. list-sections            → identifier les sections pertinentes pour la tâche
2. get-documentation(...)   → charger la doc Svelte 5 / SvelteKit exacte
3. [écrire le code]         → s'appuyer sur la doc fraîche
4. svelte-autofixer(code)   → valider, corriger jusqu'à 0 issues
5. Livrer le fichier .txt   → avec chemin exact en tête
```

### Règle d'usage

> Toujours appeler `list-sections` EN PREMIER dans chaque tâche Svelte,
> même si le sujet semble familier. La doc Svelte 5 Runes évolue fréquemment.
> Ne jamais se fier à la mémoire — utiliser le MCP pour confirmer.

---

## 🦀 Rust MCP — Outils Cargo + rust-analyzer

### Option A — rust-mcp-server (Cargo, outils de build)

**Installation** (locale, une fois) :
```bash
cargo install rust-mcp-server
```

**Configuration** :
```json
{
  "mcpServers": {
    "rust": {
      "command": "rust-mcp-server",
      "args": ["--workspace", "/path/to/nook/backend"]
    }
  }
}
```

**Outils exposés** : `cargo-check`, `cargo-build`, `cargo-test`, `cargo-clippy`, `cargo-fmt`, `cargo-add`, `cargo-doc`

**Utilisé par** : 🦀 RUST — pour valider la compilation sans sortir du contexte

### Option B — mcp-language-server + rust-analyzer (sémantique LSP)

**Installation** :
```bash
rustup component add rust-analyzer
go install github.com/isaacphi/mcp-language-server@latest
```

**Configuration** :
```json
{
  "mcpServers": {
    "rust-analyzer": {
      "command": "mcp-language-server",
      "args": ["--workspace", "/path/to/nook/backend", "--lsp", "rust-analyzer"]
    }
  }
}
```

**Outils exposés** : `definition`, `references`, `diagnostics`, `hover`, `edit_file`

**Utilisé par** : 🦀 RUST — pour naviguer le code, trouver les dépendances, diagnostics

### Règle d'usage Rust

> `rust-mcp-server` pour les actions de build/test.  
> `mcp-language-server` pour la navigation sémantique (find references, diagnostics).  
> En CI/CD : ces outils sont locaux — ne pas les référencer dans les workflows GitHub Actions.

---

## 🌐 Lightpanda — Navigateur headless (référence future)

**Repo** : `https://github.com/lightpanda-io/browser`  
**Status** : ⚠️ Beta — ne pas remplacer Playwright en production pour l'instant  
**Pertinence** : 🧪 E2E (futur), 🚀 DEVOPS (scraping CI)

### Pourquoi le noter

- Écrit en Zig — ultra-léger (9x moins de RAM que Chrome, 11x plus rapide)
- Compatible CDP → théoriquement compatible avec Playwright/Puppeteer
- Intéressant pour les tests E2E en CI si les limitations JS sont résolues

### Installation (si besoin de tester)

```bash
# Linux x86_64
curl -L -o lightpanda https://github.com/lightpanda-io/browser/releases/download/nightly/lightpanda-x86_64-linux
chmod a+x ./lightpanda
./lightpanda serve --host 127.0.0.1 --port 9222

# Docker
docker run -d --name lightpanda -p 9222:9222 lightpanda/browser:nightly
```

### Limitation actuelle

> Web APIs partielles (WIP). Des tests Playwright peuvent échouer si Lightpanda
> ne supporte pas encore l'API utilisée. Surveiller la maturité avant migration.
> Decision : garder Playwright pour Nook jusqu'à ce que Lightpanda atteigne
> la parité avec Chromium headless sur les APIs utilisées par nos tests.

---

## 📋 Résumé — Quel MCP pour quelle tâche

| Agent | Tâche | MCP recommandé |
|-------|-------|----------------|
| 🎨 SVELTE | Écrire/corriger composant Svelte 5 | `svelte` (mcp.svelte.dev) |
| 🎨 SVELTE | Valider code avant livraison | `svelte-autofixer` |
| 🦀 RUST | Vérifier compilation | `rust-mcp-server` → `cargo-check` |
| 🦀 RUST | Trouver usages d'une fonction | `mcp-language-server` → `references` |
| 🧪 E2E | Tests headless (futur) | Lightpanda (surveiller) |
