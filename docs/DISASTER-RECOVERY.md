# Nook — Disaster Recovery Plan (DRP)

> Version : 1.0 | Dernière mise à jour : 2026-05-04
> Référence P0 : No disaster recovery docs

## 1. Objectif
Ce document décrit les procédures de reprise après sinistre pour Nook (hébergé sur Zimaboard via Docker).

## 2. Sauvegardes (Backups)
### 2.1 Script de sauvegarde
Un script automatisé `scripts/backup-nook.sh` est disponible. Il sauvegarde :
- Base SQLite (`backend/nook.db`)
- Fichiers de config (`turn-config/`, `.env.example`, `docker-compose.yml`)
- Uploads utilisateurs (`backend/uploads/`)

### 2.2 Planification
Ajouter une tâche cron pour exécuter le script quotidiennement :
```bash
0 2 * * * /opt/data/home/.hermes/Nook/scripts/backup-nook.sh >> /var/log/nook-backup.log 2>&1
```

### 2.3 Stockage
- Sauvegardes locales : `/opt/data/backups/nook/`
- Sauvegardes externes : Copier les fichiers `.tar.gz` et `.db` vers un cloud (S3, Backblaze B2) ou un disque externe.

## 3. Procédure de restauration complète
### 3.1 Pré-requis
- Docker et docker-compose installés
- Accès au repo Nook (`git clone https://github.com/MX10-AC2N/Nook -b develop`)
- Fichier `.env` valide (avec `TURN_SECRET`, `DATABASE_URL`, etc.)

### 3.2 Restauration de la base de données
1. Copier le fichier de sauvegarde SQLite :
   ```bash
   cp /opt/data/backups/nook/nook-20260504-120000.db /opt/data/home/.hermes/Nook/backend/nook.db
   ```
2. Vérifier l'intégrité :
   ```bash
   sqlite3 /opt/data/home/.hermes/Nook/backend/nook.db "PRAGMA integrity_check;"
   ```

### 3.3 Restauration des fichiers de config
1. Extraire l'archive de config :
   ```bash
   tar -xzf /opt/data/backups/nook/nook-backup-20260504-120000.tar.gz -C /opt/data/home/.hermes/Nook/
   ```

### 3.4 Redémarrage des services
```bash
cd /opt/data/home/.hermes/Nook
docker-compose down
docker-compose up -d --build
```

## 4. Procédures d'urgence
### 4.1 Panne matérielle (Zimaboard)
1. Récupérer les sauvegardes sur un support externe
2. Installer Nook sur un nouveau serveur (suivre `README.md`)
3. Restaurer les sauvegardes (§3.2 et §3.3)
4. Mettre à jour le DNS (si nom de domaine utilisé)

### 4.2 Corruption de la base de données
1. Restaurer la dernière sauvegarde valide (§3.2)
2. Si aucune sauvegarde valide : recréer la base avec les migrations :
   ```bash
   cd /opt/data/home/.hermes/Nook/backend
   sqlx migrate run
   ```

## 5. Contacts
- Admin : [Ton nom/email]
- Repo : https://github.com/MX10-AC2N/Nook
- Documentation : `/docs/` dans le repo
