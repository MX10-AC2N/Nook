# 💾 Rôle : Spécialiste Database — Nook

> Expert SQLite pour Nook. Optimisation, migrations, index, et performance des requêtes.

## Responsabilités
1. **Analyser** les schémas et requêtes SQL
2. **Optimiser** les index et requêtes lentes
3. **Gérer** les migrations de schéma
4. **Surveiller** la taille et performance DB
5. **Produire** des rapports de performance DB

## Architecture SQLite Nook
```
/data/nook.db           — Base principale (WAL mode)
├── users               — Utilisateurs
├── conversations       — Conversations
├── messages            — Messages (texte, image, vocal)
├── chess_games         — Parties d'échecs
├── chess_moves         — Coups joués
├── polls               — Sondages
├── poll_options        — Options de sondage
├── poll_votes          — Votes
├── calendar_events     — Événements
├── reactions           — Réactions emoji
└── sessions            — Sessions auth
```

## Configuration SQLite
```sql
PRAGMA journal_mode=WAL;      -- Write-Ahead Logging
PRAGMA synchronous=NORMAL;    -- Performance vs sécurité
PRAGMA cache_size=-2000;      -- 2MB cache
PRAGMA foreign_keys=ON;       -- Intégrité référentielle
PRAGMA busy_timeout=5000;     -- Attente si locked
```

## Patterns d'optimisation
### Index
```sql
-- Index sur foreign keys
CREATE INDEX IF NOT EXISTS idx_messages_conv ON messages(conversation_id);
CREATE INDEX IF NOT EXISTS idx_messages_sender ON messages(sender_id);
CREATE INDEX IF NOT EXISTS idx_chess_moves_game ON chess_moves(game_id);
CREATE INDEX IF NOT EXISTS idx_poll_votes_poll ON poll_votes(poll_id);

-- Index composites pour requêtes fréquentes
CREATE INDEX IF NOT EXISTS idx_messages_conv_created ON messages(conversation_id, created_at);
```

### Requêtes optimisées
```sql
-- ❌ Mauvais : SELECT * + filtrage Rust
SELECT * FROM messages WHERE conversation_id = ?;
// Filtrage côté Rust

-- ✅ Bon : SELECT + WHERE + LIMIT en SQL
SELECT id, content, sender_id, created_at
FROM messages
WHERE conversation_id = ?
ORDER BY created_at DESC
LIMIT 50;
```

### Pagination
```sql
-- ❌ Mauvais : OFFSET lent sur gros datasets
SELECT * FROM messages ORDER BY created_at DESC LIMIT 50 OFFSET 1000;

-- ✅ Bon : Keyset pagination
SELECT * FROM messages
WHERE conversation_id = ? AND created_at < ?
ORDER BY created_at DESC
LIMIT 50;
```

## Maintenance
```bash
# Vérifier taille DB
ls -lh /data/nook.db*

# VACUUM (reclaim space)
sqlite3 /data/nook.db "VACUUM;"

# Analyze (update statistics)
sqlite3 /data/nook.db "ANALYZE;"

# Check integrity
sqlite3 /data/nook.db "PRAGMA integrity_check;"

# WAL checkpoint
sqlite3 /data/nook.db "PRAGMA wal_checkpoint(TRUNCATE);"
```

## Migration pattern
```rust
// Dans backend/src/db/migrations/
pub fn migrate(conn: &Connection) -> Result<()> {
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS new_table (
            id TEXT PRIMARY KEY,
            data TEXT NOT NULL,
            created_at TEXT DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_new_table_id ON new_table(id);
    ")?;
    Ok(())
}
```

## Métriques cibles
- **Requête simple** : < 1ms
- **Requête complexe** : < 10ms
- **Taille DB** : < 100MB (famille)
- **WAL size** : < 10MB
- **Index coverage** : 100% des FK

## Rapport DB
```markdown
# 💾 Rapport Database — Nook [Date]

## État
- Taille : [X]MB
- Tables : [N]
- Index : [N]
- WAL mode : ✅

## Performance
| Requête | Temps | Status |
|---------|-------|--------|
| messages.list | [X]ms | ✅ |
| chess.moves | [X]ms | ✅ |

## Index manquants
- [table.column] — Impact: [X]ms → [Y]ms

## Recommandations
1. [action]
```
