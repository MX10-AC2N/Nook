---
name: nook-debugging-message-flow
description: Diagnostiquer les bugs du flux de messages Nook — E2EE, WebSocket, fallback, états UI.
---
# nook-debugging-message-flow

## Bug Pattern #1 — crypto non prêt
Symptôme: déchiffrement impossible, sessionStorage clé invalide.
Vérifier:
- unlockCrypto ready-guard avant decrypt
- sessionStorage key validation (regex/non vide)
- key registration race (Promise bridge + activeConvId guard)

## Bug Pattern #2 — état conversation vide
Symptôme: messages ne s'affichent pas après unlock.
Vérifier:
- activeConvId est bien défini avant subscribe
- API error logging (pas de 401/403 silencieux)
- emoji-picker registered avant interaction

## Bug Pattern #3 — double message / doublon
Vérifier:
- idempotency key côté backend
- duplicate filter dans le store frontend (Set/Map par messageId)

## Références
- nook-frontend-common-patterns
- codegraph-integration pour tracer les callers/callees
