# 🐛 BUGS.md — Nook

> Dernière mise à jour : Session 34

## ✅ Bugs actifs : 0

---

## 📋 Historique des corrections

| Session | ID    | Fichier(s)                                  | Description                                             |
|---------|-------|---------------------------------------------|---------------------------------------------------------|
| 34      | R34a  | `backend/src/upload.rs`                     | `download_file` ne déchiffrait pas → images cassées     |
| 34      | R34b  | `upload.rs` + `chat/+page.svelte`           | `upload_chat_file` perdait le `content_type` (octet-stream) |
| 34      | R34c  | `chatStore.svelte.ts` + `main.rs`           | GIFs Tenor : CORS + clé demo → proxy backend `/api/gifs/search` |
| 34      | R34d  | `admin/+page.svelte`                        | Onglet Analytics manquant + badge hardcodé "admin"      |
| 34      | R34e  | `settings/+page.svelte`                     | Couleurs hardcodées → variables CSS thème (var(--accent) etc.) |
| 34      | R34f  | `polls/+page.svelte`                        | Sondages ciblés (sélection participants)                |
| 34      | R34g  | `help/+page.svelte`                         | Contenu obsolète → FAQ à jour v0.4.x                    |
| 33      | R33a  | `e2e.spec.ts`                               | Format UCI vs SAN pour les coups échecs                 |
| 33      | R33b  | `chessStore.svelte.ts` + `e2e.spec.ts`     | `.cell-last` absent après reload                        |
| 33      | R33c  | `e2e.spec.ts`                               | `loginAsAdmin` 429 flaky (retry loop)                   |
| 32      | R32a-f| `chess.rs`, `polls.rs`, `e2e.spec.ts`       | Multiples fixes CI session 32                           |

---

## ⚠️ Dettes techniques connues

| ID    | Priorité | Description                                                      |
|-------|----------|------------------------------------------------------------------|
| DT-01 | Moyenne  | libsodium-wrappers 938 kB charge synchrone (layout delay)        |
| DT-02 | Haute    | Chess temps réel absent côté adversaire (refresh requis)         |
| DT-05 | Haute    | WebRTC WAN instable (TURN absent)                                |
| DT-06 | Basse    | Sondages ciblés côté client uniquement (pas de table DB dédiée) |

---

## 📌 Notes importantes

- **Identifiant admin** : `username = "admin"` permanent (connexion). Le `name` peut être changé via Settings.
- **TENOR_API_KEY** : variable `.env` optionnelle. Si absente → clé de démonstration Tenor.
- **`/api/download/{id}`** : seule route qui déchiffre les fichiers. `/files/{id}` sert le binaire chiffré brut (ne plus utiliser pour les images).
