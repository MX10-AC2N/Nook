---
name: nook-security-audit
category: "software-development"
description: "Security audit of Nook codebase — OWASP Top 10, dependencies, API security. Produces report in .hermes/SECURITY-REPORT.md"
---

# 🔒 Security Audit Skill

## Trigger
Run when:
- "Security audit", "check vulnerabilities"
- Before release
- After dependency updates
- User reports security concerns

## Steps

### 1. Check dependencies
```bash
# Rust
cd backend && cargo audit

# Node
cd frontend && npm audit
```

### 2. Scan code for patterns
```bash
# SQL injection
search_files("SELECT.*\$\{|INSERT.*\$\{|UPDATE.*\$\{|DELETE.*\$\{")

# XSS
search_files("innerHTML|dangerouslySetInnerHTML|eval\(")

# Secrets
search_files("password.*=|secret.*=|token.*=|api_key.*=")

# Hardcoded credentials
search_files("ghp_|sk-|AKIA")
```

### 3. Check Docker security
```bash
# Check USER directive
search_files("USER nook|adduser -S -u 1000")

# Check exposed ports
search_files("EXPOSE")
```

### 4. Check API security
- Auth required on all endpoints
- Input validation
- Rate limiting
- CORS configured

### 5. Produce report
Save to `.hermes/SECURITY-REPORT.md` and push to GitHub.

## Report template
```markdown
# 🔒 Rapport Sécurité — Nook [Date]

## Score : [X/100]

## Vulnérabilités
| CVE | Package | Severity | Fix |
|-----|---------|----------|-----|
| [cve]| [pkg]   | [sev]    | [fix]|

## Code scan
- [✅/❌] No SQL injection
- [✅/❌] No XSS
- [✅/❌] No hardcoded secrets
- [✅/❌] Input validation

## Docker
- [✅/❌] Non-root user
- [✅/❌] Minimal ports
- [✅/❌] No Google deps

## API
- [✅/❌] Auth on all endpoints
- [✅/❌] Rate limiting
- [✅/❌] CORS configured
```
