---
name: nook-notifications
category: "devops"
description: "In-app notification system for HTTP/LAN environments where Web Push (HTTPS) is unavailable. Includes toast UI, sound via AudioContext, and per-module integration."
---

# 🔔 Notification System — Nook

## Architecture
- **Store**: `frontend/src/lib/notificationStore.svelte.ts`
- **Component**: `frontend/src/lib/components/NotificationToast.svelte`
- **Sound**: AudioContext (requires user interaction to initialize)
- **Display**: Toast + history dropdown + title badge

## Available Functions
```typescript
notify(type, title, body)                    // Generic notification
notifyMessage(title, body)                   // Chat messages
notifyChess(title, body)                     // Chess moves
notifyPoll(title, body)                      // Poll creation
notifyCalendar(title, body)                  // Calendar events
notifyCall(title, body)                      // WebRTC calls
notifyAdmin(title, body)                     // Admin actions
```

## Integration Pattern
```svelte
<script lang="ts">
  import { notifyXxx } from '$lib/notificationStore.svelte';
  
  async function handleSomething() {
    // ... do something
    notifyXxx('Success', 'Action completed');
  }
</script>
```

## Common Issues
1. **notifyXxx is not defined** → Missing import in component
2. **No sound** → AudioContext not initialized (needs user interaction)
3. **Web Push not working** → Requires HTTPS (not available on HTTP/LAN)
4. **Toast not visible** → Check z-index and CSS

## Checklist
- [ ] Import `notifyXxx` from `$lib/notificationStore.svelte`
- [ ] Call in appropriate context (send message, create poll, etc.)
- [ ] Test on HTTP/LAN (AudioContext)
- [ ] Verify CSS (z-index, position)
