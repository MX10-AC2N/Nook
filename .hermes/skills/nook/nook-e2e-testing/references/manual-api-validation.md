# Nook — Manual API Validation Quick Reference

*Created Session 55 — validated against http://192.168.1.192:6300*

## Login & Session

```bash
# Login
curl -s -c cookies.txt -X POST http://IP:6300/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"hermes-bot","password":"Hermes2026!"}'

# Check session
curl -s -b cookies.txt http://IP:6300/api/auth/me

# Logout
curl -s -b cookies.txt -X POST http://IP:6300/api/auth/logout
```

## Conversations & Messages

```bash
# List conversations
curl -s -b cookies.txt http://IP:6300/api/conversations

# Get messages (E2EE encrypted)
curl -s -b cookies.txt http://IP:6300/api/conversations/default_global/messages

# Send message
curl -s -b cookies.txt -X POST http://IP:6300/api/conversations/default_global/messages \
  -H "Content-Type: application/json" \
  -d '{"content":"Test message"}'

# Add reaction
curl -s -b cookies.txt -X POST "http://IP:6300/api/conversations/default_global/messages/MSG_ID/reactions" \
  -H "Content-Type: application/json" -d '{"emoji":"👍"}'

# Create conversation
curl -s -b cookies.txt -X POST http://IP:6300/api/conversations \
  -H "Content-Type: application/json" -d '{"name":"My Group"}'

# Add participant
curl -s -b cookies.txt -X POST "http://IP:6300/api/conversations/CONV_ID/participants" \
  -H "Content-Type: application/json" -d '{"user_id":"USER_ID"}'

# Rename conversation
curl -s -b cookies.txt -X PATCH "http://IP:6300/api/conversations/CONV_ID/rename" \
  -H "Content-Type: application/json" -d '{"name":"New Name"}'

# Leave conversation
curl -s -b cookies.txt -X POST "http://IP:6300/api/conversations/CONV_ID/leave"
```

## Polls

```bash
# Create poll
curl -s -b cookies.txt -X POST http://IP:6300/api/polls \
  -H "Content-Type: application/json" \
  -d '{"question":"Test?","options":["A","B","C"]}'

# Vote
curl -s -b cookies.txt -X POST "http://IP:6300/api/polls/POLL_ID/vote" \
  -H "Content-Type: application/json" -d '{"option_id":"OPTION_ID"}'
```

## Chess

```bash
# Create game vs AI
curl -s -b cookies.txt -X POST http://IP:6300/api/chess/create \
  -H "Content-Type: application/json" -d '{"opponent":"easy","color":"white"}'

# Make move
curl -s -b cookies.txt -X POST "http://IP:6300/api/chess/GAME_ID/move" \
  -H "Content-Type: application/json" -d '{"from":"e2","to":"e4"}'
```

## Events (BUGGED — see nook-rust-backend)

```bash
# Create event (works)
curl -s -b cookies.txt -X POST http://IP:6300/api/events \
  -H "Content-Type: application/json" \
  -d '{"title":"Event","date":"2026-06-15","start_time":"10:00","end_time":"11:00"}'

# List events (BROKEN — Missing Pool<Sqlite> injection)
curl -s -b cookies.txt http://IP:6300/api/events
```

## Push Notifications

```bash
# VAPID public key (public)
curl -s http://IP:6300/api/push/vapid-public-key

# User preferences
curl -s -b cookies.txt http://IP:6300/api/push/preferences
```

## Upload/Download

```bash
# Upload file
curl -s -b cookies.txt -X POST http://IP:6300/api/upload/chat \
  -F "file=@/path/to/file" -F "conversation_id=default_global"

# Download file (from returned URL)
curl -s -b cookies.txt http://IP:6300/api/download/FILE_ID
```

## E2EE Keys

```bash
# Register public key
curl -s -b cookies.txt -X POST http://IP:6300/api/auth/public-key \
  -H "Content-Type: application/json" \
  -d '{"public_key":"BASE64_PUBLIC_KEY"}'

# Get conversation keys
curl -s -b cookies.txt 'http://IP:6300/api/auth/public-keys?conversation_id=default_global'
```

## User Profile & Settings

```bash
# Update name
curl -s -b cookies.txt -X POST http://IP:6300/api/user/update \
  -H "Content-Type: application/json" -d '{"name":"New Name"}'

# Available users
curl -s -b cookies.txt http://IP:6300/api/users/available
```

## Health & Public

```bash
# Health check
curl -s http://IP:6300/api/health

# VAPID public key (no auth)
curl -s http://IP:6300/api/push/vapid-public-key
```

## Common Issues

| Symptom | Cause | Fix |
|---------|-------|-----|
| 401 on all routes | Cookie expired/wrong | Re-login |
| 429 on login | Rate limit (5 req/min) | Wait or use different user |
| Events GET 500 | Pool<Sqlite> not injected | See nook-rust-backend BUG section |
| Empty page in browser | SvelteKit hydration failed | Check JS console, use NOOK_ENV=development |
| `ReferenceError: BASE` in tests | Missing import | `import { BASE } from './helpers'` |
| Tests hit localhost instead of remote | NOOK_BASE_URL not set | `export NOOK_BASE_URL=http://IP:6300` |

## Test Users (from DB)

| Username | Password | Role | Notes |
|----------|----------|------|-------|
| hermes-bot | Hermes2026! | user | ✅ Works |
| e2e_ci | E2eTest123! | user | Rate limited |
| admin | AdminCI2026! | admin | Password may have changed |
| admin | changeme2026 | admin | Initial password |

## Quick Validation Script

```bash
#!/bin/bash
IP="192.168.1.192:6300"
curl -s -c /tmp/c.txt -X POST http://$IP/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"hermes-bot","password":"Hermes2026!"}' | jq .
curl -s -b /tmp/c.txt http://$IP/api/auth/me | jq .
curl -s -b /tmp/c.txt http://$IP/api/conversations | jq .
curl -s -b /tmp/c.txt http://IP:6300/api/health
```