# 📊 Agent DATA — Nook

> Spécialiste données, sondages, analytics, calendrier, événements.
> Activer pour : polls, analytics admin, calendar, events, migrations data, agrégations SQL.

---

## 🎯 Périmètre

```
Backend :
├── backend/src/polls.rs               → CRUD sondages + vote + fermeture
├── backend/migrations/004_polls.sql   → schéma polls
│   (analytics = /api/analytics — endpoint à créer ou compléter)
│   (calendar/events = à confirmer selon état du backend)

Frontend :
├── frontend/src/routes/polls/+page.svelte
├── frontend/src/routes/calendar/+page.svelte
├── frontend/src/routes/events/+page.svelte
├── frontend/src/routes/admin/analytics/+page.svelte  → Chart.js doughnut
```

---

## 🗄️ Schéma DB — Polls (migration 004)

```sql
polls(
  id TEXT PRIMARY KEY,
  question TEXT NOT NULL,
  created_by TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  closed_at INTEGER,          -- NULL = ouvert
  is_closed INTEGER DEFAULT 0
)

poll_options(
  id TEXT PRIMARY KEY,
  poll_id TEXT NOT NULL REFERENCES polls(id),
  text TEXT NOT NULL,
  position INTEGER NOT NULL   -- ordre d'affichage
)

poll_votes(
  id TEXT PRIMARY KEY,
  poll_id TEXT NOT NULL,
  option_id TEXT NOT NULL,
  user_id TEXT NOT NULL,
  voted_at INTEGER NOT NULL,
  UNIQUE(poll_id, user_id)    -- un seul vote par user par sondage
)
```

---

## 🌐 API Polls — Endpoints existants

```
GET  /api/polls              → liste les sondages (ouverts + fermés)
POST /api/polls              → créer { question, options: string[] }
GET  /api/polls/{id}         → détail + résultats (comptage votes par option)
POST /api/polls/{id}/vote    → voter { option_id }
POST /api/polls/{id}/close   → fermer (créateur ou admin seulement)
DELETE /api/polls/{id}       → supprimer (admin seulement)
```

---

## 📈 Analytics Admin

```
Endpoint existant : GET /api/analytics
Réponse actuelle  : { user_count, message_count, active_sessions }

Frontend : /admin/analytics → Chart.js doughnut
```

### Extensions analytics à implémenter

```sql
-- Requêtes SQL utiles pour analytics enrichis
SELECT COUNT(*) as user_count FROM users WHERE approved = 1;
SELECT COUNT(*) as message_count FROM messages;
SELECT COUNT(*) as poll_count FROM polls;
SELECT COUNT(*) as conversation_count FROM conversations;

-- Messages par jour (7 derniers jours)
SELECT date(created_at/1000, 'unixepoch') as day, COUNT(*) as count
FROM messages
WHERE created_at > (strftime('%s', 'now') - 7*86400) * 1000
GROUP BY day ORDER BY day;

-- Users actifs (ont envoyé un message dans les 7 derniers jours)
SELECT COUNT(DISTINCT sender_id) as active_users
FROM messages
WHERE created_at > (strftime('%s', 'now') - 7*86400) * 1000;
```

---

## 📅 Calendrier & Événements

```
État actuel : frontend existant, backend à confirmer

Types frontend (types.ts) :
interface CalendarEvent {
  id: string;
  title: string;
  date: string;        // ISO 8601
  description?: string;
  created_by: string;
}
```

---

## ⚠️ Points critiques polls.rs

```rust
// polls.rs — corrections session 27 (à ne pas réintroduire)

// ❌ Tuples anonymes complexes sans turbofish → erreur type inférence
let row = sqlx::query_as("SELECT id, question FROM polls WHERE id = ?")
    .bind(id)
    .fetch_one(&pool).await?;
// → "cannot infer type for type parameter `O`"

// ✅ Struct avec #[derive(sqlx::FromRow)] + turbofish explicite
#[derive(sqlx::FromRow)]
struct PollRow { id: String, question: String }

let row = sqlx::query_as::<_, PollRow>("SELECT id, question FROM polls WHERE id = ?")
    .bind(id)
    .fetch_one(&pool).await?;

// ❌ Macros sqlx! avec SQLX_OFFLINE=true et queries.json vide
sqlx::query!("SELECT * FROM polls WHERE id = ?", id)
// → "error: failed to find data for query"

// ✅ Sans macros (query_as::<_, T> pur)
sqlx::query_as::<_, PollRow>("SELECT * FROM polls WHERE id = ?")
    .bind(id)
```

---

## 🤝 Flux inter-agents

```
← 🔐 CRYPTO / 📐 ARCHITECT : validation modèle si données sensibles
→ 🦀 RUST                  : schéma DB, endpoints, règles métier
→ 🎨 SVELTE                : types TS (Poll, CalendarEvent), contraintes UI
→ 🧪 E2E                   : scénarios data, états limites (double vote, sondage fermé)
```

---

## 📚 Apprentissages

> *Cette section est mise à jour automatiquement à chaque session.*

### [APP-DATA-01] sqlx sans macros obligatoire quand queries.json vide — Session 27

Quand `SQLX_OFFLINE=true` et `.sqlx/queries.json` ne contient pas la requête,
les macros `sqlx::query!` et `sqlx::query_as!` échouent à la compilation.
→ Toujours utiliser `sqlx::query_as::<_, MonStruct>(sql_string)` dans polls.rs.
→ Régénérer `queries.json` via `cargo sqlx prepare` si on veut les macros.

### [APP-DATA-02] UNIQUE constraint poll_votes — Design intentionnel

`UNIQUE(poll_id, user_id)` dans `poll_votes` : le backend retourne 409 si l'user
a déjà voté. Le frontend doit gérer ce code HTTP et désactiver le bouton vote.
