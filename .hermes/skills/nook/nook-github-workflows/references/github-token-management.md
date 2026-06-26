# GitHub Token Management for Nook CI/CD

## Current State (2026-06-17 session)

**gh CLI is NOT available in Docker container** — it was removed in a recent Docker update. **This update also invalidated all existing PATs** (credential helper broken). All workflow triggering must use direct GitHub API calls.

### ⚠️ Docker Update Breaks GitHub Auth (Session 2026-06-17)
After a Docker update on the homeserver:
- `gh` CLI was removed from the container
- Git credential helper stopped working
- **All existing PATs (classic and fine-grained) became invalid/placeholders**
- New Classic PAT with `repo` + `workflow` scopes required
- Token must be stored in **`/opt/data/.env`** (persistent), NOT `/tmp/.git_token` (ephemeral)

## Token Strategy

### Token Format
- **Classic PAT (ghp_)** with scopes: `repo` + `workflow` — simpler, works reliably
- **Fine-grained PAT** needs: `Contents: Read/Write`, `Workflows: Read/Write`, `Actions: Read` — more restrictive but complex to configure

### Token Validation (MANDATORY before use)
Always validate token has push/workflow permissions via `/user` endpoint:
```python
import urllib.request, json

token = "YOUR_TOKEN"
req = urllib.request.Request("https://api.github.com/user")
req.add_header("Authorization", f"token {token}")
try:
    with urllib.request.urlopen(req) as resp:
        data = json.loads(resp.read())
        print(f"User: {data.get('login')}")
        # Check if token has expected scopes via response headers
        scopes = resp.headers.get('X-OAuth-Scopes', '').split(', ')
        print(f"Scopes: {scopes}")
except urllib.error.HTTPError as e:
    print(f"Token invalid: {e.code} - {e.read().decode()}")
```

### Token Storage
- **Use `.env` file** ( `/opt/data/.env` ) — persistent, accessible to scripts
- **NOT `/tmp/.git_token`** — ephemeral, cleared on container restart
- Update `.env` via Python `open()` (patch tool may fail due to caching):
```python
env_path = "/opt/data/.env"
new_token = "ghp_YOUR_NEW_TOKEN"
with open(env_path, "r") as f:
    lines = f.readlines()
with open(env_path, "w") as f:
    for line in lines:
        if line.startswith("GITHUB_TOKEN="):
            f.write(f"GITHUB_TOKEN={new_token}\n")
        else:
            f.write(line)
```

## API-Based Workflow Triggering (Replaces gh CLI)

### List Workflows to Get Exact IDs (names have emojis!)
```python
import urllib.request, json

token = "YOUR_TOKEN"
repo = "MX10-AC2N/Nook"

req = urllib.request.Request(
    f"https://api.github.com/repos/{repo}/actions/workflows",
    headers={"Authorization": f"Bearer {token}", "Accept": "application/vnd.github.v3+json"}
)
with urllib.request.urlopen(req) as resp:
    workflows = json.loads(resp.read()).get("workflows", [])
    for w in workflows:
        print(f"ID: {w['id']} | Name: {w['name']}")
```

### Trigger Workflow by ID (Not Name!)
```python
def trigger_workflow(workflow_id, ref="develop"):
    token = "YOUR_TOKEN"
    repo = "MX10-AC2N/Nook"
    data = json.dumps({"ref": ref}).encode()
    req = urllib.request.Request(
        f"https://api.github.com/repos/{repo}/actions/workflows/{workflow_id}/dispatches",
        data=data,
        headers={"Authorization": f"Bearer {token}", "Content-Type": "application/json"},
        method="POST"
    )
    with urllib.request.urlopen(req) as resp:
        print(f"Triggered workflow {workflow_id}: {resp.status}")
```

### Monitor Workflow Runs
```python
def wait_for_workflows(timeout=600):
    import time
    token = "YOUR_TOKEN"
    repo = "MX10-AC2N/Nook"
    start = time.time()
    while time.time() - start < timeout:
        req = urllib.request.Request(
            f"https://api.github.com/repos/{repo}/actions/runs?per_page=10&branch=develop",
            headers={"Authorization": f"Bearer {token}"}
        )
        with urllib.request.urlopen(req) as resp:
            runs = json.loads(resp.read()).get("workflow_runs", [])
        pending = [r for r in runs if r["status"] in ["queued", "in_progress"]]
        if not pending:
            print("All workflows completed!")
            return True
        print(f"Pending: {len(pending)}")
        time.sleep(30)
    return False
```

### Trigger Order (Frontend → Backend → Turn → Docker → Test Nook)
```python
# 1. Get workflow IDs
wf_ids = {
    "frontend": 220018364,   # "2==> 🎨 Frontend Build & Artifact"
    "backend": 220018362,    # "1==>🏗️ Backend Build & Artifact"
    "turn": 257238341,       # "3 ==> Turn-Server Build and Artifact"
    "docker": 220018363,     # "4==> 🐳 Docker Build & Push"
    "test_nook": None        # Get from list - "4 ==> Test Nook"
}

# 2. Trigger in order
trigger_workflow(wf_ids["frontend"])
wait_for_workflows(300)

trigger_workflow(wf_ids["backend"])
wait_for_workflows(300)

trigger_workflow(wf_ids["turn"])
wait_for_workflows(300)

trigger_workflow(wf_ids["docker"])
wait_for_workflows(300)

# 3. Final E2E validation
trigger_workflow(wf_ids["test_nook"])
```

## Common Pitfalls

| Issue | Cause | Fix |
|-------|-------|-----|
| 401 Bad credentials | Token expired/revoked or wrong format | Regenerate classic PAT with `repo`+`workflow` |
| 404 on dispatch | Used workflow name instead of ID | List workflows first, use numeric ID |
| 422 Unprocessable | Wrong ref or missing required params | Always pass `{"ref": "develop"}` |
| 403 Forbidden | Token lacks workflow scope | Add `workflow` scope to classic PAT |
| Token not found in env | Using `/tmp/.git_token` which was cleared | Store in `.env`, read from there |

## Quick Token Refresh Checklist
1. Generate new classic PAT at github.com/settings/tokens/new → `repo` + `workflow`
2. Validate via `/user` endpoint (200 OK)
3. Update `/opt/data/.env` via Python
4. Test trigger with API (list → frontend dispatch)
5. Confirm run appears in `gh run list` (if gh available) or API

### Docker Update Recovery Checklist
If Docker update breaks GitHub auth again:
1. Generate new **Classic PAT** (`ghp_...`) with `repo` + `workflow` scopes
2. Validate: `curl -H "Authorization: token <PAT>" https://api.github.com/user` → 200 OK
3. Update `/opt/data/.env`:
   ```python
   with open("/opt/data/.env", "r") as f:
       lines = f.readlines()
   with open("/opt/data/.env", "w") as f:
       for l in lines:
           f.write(f"GITHUB_TOKEN={new_token}\n" if l.startswith("GITHUB_TOKEN=") else l)
   ```
4. Also update `/opt/data/profiles/nook/.env` if using nook profile
5. Update `config.yaml` → `mcp_servers.github.env.GITHUB_TOKEN`
6. Test API call to list workflows