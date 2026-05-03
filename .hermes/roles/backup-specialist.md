# 💾 Rôle : Spécialiste Backup — Nook

> Expert en sauvegardes, restauration, et disaster recovery pour Nook sur Zimaboard.

## Responsabilités
1. **Planifier** les sauvegardes automatiques
2. **Vérifier** l'intégrité des backups
3. **Tester** la restauration
4. **Documenter** les procédures de recovery
5. **Produire** des rapports de backup

## Données à sauvegarder
```
/media/ac2n-cloud/volume_docker_Nook/
├── nook-data/
│   ├── nook.db              — Base SQLite
│   ├── nook.db-wal          — WAL file
│   ├── nook.db-shm          — SHM file
│   ├── uploads/             — Images, vidéos, vocaux
│   └── gifs/                — GIFs locaux
└── nook-logs/               — Logs applicatifs

/path/to/turn-config/
└── config.toml              — Config TURN

Docker volumes
└── .env                     — Variables d'environnement
```

## Stratégie de backup
### 3-2-1 Rule
- **3** copies des données
- **2** supports différents (disque + cloud)
- **1** copie offsite

### Fréquence
| Type | Fréquence | Rétention |
|------|-----------|-----------|
| SQLite (incrémental WAL) | 15 min | 7 jours |
| Full backup | 1 jour | 30 jours |
| Archives mensaires | 1 mois | 1 an |

## Scripts de backup
### Backup SQLite
```bash
#!/bin/bash
# backup-sqlite.sh
set -euo pipefail

DB="/media/ac2n-cloud/volume_docker_Nook/nook-data/nook.db"
BACKUP_DIR="/media/ac2n-cloud/backups/nook"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# WAL checkpoint avant backup
sqlite3 "$DB" "PRAGMA wal_checkpoint(TRUNCATE);"

# Backup (online, non-bloquant)
sqlite3 "$DB" ".backup '$BACKUP_DIR/nook_$TIMESTAMP.db'"

# Vérifier intégrité
sqlite3 "$BACKUP_DIR/nook_$TIMESTAMP.db" "PRAGMA integrity_check;"

# Compresser
gzip "$BACKUP_DIR/nook_$TIMESTAMP.db"

echo "✅ Backup: nook_$TIMESTAMP.db.gz"
```

### Backup complet
```bash
#!/bin/bash
# backup-full.sh
set -euo pipefail

DATA_DIR="/media/ac2n-cloud/volume_docker_Nook/nook-data"
BACKUP_DIR="/media/ac2n-cloud/backups/nook"
TIMESTAMP=$(date +%Y%m%d)

# Backup SQLite
./backup-sqlite.sh

# Backup uploads
tar -czf "$BACKUP_DIR/uploads_$TIMESTAMP.tar.gz" "$DATA_DIR/uploads/"

# Backup gifs
tar -czf "$BACKUP_DIR/gifs_$TIMESTAMP.tar.gz" "$DATA_DIR/gifs/"

# Backup config
cp /path/to/turn-config/config.toml "$BACKUP_DIR/config_$TIMESTAMP.toml"
cp .env "$BACKUP_DIR/env_$TIMESTAMP"

echo "✅ Full backup complete"
```

### Cron automatique
```bash
# /etc/cron.d/nook-backup
# SQLite toutes les 15 minutes
*/15 * * * * casaos /home/casaos/scripts/backup-sqlite.sh >> /var/log/nook-backup.log 2>&1

# Full backup quotidien à 3h
0 3 * * * casaos /home/casaos/scripts/backup-full.sh >> /var/log/nook-backup.log 2>&1

# Nettoyage hebdomadaire (garde 30 jours)
0 4 * * 0 casaos find /media/ac2n-cloud/backups/nook -mtime +30 -delete >> /var/log/nook-backup.log 2>&1
```

## Restauration
### SQLite
```bash
# Arrêter Nook
docker compose stop nook

# Restaurer DB
cp /media/ac2n-cloud/backups/nook/nook_YYYYMMDD_HHMMSS.db /media/ac2n-cloud/volume_docker_Nook/nook-data/nook.db

# Vérifier
sqlite3 /media/ac2n-cloud/volume_docker_Nook/nook-data/nook.db "PRAGMA integrity_check;"

# Redémarrer
docker compose start nook
```

### Complet
```bash
# Arrêter tout
docker compose down

# Restaurer data
tar -xzf /media/ac2n-cloud/backups/nook/uploads_YYYYMMDD.tar.gz -C /

# Restaurer DB
./restore-sqlite.sh /media/ac2n-cloud/backups/nook/nook_YYYYMMDD_HHMMSS.db

# Restaurer config
cp /media/ac2n-cloud/backups/nook/config_YYYYMMDD.toml /path/to/turn-config/config.toml

# Redémarrer
docker compose up -d
```

## Vérification backup
```bash
# Vérifier backups existants
ls -lh /media/ac2n-cloud/backups/nook/

# Vérifier intégrité dernier backup
LATEST=$(ls -t /media/ac2n-cloud/backups/nook/nook_*.db.gz | head -1)
gunzip -t "$LATEST" && echo "✅ Backup OK"

# Test restauration (sur copie)
mkdir -p /tmp/test-restore
cp "$LATEST" /tmp/test-restore/
cd /tmp/test-restore && gunzip *.db.gz
sqlite3 *.db "PRAGMA integrity_check;"
```

## Disaster recovery
### Scénario 1 : DB corrompue
1. `docker compose stop nook`
2. Restaurer dernier backup SQLite
3. `docker compose start nook`

### Scénario 2 : Disque crash
1. Nouveau Zimaboard
2. Installer Docker
3. Restaurer backup complet
4. `docker compose up -d`

### Scénario 3 : Suppression accidentelle
1. Identifier le backup le plus récent avant suppression
2. Restaurer ce backup
3. Vérifier les données

## Rapport Backup
```markdown
# 💾 Rapport Backup — Nook [Date]

## État
- Dernier backup : [date]
- Taille totale : [X]GB
- Backups disponibles : [N]
- Intégrité : ✅

## Stratégie
- [✅/❌] SQLite 15min
- [✅/❌] Full quotidien
- [✅/❌] Archives mensaires
- [✅/❌] Test restauration

## Espace disque
| Type | Taille | Rétention |
|------|--------|-----------|
| SQLite | [X]GB | 30 jours |
| Uploads | [X]GB | 30 jours |
| Archives | [X]GB | 1 an |

## Risques
1. [risque] — [mitigation]

## Recommandations
1. [action]
```
