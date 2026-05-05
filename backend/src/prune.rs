// backend/src/prune.rs
// Nettoyage périodique des données anciennes (> 7 jours)
// Session 9 — correction conversation_members → conversation_participants
// Session 10 — FIX: ne pas supprimer les conversations de groupe (is_group = 1)
//               Cause : default_global (groupe, 0 messages au boot) était supprimée
//               par le prune 10s après sa création → POST /messages retournait 404

use crate::db::Upload;
use sqlx::{Error, SqlitePool};
use std::path::Path;
use tokio::fs;

pub async fn prune_old_data(pool: &SqlitePool) -> Result<(), Error> {
    let seven_days_ago = chrono::Utc::now().timestamp() - (7 * 24 * 3600);

    // ─── 1. Nettoyage des dépendances avant suppression messages ──────────
    // Note : message_reactions et message_keys ont ON DELETE CASCADE, mais on supprime
    // explicitement pour éviter les erreurs si les contraintes ne sont pas actives.
    // ─── 1. Nettoyage des dépendances avant suppression messages ──────────
    // Clés E2EE liées aux messages anciens (message_keys a FOREIGN KEY vers messages)
    let deleted_keys = sqlx::query(
        "DELETE FROM message_keys WHERE message_id IN (SELECT id FROM messages WHERE created_at < ?)"
    )
        .bind(seven_days_ago)
        .execute(pool)
        .await?
        .rows_affected();
    tracing::info!(count = deleted_keys, "Prune : clés E2EE orphelines supprimées");

    // Réactions liées aux messages anciens (table message_reactions)
    let deleted_reactions = sqlx::query(
        "DELETE FROM message_reactions WHERE message_id IN (SELECT id FROM messages WHERE created_at < ?)"
    )
        .bind(seven_days_ago)
        .execute(pool)
        .await?
        .rows_affected();
    tracing::info!(count = deleted_reactions, "Prune : réactions orphelines supprimées");

    // Note : poll_votes n'a pas de colonne message_id dans le schéma actuel.
    // Les sondages sont liés aux messages par message_type = 'poll' mais
    // poll_votes référence poll_id, pas message_id.

    // ─── 2. Messages anciens (hard delete) ────────────────────────────────
    let deleted_messages = sqlx::query("DELETE FROM messages WHERE created_at < ?")
        .bind(seven_days_ago)
        .execute(pool)
        .await?
        .rows_affected();

    tracing::info!(
        count = deleted_messages,
        "Prune : messages anciens supprimés"
    );

    // ─── 2. Uploads orphelins (fichier physique + DB) ──────────────────────
    let orphaned_uploads: Vec<Upload> = sqlx::query_as(
        r#"
        SELECT u.* FROM uploads u
        LEFT JOIN messages m ON m.file_id = u.id
        WHERE m.id IS NULL
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut deleted_files: usize = 0;
    for upload in &orphaned_uploads {
        let file_path = Path::new(&upload.file_path);
        if file_path.exists() {
            match fs::remove_file(file_path).await {
                Ok(_) => deleted_files += 1,
                Err(e) => tracing::warn!(
                    upload_id = %upload.id,
                    error = %e,
                    "Prune : échec suppression fichier physique"
                ),
            }
        }
    }

    let deleted_uploads = sqlx::query(
        r#"
        DELETE FROM uploads
        WHERE id IN (
            SELECT u.id FROM uploads u
            LEFT JOIN messages m ON m.file_id = u.id
            WHERE m.id IS NULL
        )
        "#,
    )
    .execute(pool)
    .await?
    .rows_affected();

    tracing::info!(
        db_rows = deleted_uploads,
        files = deleted_files,
        "Prune : uploads orphelins supprimés"
    );

    // ─── 3. Conversations vides ──────────────────────────────────────
    // ⚠️  IMPORTANT : on ne supprime QUE les conversations directes (is_group = 0).
    // Les groupes (is_group = 1, ex: default_global) sont créés intentionnellement
    // par un admin et peuvent légitimement être vides au démarrage ou entre deux
    // messages. Les supprimer causait un 404 sur POST /messages en CI (bug session 10).

    // 3a. D'abord supprimer les participants des conversations vides
    let deleted_parts = sqlx::query(
        r#"
        DELETE FROM conversation_participants
        WHERE conversation_id IN (
            SELECT id FROM conversations
            WHERE is_group = 0
              AND NOT EXISTS (
                SELECT 1 FROM messages WHERE messages.conversation_id = conversations.id
              )
        )
        "#,
    )
    .execute(pool)
    .await?
    .rows_affected();

    tracing::info!(
        count = deleted_parts,
        "Prune : participants de conversations vides supprimés"
    );

    // 3b. Ensuite supprimer les conversations vides (maintenant que les participants sont partis)
    let deleted_convos = sqlx::query(
        r#"
        DELETE FROM conversations
        WHERE is_group = 0
          AND NOT EXISTS (
            SELECT 1 FROM messages WHERE messages.conversation_id = conversations.id
          )
        "#,
    )
    .execute(pool)
    .await?
    .rows_affected();

    tracing::info!(
        count = deleted_convos,
        "Prune : conversations directes vides supprimées"
    );

    // ─── 4. Invitations expirées (nettoyage bonus) ────────────────────────
    let expired_invites = sqlx::query("DELETE FROM invites WHERE expires_at < ? AND used = 0")
        .bind(chrono::Utc::now().timestamp())
        .execute(pool)
        .await?
        .rows_affected();

    if expired_invites > 0 {
        tracing::info!(
            count = expired_invites,
            "Prune : invitations expirées supprimées"
        );
    }

    Ok(())
}
