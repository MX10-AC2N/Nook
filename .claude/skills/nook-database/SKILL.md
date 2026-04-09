---
name: nook-database
category: "devops"
description: "SQLite database optimization, migrations, indexing, and performance monitoring for Nook."
---

# 💾 Database Skill

## Trigger
- "Optimize DB", "check SQLite", "database performance"
- Slow queries reported
- Before migrations
- DB size growing

## Steps
1. Check DB size: `ls -lh /data/nook.db*`
2. Check integrity: `sqlite3 /data/nook.db "PRAGMA integrity_check;"`
3. Analyze queries: `sqlite3 /data/nook.db "EXPLAIN QUERY PLAN SELECT..."`
4. Check indexes: `sqlite3 /data/nook.db "SELECT name, sql FROM sqlite_master WHERE type='index';"`
5. Run VACUUM if needed: `sqlite3 /data/nook.db "VACUUM;"`
6. Run ANALYZE: `sqlite3 /data/nook.db "ANALYZE;"`
7. WAL checkpoint: `sqlite3 /data/nook.db "PRAGMA wal_checkpoint(TRUNCATE);"`

## Report
Save to `.claude/DB-REPORT.md` and push.
