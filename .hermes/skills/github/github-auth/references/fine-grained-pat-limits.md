# Fine-Grained PAT Limitations — Detailed Findings

## What Happened (Nook repo, May 2026)

Agent had 2 local commits ready to push to `develop` but could not authenticate
for a git push or any API write operation.

## Error Transcripts

### HTTPS git push
```
remote: Invalid username or token. Password authentication is not supported for Git operations.
fatal: Authentication failed for 'https://github.com/MX10-AC2N/Nook.git/'
```
Fine-grained PATs cannot be used as HTTPS passwords. Only classic PATs work.

### Git credential helper (via PAT)
```
remote: Permission to MX10-AC2N/Nook.git denied to MX10-AC2N.
fatal: unable to access 'https://github.com/MX10-AC2N/Nook.git/': 403
```
Even with credentials properly configured, the token is rejected for push.

### Git Data API (/git/blobs, /git/trees, /git/commits)
```
API Error 403: {"message":"Resource not accessible by personal access token","documentation_url":"https://docs.github.com/rest/git/blobs#create-a-blob","status":"403"}
```
The Git Data API does not support fine-grained PATs at all, regardless of repo permissions.

### Contents API (/repos/.../contents/... PUT)
```
API Error 403: {"message":"Resource not accessible by personal access token","documentation_url":"https://docs.github.com/rest/repos/contents#create-or-update-file-contents","status":"403"}
```
This endpoint requires `Contents: Read and Write` in the fine-grained PAT settings.
If the PAT only has `Contents: Read`, GET works but PUT fails with 403.

## What DID work

- GET requests on the API (read ops) — user identity, repo info, file contents, refs
- `curl -s -H "Authorization: Bearer $TOKEN" https://api.github.com/user` → 200
- Reading `/git/refs/heads/develop` → 200 (returns the current SHA)
- Reading `/repos/MX10-AC2N/Nook/contents/<path>` → 200

## Root Cause

The token `github_pat_11AO5547I...` is a fine-grained PAT (prefix `github_pat_`). It had:
- Repository permissions: `Contents: Read` (not `Read and Write`)
- No permission to use Git Data API endpoints

## Fixes to tell the user

1. **Fastest**: user runs `git push origin develop` from their own machine (SSH or stored classic PAT)
2. **For agent access**: generate a classic PAT with `repo` + `workflow` scopes
3. **If keeping fine-grained**: upgrade `Contents` to `Read and Write` — this fixes
   Contents API writes but still won't enable Git Data API or HTTPS git push
