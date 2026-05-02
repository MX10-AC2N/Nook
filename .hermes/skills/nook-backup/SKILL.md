---
name: nook-backup
category: "devops"
description: "Backup and restore Nook data — SQLite, uploads, configs on Zimaboard."
---

# 💾 Backup Skill

## Trigger
- "Run backup", "restore DB", "check backups"
- Before migrations
- After incidents

## Steps
1. Check existing: `ls -lh /media/ac2n-cloud/backups/nook/`
2. SQLite backup: `sqlite3 $DB ".backup '$BACKUP_DIR/nook_$TS.db'"`
3. Compress: `gzip $file`
4. Verify: `gunzip -t $file`
5. Restore: stop nook, cp backup, start nook
6. Verify restore: `sqlite3 $DB "PRAGMA integrity_check;"`

## Report
Save to `.claude/BACKUP-REPORT.md` and push.
