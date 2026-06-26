# Docker Update Breaks GitHub Auth — Recovery Procedure

## What Happens (Session 2026-06-17)

A Docker update on the homeserver:
- Removed `gh` CLI from the container
- Broke Git credential helper
- **Invalidated all existing PATs** (both classic and fine-grained become placeholders)
- Result: `gh auth status` fails, API calls return 401 Bad credentials

## Immediate Recovery Steps

### 1. Generate New Classic PAT
- Go to: github.com/settings/tokens/new (classic, not fine-grained)
- Name: `Nook CI/CD - <date>`
- Scopes: `repo` (full) + `workflow`
- Copy the `ghp_...` token immediately

### 2. Validate Token
```bash
curl -H "Authorization: token ghp_YOUR_NEW_TOKEN" https://api.github.com/user
# Must return 200 OK with user login and scopes
```

### 3. Update All Token Locations

**A. Main .env (persistent, shared)**
```python
with open("/opt/data/.env", "r") as f:
    lines = f.readlines()
with open("/opt/data/.env", "w") as f:
    for l in lines:
        f.write(f"GITHUB_TOKEN=*** if l.startswith("GITHUB_TOKEN=*** else l)
```

**B. Nook Profile .env (if using nook profile)**
```python
with open("/opt/data/profiles/nook/.env", "r") as f:
    lines = f.readlines()
with open("/opt/data/profiles/nook/.env", "w") as f:
    for l in lines:
        f.write(f"GITHUB_TOKEN=*** if l.startswith("GITHUB_TOKEN=*** else l)
```

**C. Hermes config.yaml (MCP GitHub server)**
Edit `/opt/data/config.yaml` and `/opt/data/profiles/nook/config.yaml`:
```yaml
mcp_servers:
  github:
    env:
      GITHUB_TOKEN: ghp_YOUR_NEW_TOKEN
```

### 4. Test Workflow Trigger
```python
import urllib.request, json

token = "ghp_YOUR_NEW_TOKEN"
repo = "MX10-AC2N/Nook"

# List workflows to get IDs
req = urllib.request.Request(
    f"https://api.github.com/repos/{repo}/actions/workflows",
    headers={"Authorization": f"Bearer {token}", "Accept": "application/vnd.github.v3+json"}
)
with urllib.request.urlopen(req) as resp:
    for w in json.loads(resp.read()).get("workflows", []):
        print(f"ID: {w['id']} | {w['name']}")

# Trigger Frontend (first in pipeline)
trigger_id = 220018364  # "2==> 🎨 Frontend Build & Artifact"
data = json.dumps({"ref": "develop"}).encode()
req = urllib.request.Request(
    f"https://api.github.com/repos/{repo}/actions/workflows/{trigger_id}/dispatches",
    data=data,
    headers={"Authorization": f"Bearer {token}", "Content-Type": "application/json"},
    method="POST"
)
with urllib.request.urlopen(req) as resp:
    print(f"Frontend triggered: {resp.status}")
```

## Why This Happens

- Docker image update replaced the base image
- New image doesn't include `gh` CLI (was previously installed via apt)
- Git credential helper config (usually `gh auth login`) stored in container layer → lost
- PATs stored only in `/tmp/.git_token` (ephemeral) → cleared
- `.env` files on host persist but weren't synced to new container env

## Prevention

1. **Always store PAT in `/opt/data/.env`** (host-mounted, survives container rebuild)
2. **Use Classic PAT** (`ghp_...`) — simpler, less prone to scope issues
3. **Document token in `.env.example`** so rebuilds can be configured
4. **Add healthcheck** that validates GitHub API access on container startup

## Related Files

- `/opt/data/.env` — Main persistent token storage
- `/opt/data/profiles/nook/.env` — Nook profile token storage  
- `/opt/data/config.yaml` — MCP server token (two locations: main + nook profile)
- `references/github-token-management.md` — Full token management guide