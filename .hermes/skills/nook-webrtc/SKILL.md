---
name: nook-webrtc
category: "devops"
description: "WebRTC audio/video calls — testing, debugging, TURN server integration for Nook."
---

# 📹 WebRTC Skill

## Trigger
- "Test calls", "debug WebRTC", "audio not working"
- Before/after call feature changes
- TURN server issues

## Steps
1. Check TURN: `curl -v http://localhost:3478`
2. Check logs: `docker compose logs turn`
3. Test in browser: Navigate to chat, initiate call
4. Check console: `browser_console()` for WebRTC errors
5. Check ICE: `pc.iceConnectionState` in browser console
6. Check stats: `pc.getStats()` for quality metrics

## Diagnostic checklist
- [ ] TURN server running
- [ ] Port 3478 open (UDP+TCP)
- [ ] Browser permissions granted
- [ ] ICE gathering complete
- [ ] Connection established

## Report
Save to `.claude/WEBRTC-REPORT.md` and push.
