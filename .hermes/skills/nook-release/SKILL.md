---
name: nook-release
category: "devops"
description: "Release workflow — version bump, changelog, tag, Docker build, deploy for Nook."
---

# 🚀 Release Skill

## Trigger
- "Create release", "ship v0.X.Y", "deploy"
- User wants to publish a new version

## Steps
1. Verify: all tests pass, build OK
2. Changelog: summarize changes since last tag
3. Version bump in Cargo.toml / package.json
4. Tag: `git tag -a v0.X.Y -m "Release v0.X.Y"`
5. Push tag: `git push origin v0.X.Y`
6. GitHub Release: `gh release create v0.X.Y --notes-file CHANGELOG.md`
7. Docker: tag + push images
8. Deploy: docker compose pull && docker compose restart

## Checklist
- [ ] Tests pass
- [ ] Build pass
- [ ] Changelog written
- [ ] Version bumped
- [ ] Tagged
- [ ] Docker built
- [ ] Deployed
- [ ] Smoke test

## Report
Save to `.claude/RELEASE-REPORT.md` and push.
