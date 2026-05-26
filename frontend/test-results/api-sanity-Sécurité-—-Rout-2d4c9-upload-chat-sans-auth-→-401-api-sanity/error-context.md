# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: api-sanity.spec.ts >> Sécurité — Routes non-auth → 401 >> POST /api/upload/chat sans auth → 401
- Location: tests/api-sanity.spec.ts:93:3

# Error details

```
Error: apiRequestContext.post: connect ECONNREFUSED ::1:6300
Call log:
  - → POST http://localhost:6300/api/upload/chat
    - user-agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.7727.15 Safari/537.36
    - accept: */*
    - accept-encoding: gzip,deflate,br
    - content-type: multipart/form-data; boundary=----WebKitFormBoundaryBI724Rq9ZJJ7uKL3
    - content-length: 296

```