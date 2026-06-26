---
name: x-twitter-cli
description: Complete X/Twitter CLI toolkit -- x-cli (xitter) and xurl (official X API CLI) for posting, reading, searching, DMs, media, and account operations.
version: 2.0.0
tags: [x, twitter, cli, api, posting, timeline, search, dms, media, bookmarks, followers]
related_skills: []
---

# X/Twitter CLI Toolkit

Unified interface for X/Twitter operations using two complementary CLIs: x-cli (xitter) for interactive terminal use, and xurl (official X API CLI) for scriptable API access.

## Quick Navigation

| CLI | Package | Best For | Auth |
|-----|---------|----------|------|
| x-cli (xitter) | x-cli | Interactive use, timelines, search, bookmarks | X API credentials |
| xurl | xurl | Scripting, automation, raw API access, media upload | Official X API credentials |

---

## 1. x-cli (xitter)

### Install & Auth

```bash
# Install
cargo install x-cli
# or download binary from GitHub releases

# Auth: Run interactive setup
x login
# Enter API Key, API Secret, Access Token, Access Token Secret
# Credentials stored in ~/.config/x-cli/config.toml
```

### Core Commands

```bash
# Post a tweet
x post "Hello from Hermes Agent!"

# Post with media
x post "Check this out" --media ./image.png

# Reply to a tweet
x reply 1234567890 "Thanks for the mention!"

# Quote tweet
x quote 1234567890 "My thoughts on this..."

# Delete a tweet
x delete 1234567890
```

### Reading & Search

```bash
# Home timeline
x timeline

# User timeline
x timeline @username

# Mentions
x mentions

# Search tweets
x search "rust lang" --limit 20

# Search with filters
x search "from:username since:2024-01-01 until:2024-12-31"

# View single tweet
x show 1234567890
```

### Interactions

```bash
# Like/unlike
x like 1234567890
x unlike 1234567890

# Retweet/unretweet
x retweet 1234567890
x unretweet 1234567890

# Bookmark
x bookmark 1234567890
x bookmarks  # List bookmarks
```

### User Operations

```bash
# User lookup
x user @username

# Followers/following
x followers @username
x following @username

# Follow/unfollow
x follow @username
x unfollow @username

# Lists
x lists @username
x list-members "list-slug" @username
```

---

## 2. xurl (Official X API CLI)

### Install & Auth

```bash
# Install
go install github.com/xurl/xurl@latest
# or download binary

# Auth: Set environment variables (see references/xurl-env-setup.md for details)
# Required: X_API_KEY, X_API_SECRET, X_ACCESS_TOKEN, X_ACCESS_TOKEN_SECRET
# For v2 write: X_BEARER_TOKEN
```

### Core Commands (v2 API)

```bash
# Post tweet (v2)
xurl post "Hello from xurl!"

# Post with media
xurl media-upload ./image.png
xurl post "With image" --media-id <media_id_from_upload>

# Reply
xurl post "Reply text" --reply-to 1234567890

# Quote
xurl post "Quote text" --quote-tweet-id 1234567890

# Delete
xurl delete 1234567890
```

### Reading & Search (v2)

```bash
# Home timeline
xurl timeline

# User timeline
xurl timeline --user username

# Mentions
xurl mentions

# Search recent (v2)
xurl search "rust lang" --max-results 20

# Search all (academic/research access)
xurl search "rust lang" --all --start-time 2024-01-01 --end-time 2024-12-31

# View tweet
xurl show 1234567890
```

### Interactions

```bash
# Like
xurl like 1234567890
xurl unlike 1234567890

# Retweet
xurl retweet 1234567890
xurl unretweet 1234567890

# Bookmarks
xurl bookmarks
xurl bookmark 1234567890
xurl remove-bookmark 1234567890
```

### User & Account Operations

```bash
# User lookup
xurl user @username
xurl user --id 12345678

# Followers/following
xurl followers @username
xurl following @username

# Follow/unfollow
xurl follow @username
xurl unfollow @username

# My account
xurl me

# Lists
xurl lists @username
xurl list-members --list-id 1234567890
```

### Advanced: Raw v2 Endpoint Access

```bash
# Direct API calls
xurl get /2/users/me
xurl post /2/tweets --data '{"text":"Direct API call"}'
xurl get /2/tweets/1234567890 --params "tweet.fields=created_at,public_metrics"

# Pagination
xurl get /2/users/1234567890/tweets --params "max_results=100&pagination_token=next_token"
```

### Media Upload (Chunked)

