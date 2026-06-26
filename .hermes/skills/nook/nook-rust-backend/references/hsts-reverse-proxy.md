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

Check `x-forwarded-proto` header (set by nginx) before sending HSTS:

```rust
// src/main.rs - security headers middleware
.layer(middleware::from_fn(|req: Request<Body>, next: Next| async move {
    let mut response = next.run(req).await;
    let headers = response.headers_mut();
    // ... other security headers ...

    // HSTS only on HTTPS (nginx terminates TLS and forwards x-forwarded-proto: https)
    if req.headers().get("x-forwarded-proto").and_then(|v| v.to_str().ok()) == Some("https") {
        headers.insert("Strict-Transport-Security", "max-age=31536000; includeSubDomains".parse().unwrap());
    }
    response
}))
```

## Nginx Required Config

Nginx must set `X-Forwarded-Proto` header:

```
proxy_set_header X-Forwarded-Proto https;
```

(Full nginx config in docs/nginx-local.md)

## Test Verification

- HTTP to backend port: NO HSTS header
- HTTPS via nginx: HSTS header present

## References

- OWASP HSTS Cheat Sheet
- MDN: Strict-Transport-Security