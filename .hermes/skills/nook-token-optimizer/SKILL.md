---
name: nook-token-optimizer
category: "devops"
description: "Optimize token usage and reduce costs during Hermes interactions. Focus on concise responses, combined tool calls, and minimal context."
---

# 💰 Token Optimizer Skill

## Trigger
Run when:
- User says "optimize", "reduce tokens", "be concise"
- Session uses > 10k tokens
- Responses are verbose
- Multiple tool calls for same task

## Patterns

### 1. Concise responses
- No preambles ("Voici...", "Je vais...")
- Tables instead of lists
- Bullet points instead of paragraphs
- French only (no translation)

### 2. Combined tool calls
```python
# Bad: 3 separate calls
terminal("cd frontend && npm run build")
terminal("cd frontend && npm run lint")
terminal("npx playwright test --list")

# Good: 1 combined call
terminal("cd frontend && npm run build && npm run lint && npx playwright test --list")
```

### 3. Minimal context
```python
# Bad: read entire file
content = read_file("file.ts")

# Good: read relevant lines
content = read_file("file.ts", offset=100, limit=50)
```

### 4. Batch operations
```python
# Bad: 3 separate commits
git commit -m "fix1"
git commit -m "fix2"
git commit -m "fix3"

# Good: 1 batch commit
git add -A && git commit -m "fix: all three issues"
```

## Metrics
- **Target**: < 5k tokens per exchange
- **Tool calls**: < 5 per task
- **Response**: < 150 words
- **Redundancy**: < 5%

## Checklist
1. Is response concise?
2. Are tool calls combined?
3. Is context minimal?
4. Is there redundancy?
5. Can anything be batched?