```bash
# Upload video/large images
xurl media-upload ./video.mp4 --chunked
# Returns media_id for use in post
```

### DMs (Direct Messages)

```bash
# List conversations
xurl dm-conversations

# List messages in conversation
xurl dm-messages --conversation-id 1234567890

# Send DM
xurl dm-send --recipient @username --text "Hello!"

# Send DM with media
xurl media-upload ./image.png
xurl dm-send --recipient @username --text "See this" --media-id <media_id>
```

---

## Comparison: When to Use Which

| Task | Use x-cli (xitter) | Use xurl |
|------|-------------------|----------|
| Interactive timeline browsing | Yes | No |
| Quick one-off posts | Yes | Yes |
| Scripted automation | No | Yes |
| Media upload (large/video) | Limited | Yes (chunked) |
| Raw v2 API access | No | Yes |
| DM management | No | Yes (full) |
| Bookmarks | Yes | Yes |
| Search (recent) | Yes | Yes |
| Search (full archive) | No | Yes (academic) |
| User analytics/metrics | No | Yes (public_metrics) |

---

## Common Workflows

### Daily Posting (Interactive)

```bash
# Quick post
x post "Morning thoughts on $(date)"

# With image
x post "Today's dashboard" --media ./screenshot.png
```

### Automated Posting (Scripted)

```bash
#!/bin/bash
# auto-post.sh
CONTENT=$(generate_content_today)
MEDIA_ID=$(xurl media-upload ./daily-chart.png --quiet)
xurl post "$CONTENT" --media-id "$MEDIA_ID"
```

### Monitoring & Engagement

```bash
# Check mentions (interactive)
x mentions

# Check mentions (scripted)
xurl mentions --max-results 50 | jq '.data[] | {id, text, author_id}'

# Reply to all unread mentions
xurl mentions --max-results 20 | jq -r '.data[] | select(.referenced_tweets[].type=="replied_to" | not) | .id' | \
  while read id; do xurl post "Thanks!" --reply-to "$id"; done
```

### Media Management

```bash
# Upload multiple images for a thread
MEDIA_IDS=()
for img in ./thread-images/*.png; do
  ID=$(xurl media-upload "$img" --quiet)
  MEDIA_IDS+=("$ID")
done

# Post thread
xurl post "Thread 1/n" --media-id "${MEDIA_IDS[0]}"
PREV_ID=$(xurl post "Thread 2/n" --media-id "${MEDIA_IDS[1]}" --quiet | jq -r .data.id)
# ... continue thread
```

---

## Authentication Details

### x-cli (xitter)

Stores credentials in ~/.config/x-cli/config.toml:
```toml
[default]
api_key = "xxx"
api_secret = "xxx"
access_token = "xxx"
access_token_secret = "xxx"
```

Multiple profiles supported:
```bash
x login --profile work
x post "Work tweet" --profile work
```

### xurl

Uses environment variables (standard for CLI tools). See references/xurl-env-setup.md for complete setup.

---

## Rate Limits & Best Practices

| Endpoint | Limit (15-min window) |
|----------|----------------------|
| POST /2/tweets | 300 |
| GET /2/tweets/search/recent | 450 |
| GET /2/users/:id/tweets | 900 |
| POST /2/users/:id/likes | 500 |
| POST /1.1/media/upload | 50 |

**Best practices:**
- Cache user lookups
- Use pagination for large result sets
- Respect x-rate-limit-reset headers
- Batch media uploads
- Use --quiet for scripted output parsing

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| 401 Unauthorized | Check all 4 credentials + bearer token |
| 403 Forbidden | Verify app permissions (Read+Write+DM) |
| 429 Too Many Requests | Wait for rate limit reset, add delays |
| Media upload failed | Check file size (<512MB), format support |
| DM send failed | Recipient must allow DMs from everyone or follow you |
| xurl: command not found | Add ~/go/bin to PATH |

---

## Reference Files

| File | Purpose |
|------|---------|
| references/x-api-endpoints.md | Complete v2 API endpoint reference |
| references/x-cli-config.md | x-cli configuration patterns |
| references/xurl-env-setup.md | xurl environment setup details |

---

## When to Use This Skill

- Posting tweets (interactive or automated)
- Reading timelines, mentions, search results
- Managing likes, retweets, bookmarks
- User lookup, follow/unfollow operations
- DM conversations (xurl only)
- Media upload including large videos (xurl only)
- Raw v2 API access for advanced use cases
- Building X/Twitter automation workflows