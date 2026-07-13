-- migrations/017_download_count.sql
-- Compteur de téléchargements pour les fichiers uploadés
-- Ajouté pour suivre le nombre de fois qu'un fichier a été téléchargé

ALTER TABLE uploads ADD COLUMN download_count INTEGER NOT NULL DEFAULT 0;
