# 🧪 Rapport E2E ciblé — Nook

Généré automatiquement par `e2e-targeted.yml`
**Dernière mise à jour : 2026-08-29 08:22 UTC**

---

## Résumé

| Champ | Valeur |
|-------|--------|
| **Statut** | ❌ ÉCHEC |
| **Suite lancée** | 🔐 Auth — login, logout, /auth/me |
| **Filtre Playwright** | `Auth` |
| **Tests passés** | 5 |
| **Tests échoués** | 69 |
| **Tests flaky** |  |
| **Traces activées** | false |
| **Branche** | `develop` |
| **Commit** | [`3fe3c4e`](https://github.com/MX10-AC2N/Nook/commit/3fe3c4e25a0ec221e8dc81fc1f6fa77e2db6993f) |
| **Run CI** | [Voir le run complet](https://github.com/MX10-AC2N/Nook/actions/runs/33242677228) |

---

## Résultats par test

```
✘  128 [api-sanity] › tests/api-sanity.spec.ts:392:3 › Sécurité — Change password autre user → 403 (fix C1) › User change pwd autre user → 403 (integration) (621ms)
  ✘  129 [api-sanity] › tests/api-sanity.spec.ts:392:3 › Sécurité — Change password autre user → 403 (fix C1) › User change pwd autre user → 403 (integration) (retry #1) (639ms)
        npx playwright show-trace test-results/api-sanity-Sanité-—-Serveur-GET-api-health-→-OK--api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sanité-—-Serveu-03c0a-ublique-pas-d-auth-requise--api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-48605-uth-→-401-GET-auth-me-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-63889--401-POST-auth-logout-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-d3308--auth-change-password-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-6bb32--POST-auth-public-key-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-06e39-ion-id-default-global-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-391b2-401-GET-conversations-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-e0aed-01-POST-conversations-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-ce22c-ations-default-global-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-3964a-fault-global-messages-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-b6bcc-fault-global-messages-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-7f375-t-global-participants-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-12751-t-global-participants-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-9f2f0--default-global-leave-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-cd729-default-global-rename-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-864a6--download-fake-id-000-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Routes-non-auth-→-401-GET-events-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-823a4-uth-→-401-POST-events-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Routes-non-auth-→-401-GET-polls-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Routes-non-auth-→-401-POST-polls-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-0ac7d-401-GET-polls-fake-id-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-e93b9-ST-polls-fake-id-vote-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-66ee2-T-polls-fake-id-close-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-2a5ec--DELETE-polls-fake-id-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-49c8b--→-401-GET-chess-list-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-0cf08-401-POST-chess-create-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-be7a5-GET-chess-invitations-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-7c376-401-GET-chess-fake-id-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-30dbd-ST-chess-fake-id-move-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-70741-fake-id-moves-from-e2-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-423fa-chess-fake-id-ai-move-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-175f4--chess-fake-id-resign-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-bf785--messages-x-reactions-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-72cab--messages-x-reactions-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-f7cce--messages-x-reactions-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-3d29e--401-POST-user-update-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-0d323-1-GET-users-available-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-844f9--GET-push-preferences-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-2d4c9-upload-chat-sans-auth-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-ef16a-uth-GET-users-pending-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-203fd-1-sans-auth-GET-users-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-07ed6-th-POST-users-approve-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-5f42b-sans-auth-GET-invites-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-74153-ans-auth-POST-invites-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-35fd8-h-POST-invites-delete-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Rout-19634-ns-auth-GET-analytics-→-401-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Mot--2d5ec-é-Mot-de-passe-1-char-→-400-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Mot--2058c--Mot-de-passe-5-chars-→-400-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Mot--6a571--Mot-de-passe-7-chars-→-400-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Mot--131cf--de-passe-8-chars-→-accepte-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Chan-9d71b-change-pwd-autre-user-→-403-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Uplo-6d365-n-Upload-fichier-vide-→-400-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Uplo-7ddb6--→-file-id-puis-download-OK-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Uplo-4df3a-ad-fichier-inexistant-→-404-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Mess-79bc5-écupérer-→-contient-message-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Mess-375a3-D-Rename-conversation-→-200-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Call-9fcb6--auth-→-redirige-vers-login-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Ches-9006e-e-→-jouer-e2→e4-→-IA-répond-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-renfor-ff983-e-passe-faible-1-char-→-400-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-renfor-2b331--passe-faible-5-chars-→-400-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-renfor-33c4e-se-faible-8-chars-→-accepte-api-sanity-retry1/trace.zip
  63) [api-sanity] › tests/api-sanity.spec.ts:392:3 › Sécurité — Change password autre user → 403 (fix C1) › User change pwd autre user → 403 (integration) 
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Chan-1362b-tre-user-→-403-integration--api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Uplo-4b668--refuse-→-400-second-block--api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Uplo-09a50--Upload-fichier-texte-→-200-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Uplo-7b520-pload-→-Download-end-to-end-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Uplo-fcd77-n-Download-inexistant-→-404-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Mess-d0854-ation-Envoyer-message-→-200-api-sanity-retry1/trace.zip
        npx playwright show-trace test-results/api-sanity-Sécurité-—-Mess-f0449-rsation-→-200-second-block--api-sanity-retry1/trace.zip
  69 failed
    [api-sanity] › tests/api-sanity.spec.ts:392:3 › Sécurité — Change password autre user → 403 (fix C1) › User change pwd autre user → 403 (integration) 
  5 passed (2.6m)
```

---

## Erreurs détectées

```
Error: apiRequestContext.get: connect ECONNREFUSED ::1:6300
    Call log:
      - → GET http://localhost:6300/api/health
        - user-agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/151.0.7922.34 Safari/537.36
        - accept: */*
        - accept-encoding: gzip,deflate,br
--
    Error: apiRequestContext.get: connect ECONNREFUSED ::1:6300
    Call log:
      - → GET http://localhost:6300/api/health
        - user-agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/151.0.7922.34 Safari/537.36
        - accept: */*
        - accept-encoding: gzip,deflate,br
--
    Error: apiRequestContext.get: connect ECONNREFUSED ::1:6300
    Call log:
      - → GET http://localhost:6300/api/push/vapid-public-key
        - user-agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/151.0.7922.34 Safari/537.36
        - accept: */*
        - accept-encoding: gzip,deflate,br
--
    Error: apiRequestContext.get: connect ECONNREFUSED ::1:6300
    Call log:
      - → GET http://localhost:6300/api/push/vapid-public-key
        - user-agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/151.0.7922.34 Safari/537.36
        - accept: */*
        - accept-encoding: gzip,deflate,br
--
    Error: apiRequestContext.get: connect ECONNREFUSED ::1:6300
    Call log:
      - → GET http://localhost:6300/api/auth/me
        - user-agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/151.0.7922.34 Safari/537.36
        - accept: */*
        - accept-encoding: gzip,deflate,br
--
    Error: apiRequestContext.get: connect ECONNREFUSED ::1:6300
    Call log:
      - → GET http://localhost:6300/api/auth/me
        - user-agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/151.0.7922.34 Safari/537.36
        - accept: */*
        - accept-encoding: gzip,deflate,br
--
    Error: apiRequestContext.post: connect ECONNREFUSED ::1:6300
    Call log:
      - → POST http://localhost:6300/api/auth/logout
        - user-agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/151.0.7922.34 Safari/537.36
        - accept: */*
        - accept-encoding: gzip,deflate,br
--
    Error: apiRequestContext.post: connect ECONNREFUSED ::1:6300
    Call log:
      - → POST http://localhost:6300/api/auth/logout
        - user-agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/151.0.7922.34 Safari/537.36
        - accept: */*
        - accept-encoding: gzip,deflate,br
--
    Error: apiRequestContext.post: connect ECONNREFUSED ::1:6300
    Call log:
      - → POST http://localhost:6300/api/auth/change-password
        - user-agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/151.0.7922.34 Safari/537.36
```

---

## Output brut (200 dernières lignes)

```
test-results/api-sanity-Sécurité-—-Uplo-7b520-pload-→-Download-end-to-end-api-sanity-retry1/trace.zip
    Usage:

        npx playwright show-trace test-results/api-sanity-Sécurité-—-Uplo-7b520-pload-→-Download-end-to-end-api-sanity-retry1/trace.zip

    ────────────────────────────────────────────────────────────────────────────────────────────────

  67) [api-sanity] › tests/api-sanity.spec.ts:461:3 › Sécurité — Upload validation › Download inexistant → 404 

    Error: apiRequestContext.post: connect ECONNREFUSED ::1:6300
    Call log:
      - → POST http://localhost:6300/api/auth/login
        - user-agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/151.0.7922.34 Safari/537.36
        - accept: */*
        - accept-encoding: gzip,deflate,br
        - content-type: application/json
        - content-length: 46

        at apiRequestContext.post: connect ECONNREFUSED ::1:6300
        at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:462:33

    Error Context: test-results/api-sanity-Sécurité-—-Uplo-fcd77-n-Download-inexistant-→-404-api-sanity/error-context.md

    Retry #1 ───────────────────────────────────────────────────────────────────────────────────────

    Error: apiRequestContext.post: connect ECONNREFUSED ::1:6300
    Call log:
      - → POST http://localhost:6300/api/auth/login
        - user-agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/151.0.7922.34 Safari/537.36
        - accept: */*
        - accept-encoding: gzip,deflate,br
        - content-type: application/json
        - content-length: 46

        at apiRequestContext.post: connect ECONNREFUSED ::1:6300
        at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:462:33

    Error Context: test-results/api-sanity-Sécurité-—-Uplo-fcd77-n-Download-inexistant-→-404-api-sanity-retry1/error-context.md

    attachment #2: trace (application/zip) ─────────────────────────────────────────────────────────
    test-results/api-sanity-Sécurité-—-Uplo-fcd77-n-Download-inexistant-→-404-api-sanity-retry1/trace.zip
    Usage:

        npx playwright show-trace test-results/api-sanity-Sécurité-—-Uplo-fcd77-n-Download-inexistant-→-404-api-sanity-retry1/trace.zip

    ────────────────────────────────────────────────────────────────────────────────────────────────

  68) [api-sanity] › tests/api-sanity.spec.ts:475:3 › Sécurité — Message CRUD conversation › Envoyer message → 200 

    Error: apiRequestContext.post: connect ECONNREFUSED ::1:6300
    Call log:
      - → POST http://localhost:6300/api/auth/login
        - user-agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/151.0.7922.34 Safari/537.36
        - accept: */*
        - accept-encoding: gzip,deflate,br
        - content-type: application/json
        - content-length: 46

        at apiRequestContext.post: connect ECONNREFUSED ::1:6300
        at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:476:33

    Error Context: test-results/api-sanity-Sécurité-—-Mess-d0854-ation-Envoyer-message-→-200-api-sanity/error-context.md

    Retry #1 ───────────────────────────────────────────────────────────────────────────────────────

    Error: apiRequestContext.post: connect ECONNREFUSED ::1:6300
    Call log:
      - → POST http://localhost:6300/api/auth/login
        - user-agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/151.0.7922.34 Safari/537.36
        - accept: */*
        - accept-encoding: gzip,deflate,br
        - content-type: application/json
        - content-length: 46

        at apiRequestContext.post: connect ECONNREFUSED ::1:6300
        at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:476:33

    Error Context: test-results/api-sanity-Sécurité-—-Mess-d0854-ation-Envoyer-message-→-200-api-sanity-retry1/error-context.md

    attachment #2: trace (application/zip) ─────────────────────────────────────────────────────────
    test-results/api-sanity-Sécurité-—-Mess-d0854-ation-Envoyer-message-→-200-api-sanity-retry1/trace.zip
    Usage:

        npx playwright show-trace test-results/api-sanity-Sécurité-—-Mess-d0854-ation-Envoyer-message-→-200-api-sanity-retry1/trace.zip

    ────────────────────────────────────────────────────────────────────────────────────────────────

  69) [api-sanity] › tests/api-sanity.spec.ts:529:3 › Sécurité — Message CRUD conversation › Rename conversation → 200 (second block) 

    Error: apiRequestContext.post: connect ECONNREFUSED ::1:6300
    Call log:
      - → POST http://localhost:6300/api/auth/login
        - user-agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/151.0.7922.34 Safari/537.36
        - accept: */*
        - accept-encoding: gzip,deflate,br
        - content-type: application/json
        - content-length: 46

        at apiRequestContext.post: connect ECONNREFUSED ::1:6300
        at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:530:33

    Error Context: test-results/api-sanity-Sécurité-—-Mess-f0449-rsation-→-200-second-block--api-sanity/error-context.md

    Retry #1 ───────────────────────────────────────────────────────────────────────────────────────

    Error: apiRequestContext.post: connect ECONNREFUSED ::1:6300
    Call log:
      - → POST http://localhost:6300/api/auth/login
        - user-agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/151.0.7922.34 Safari/537.36
        - accept: */*
        - accept-encoding: gzip,deflate,br
        - content-type: application/json
        - content-length: 46

        at apiRequestContext.post: connect ECONNREFUSED ::1:6300
        at /home/runner/work/Nook/Nook/frontend/tests/api-sanity.spec.ts:530:33

    Error Context: test-results/api-sanity-Sécurité-—-Mess-f0449-rsation-→-200-second-block--api-sanity-retry1/error-context.md

    attachment #2: trace (application/zip) ─────────────────────────────────────────────────────────
    test-results/api-sanity-Sécurité-—-Mess-f0449-rsation-→-200-second-block--api-sanity-retry1/trace.zip
    Usage:

        npx playwright show-trace test-results/api-sanity-Sécurité-—-Mess-f0449-rsation-→-200-second-block--api-sanity-retry1/trace.zip

    ────────────────────────────────────────────────────────────────────────────────────────────────

  69 failed
    [api-sanity] › tests/api-sanity.spec.ts:12:3 › Sanité — Serveur › GET /api/health → "OK" ───────
    [api-sanity] › tests/api-sanity.spec.ts:18:3 › Sanité — Serveur › GET /push/vapid-public-key → 200 (route publique, pas d'auth requise) 
    [api-sanity] › tests/api-sanity.spec.ts:81:5 › Sécurité — Routes non-auth → 401 › GET /auth/me → 401 
    [api-sanity] › tests/api-sanity.spec.ts:81:5 › Sécurité — Routes non-auth → 401 › POST /auth/logout → 401 
    [api-sanity] › tests/api-sanity.spec.ts:81:5 › Sécurité — Routes non-auth → 401 › POST /auth/change-password → 401 
    [api-sanity] › tests/api-sanity.spec.ts:81:5 › Sécurité — Routes non-auth → 401 › POST /auth/public-key → 401 
    [api-sanity] › tests/api-sanity.spec.ts:81:5 › Sécurité — Routes non-auth → 401 › GET /auth/public-keys?conversation_id=default_global → 401 
    [api-sanity] › tests/api-sanity.spec.ts:81:5 › Sécurité — Routes non-auth → 401 › GET /conversations → 401 
    [api-sanity] › tests/api-sanity.spec.ts:81:5 › Sécurité — Routes non-auth → 401 › POST /conversations → 401 
    [api-sanity] › tests/api-sanity.spec.ts:81:5 › Sécurité — Routes non-auth → 401 › GET /conversations/default_global → 401 
    [api-sanity] › tests/api-sanity.spec.ts:81:5 › Sécurité — Routes non-auth → 401 › GET /conversations/default_global/messages → 401 
    [api-sanity] › tests/api-sanity.spec.ts:81:5 › Sécurité — Routes non-auth → 401 › POST /conversations/default_global/messages → 401 
    [api-sanity] › tests/api-sanity.spec.ts:81:5 › Sécurité — Routes non-auth → 401 › GET /conversations/default_global/participants → 401 
    [api-sanity] › tests/api-sanity.spec.ts:81:5 › Sécurité — Routes non-auth → 401 › POST /conversations/default_global/participants → 401 
    [api-sanity] › tests/api-sanity.spec.ts:81:5 › Sécurité — Routes non-auth → 401 › POST /conversations/default_global/leave → 401 
    [api-sanity] › tests/api-sanity.spec.ts:81:5 › Sécurité — Routes non-auth → 401 › PATCH /conversations/default_global/rename → 401 
    [api-sanity] › tests/api-sanity.spec.ts:81:5 › Sécurité — Routes non-auth → 401 › GET /download/fake-id-000 → 401 
    [api-sanity] › tests/api-sanity.spec.ts:81:5 › Sécurité — Routes non-auth → 401 › GET /events → 401 
    [api-sanity] › tests/api-sanity.spec.ts:81:5 › Sécurité — Routes non-auth → 401 › POST /events → 401 
    [api-sanity] › tests/api-sanity.spec.ts:81:5 › Sécurité — Routes non-auth → 401 › GET /polls → 401 
    [api-sanity] › tests/api-sanity.spec.ts:81:5 › Sécurité — Routes non-auth → 401 › POST /polls → 401 
    [api-sanity] › tests/api-sanity.spec.ts:81:5 › Sécurité — Routes non-auth → 401 › GET /polls/fake-id → 401 
    [api-sanity] › tests/api-sanity.spec.ts:81:5 › Sécurité — Routes non-auth → 401 › POST /polls/fake-id/vote → 401 
    [api-sanity] › tests/api-sanity.spec.ts:81:5 › Sécurité — Routes non-auth → 401 › POST /polls/fake-id/close → 401 
    [api-sanity] › tests/api-sanity.spec.ts:81:5 › Sécurité — Routes non-auth → 401 › DELETE /polls/fake-id → 401 
    [api-sanity] › tests/api-sanity.spec.ts:81:5 › Sécurité — Routes non-auth → 401 › GET /chess/list → 401 
    [api-sanity] › tests/api-sanity.spec.ts:81:5 › Sécurité — Routes non-auth → 401 › POST /chess/create → 401 
    [api-sanity] › tests/api-sanity.spec.ts:81:5 › Sécurité — Routes non-auth → 401 › GET /chess/invitations → 401 
    [api-sanity] › tests/api-sanity.spec.ts:81:5 › Sécurité — Routes non-auth → 401 › GET /chess/fake-id → 401 
    [api-sanity] › tests/api-sanity.spec.ts:81:5 › Sécurité — Routes non-auth → 401 › POST /chess/fake-id/move → 401 
    [api-sanity] › tests/api-sanity.spec.ts:81:5 › Sécurité — Routes non-auth → 401 › GET /chess/fake-id/moves?from=e2 → 401 
    [api-sanity] › tests/api-sanity.spec.ts:81:5 › Sécurité — Routes non-auth → 401 › POST /chess/fake-id/ai-move → 401 
    [api-sanity] › tests/api-sanity.spec.ts:81:5 › Sécurité — Routes non-auth → 401 › POST /chess/fake-id/resign → 401 
    [api-sanity] › tests/api-sanity.spec.ts:81:5 › Sécurité — Routes non-auth → 401 › POST /conversations/default_global/messages/x/reactions → 401 
    [api-sanity] › tests/api-sanity.spec.ts:81:5 › Sécurité — Routes non-auth → 401 › DELETE /conversations/default_global/messages/x/reactions → 401 
    [api-sanity] › tests/api-sanity.spec.ts:81:5 › Sécurité — Routes non-auth → 401 › GET /conversations/default_global/messages/x/reactions → 401 
    [api-sanity] › tests/api-sanity.spec.ts:81:5 › Sécurité — Routes non-auth → 401 › POST /user/update → 401 
    [api-sanity] › tests/api-sanity.spec.ts:81:5 › Sécurité — Routes non-auth → 401 › GET /users/available → 401 
    [api-sanity] › tests/api-sanity.spec.ts:81:5 › Sécurité — Routes non-auth → 401 › GET /push/preferences → 401 
    [api-sanity] › tests/api-sanity.spec.ts:91:3 › Sécurité — Routes non-auth → 401 › POST /api/upload/chat sans auth → 401 
    [api-sanity] › tests/api-sanity.spec.ts:116:5 › Sécurité — Routes admin → 401 sans auth › GET /users/pending → 401 
    [api-sanity] › tests/api-sanity.spec.ts:116:5 › Sécurité — Routes admin → 401 sans auth › GET /users → 401 
    [api-sanity] › tests/api-sanity.spec.ts:116:5 › Sécurité — Routes admin → 401 sans auth › POST /users/approve → 401 
    [api-sanity] › tests/api-sanity.spec.ts:116:5 › Sécurité — Routes admin → 401 sans auth › GET /invites → 401 
    [api-sanity] › tests/api-sanity.spec.ts:116:5 › Sécurité — Routes admin → 401 sans auth › POST /invites → 401 
    [api-sanity] › tests/api-sanity.spec.ts:116:5 › Sécurité — Routes admin → 401 sans auth › POST /invites/delete → 401 
    [api-sanity] › tests/api-sanity.spec.ts:116:5 › Sécurité — Routes admin → 401 sans auth › GET /analytics → 401 
    [api-sanity] › tests/api-sanity.spec.ts:130:3 › Sécurité — Mot de passe faible → rejeté › Mot de passe 1 char → 400 
    [api-sanity] › tests/api-sanity.spec.ts:137:3 › Sécurité — Mot de passe faible → rejeté › Mot de passe 5 chars → 400 
    [api-sanity] › tests/api-sanity.spec.ts:144:3 › Sécurité — Mot de passe faible → rejeté › Mot de passe 7 chars → 400 
    [api-sanity] › tests/api-sanity.spec.ts:151:3 › Sécurité — Mot de passe faible → rejeté › Mot de passe 8 chars → accepte 
    [api-sanity] › tests/api-sanity.spec.ts:161:3 › Sécurité — Change password autre user → 403 › User normal change pwd autre user → 403 
    [api-sanity] › tests/api-sanity.spec.ts:177:3 › Sécurité — Upload validation › Upload fichier vide → 400 
    [api-sanity] › tests/api-sanity.spec.ts:195:3 › Sécurité — Upload/Download end-to-end › Upload fichier texte → file_id, puis download OK 
    [api-sanity] › tests/api-sanity.spec.ts:218:3 › Sécurité — Upload/Download end-to-end › Download fichier inexistant → 404 
    [api-sanity] › tests/api-sanity.spec.ts:230:3 › Sécurité — Message conversation CRUD › Envoyer message → 200, récupérer → contient message 
    [api-sanity] › tests/api-sanity.spec.ts:264:3 › Sécurité — Message conversation CRUD › Rename conversation → 200 
    [api-sanity] › tests/api-sanity.spec.ts:279:3 › Sécurité — Call page access › /call/fake-id sans auth → redirige vers /login 
    [api-sanity] › tests/api-sanity.spec.ts:311:3 › Sécurité — Chess spécial › Créer partie → jouer e2→e4 → IA répond 
    [api-sanity] › tests/api-sanity.spec.ts:369:3 › Sécurité renforcée — Mot de passe faible › 1 char → 400 
    [api-sanity] › tests/api-sanity.spec.ts:376:3 › Sécurité renforcée — Mot de passe faible › 5 chars → 400 
    [api-sanity] › tests/api-sanity.spec.ts:383:3 › Sécurité renforcée — Mot de passe faible › 8 chars → accepte 
    [api-sanity] › tests/api-sanity.spec.ts:392:3 › Sécurité — Change password autre user → 403 (fix C1) › User change pwd autre user → 403 (integration) 
    [api-sanity] › tests/api-sanity.spec.ts:406:3 › Sécurité — Upload validation › Upload sec -- fichier vide refuse → 400 (second block) 
    [api-sanity] › tests/api-sanity.spec.ts:422:3 › Sécurité — Upload validation › Upload fichier texte → 200 
    [api-sanity] › tests/api-sanity.spec.ts:440:3 › Sécurité — Upload validation › Upload → Download end-to-end 
    [api-sanity] › tests/api-sanity.spec.ts:461:3 › Sécurité — Upload validation › Download inexistant → 404 
    [api-sanity] › tests/api-sanity.spec.ts:475:3 › Sécurité — Message CRUD conversation › Envoyer message → 200 
    [api-sanity] › tests/api-sanity.spec.ts:529:3 › Sécurité — Message CRUD conversation › Rename conversation → 200 (second block) 
  1 skipped
  6 did not run
  5 passed (2.6m)
```

---

*Rapport généré par `.github/workflows/e2e-targeted.yml`*
