# Nook-Specific Runtime Pitfalls

Common bugs found during manual browser testing of the Nook application (http://192.168.1.192:6300).

## Session & Auth

### Session Lost on Navigation
- **Symptom**: Clicking any in-app nav link (Chess, Calendar, Polls, Settings) redirects to /login
- **Symptom**: Direct URL navigation to /calendar, /chess, /polls also redirects to /login
- **Symptom**: Page reload also redirects to /login
- **Root cause**: Auth token stored in memory only, not in localStorage or a persistent cookie. Session is also lost on page reload.
- **Severity**: Critical
- **Workaround for testing**: Re-login, test ONE feature per session, document, move on. Do NOT re-click the same link after redirect — each click burns a login/re-navigate cycle.
- **Status**: Known issue — auth persistence across navigation not implemented

## Chat

### Send Button Disabled Despite Input
- **Symptom**: "Envoyer" button stays `disabled` even with text in the message input
- **Symptom**: Clicking the button (when somehow enabled) clears input without sending
- **Root cause**: Likely a Svelte event binding issue — the input's state change isn't properly connected to the button's disabled state and send handler
- **Expected**: Button enables when non-empty text is typed; click sends message via PUT API
- **Severity**: Critical
- **Investigation**: Check `chat/+page.svelte` — likely `$state`/`$effect` not properly wired, or the button's `disabled` attribute is bound to a stale computed value

### E2EE Key Persistence Broken
- **Symptom**: After page reload, messages show "Message chiffré (clé indisponible)" or raw base64 ciphertext
- **Symptom**: Previously sent messages that were visible in clear become unreadable
- **Console evidence**: `[cryptoStore] loadKeysFromIndexedDB result: NO_KEYS_FOUND for userId: xxx` followed by `génération initiale E2EE` on every session
- **Console evidence**: `Erreur déchiffrement message: "incorrect key pair for the given ciphertext"` — retries 38+ times in infinite loop
- **Root cause**: cryptoStore generates new keys each session because IndexedDB doesn't store them. Messages encrypted with old public key can't be decrypted with new private key.
- **Loop bug**: Code retries decryption 38+ times in infinite loop — wastes CPU, should cap retries and display fallback after failure
- **Expected**: Keys persisted in indexedDB on generation, reloaded on session init
- **Severity**: Critical — breaks core value proposition
- **Fix approach**: Trace `cryptoStore.ts` — verify `saveKeysToIndexedDB` is called after key generation, verify IndexedDB transactions complete, verify key is loaded before decryption starts

## Calendar

### Events Fail to Load
- **Symptom**: "Impossible de charger les événements" error on calendar page
- **Root cause**: API endpoint for calendar events returns error or is unavailable
- **Note**: Calendar UI renders correctly (month grid, date buttons, navigation). Add event dialog opens. Only the event list fails to populate.
- **Status**: Known issue — calendar event API endpoint broken or missing data

## Testing Workflow

### Recommended Test Order for Nook
1. Login → verify session starts (don't reload yet)
2. Navigate to each feature from within the same session
3. For each feature: check UI renders, check for JS errors, check for loading failures
4. Test one state-changing action per feature (send message, create event, etc.)
5. THEN test persistence: reload page, verify session survives, verify state persists

### Known Credentials
- Username: `hermes-bot`
- Password: `Hermes2026!`
