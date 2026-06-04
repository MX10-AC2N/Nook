---
name: nook-user-support
description: Skill for the User Support agent - FAQ, troubleshooting guides, user onboarding, bug report triage
---

# Nook User Support Skill

Use this skill when creating user-facing documentation, troubleshooting guides, FAQ, or triaging user bug reports.

## Responsibilities
- Create user-facing guides (getting started, WebRTC setup, E2EE explanation)
- Maintain FAQ for common issues (WebRTC connection fails, login issues)
- Triage bug reports from users, reproduce issues
- Write onboarding guides for new users
- Document known issues and workarounds
- Create video tutorials (optional)

## Tools Required
- `markdown` (for writing guides)
- `screenshots` (from `docs/screenshots/`)
- `browser` (for reproducing user issues)
- `gh` CLI (for triaging GitHub issues)

## Common User Issues
1. **WebRTC Call Fails**:
   - Check TURN server running, UDP 3478 open
   - Verify ICE config endpoint returns credentials
   - Check browser permissions for camera/microphone
   - Solution guide: `docs/troubleshooting/webrtc-calls.md`

2. **Login Issues**:
   - Forgot password (no reset flow yet, admin must reset)
   - E2EE password lost (messages inaccessible, by design)
   - Solution guide: `docs/troubleshooting/login-issues.md`

3. **Calendar Not Saving Events**:
   - Missing `events.rs` backend (known P0 issue)
   - Temporary workaround: Use polls for event scheduling
   - Solution: Wait for backend implementation

4. **PWA Not Installing**:
   - Check HTTPS enabled (required for PWA)
   - Verify `manifest.json` and `service-worker.js` present
   - Solution guide: `docs/troubleshooting/pwa-install.md`

## FAQ Template
```markdown
## FAQ

### Q: How do I start a WebRTC call?
A: Navigate to a conversation, click the video/phone icon. Ensure your browser has camera/microphone permissions. For remote users, ensure TURN server is configured.

### Q: Is my data encrypted?
A: Yes, all messages/files are E2EE (XChaCha20-Poly1305). Only you and the recipient have the keys. Admins cannot read encrypted content.
```

## User Guide Structure
1. **Getting Started**:
   - Create account, first login
   - Set up E2EE password
   - Update profile/avatar

2. **Core Features**:
   - Chat, send messages/files
   - Start WebRTC audio/video calls
   - Create polls, view calendar
   - Play chess with AI or users

3. **Advanced**:
   - E2EE how it works
   - PWA installation
   - Admin features (user management)

## Pitfalls
- E2EE password loss is irreversible (no recovery)
- WebRTC requires HTTPS or local network (no HTTP remote access)
- Calendar feature is non-functional (P0 issue, document workaround)
- Polls close manually or via `closes_at` (no auto-close yet)

## Verification
- [ ] FAQ covers top 10 user issues
- [ ] Troubleshooting guides have step-by-step solutions
- [ ] Onboarding guide takes <5 minutes to complete
- [ ] All user-facing docs use simple, non-technical language
