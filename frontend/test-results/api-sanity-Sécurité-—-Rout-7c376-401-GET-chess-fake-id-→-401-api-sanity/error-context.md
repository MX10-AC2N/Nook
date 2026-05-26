# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: api-sanity.spec.ts >> Sécurité — Routes non-auth → 401 >> GET /chess/fake-id → 401
- Location: tests/api-sanity.spec.ts:83:5

# Error details

```
Error: apiRequestContext.get: connect ECONNREFUSED ::1:6300
Call log:
  - → GET http://localhost:6300/api/chess/fake-id
    - user-agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.7727.15 Safari/537.36
    - accept: */*
    - accept-encoding: gzip,deflate,br

```