# 🛠️ Session Start — Outils Hermes

Au démarrage de chaque session, exécuter :

```bash
bash scripts/hermes-tools-setup.sh
```

Cela installe :
- `curl` — requêtes HTTP
- `playwright` — screenshots navigateur
- `chromium` — headless browser
- `lightpanda` — fast headless browser (111MB)

## Raccourci
Si le script échoue, installer manuellement :
```bash
apt-get install -y curl
pip install --break-system-packages playwright
playwright install chromium
curl -L -o /tmp/lightpanda https://github.com/lightpanda-io/browser/releases/download/nightly/lightpanda-x86_64-linux && chmod +x /tmp/lightpanda
```

## Utilisation
- `curl http://192.168.1.192:6300/api/health` — test API
- `python3 -c "from playwright.sync_api import sync_playwright"` — vérifier playwright
- `/tmp/lightpanda fetch --dump html http://...` — fetch rapide
