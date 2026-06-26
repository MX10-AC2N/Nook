# HSTS Header in Reverse Proxy Setup

## Problem

HSTS (`Strict-Transport-Security`) header MUST only be sent on HTTPS responses. When backend runs behind a reverse proxy (nginx) that terminates TLS:

```
Client (HTTPS) -> Nginx (SSL termination) -> Backend (HTTP)
```

If backend unconditionally sends HSTS on HTTP, browser:
1. Receives HSTS on HTTP -> caches HSTS policy
2. Forces all future requests to HTTPS
3. Hits nginx with self-signed cert -> certificate error -> blank page

## Solution

Backend checks `x-forwarded-proto` header (set by nginx) before sending HSTS.

## Nginx Required Config

Nginx must set `X-Forwarded-Proto` header:

```
proxy_set_header X-Forwarded-Proto https;
```

(Full nginx config in docs/nginx-local.md)

## Test Verification

- HTTP to backend port: NO HSTS header
- HTTPS via nginx: HSTS header present

## Related Skill

See `nook-rust-backend/references/hsts-reverse-proxy.md` for backend implementation.