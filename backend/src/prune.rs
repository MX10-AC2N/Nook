use crate::db::Upload;
use sqlx::{Error, SqlitePool};
use std::path::Path;
use tokio::fs;

/// Nettoyage périodique des données anciennes (> 7 jours)
/// - Messages anciens : hard delete
/// - Uploads orphelins : suppression physique + DB
/// - Conversations vides : suppression
/// - Membres orphelins : nettoyage
pub async fn prune_old_data(pool: &SqlitePool) -> Result<(), Error> {
    let seven_days_ago = chrono::Utc::now().timestamp() - (7 * 24 * 3600);

    // 1. Supprimer les messages > 7 jours (hard delete direct)
    // → Colonne corrigée : created_at au lieu de l'ancien timestamp
    let deleted_messages = sqlx::query("DELETE FROM messages WHERE created_at < ?")
        .bind(seven_days_ago)
        .execute(pool)
        .await?
        .rows_affected();

    println!(
        "[Prune] {} messages anciens supprimés (hard delete)",
        deleted_messages
    );

    // 2. Récupérer tous les uploads qui deviennent orphelins
    // (pas référencés par un message restant)
    // ATTENTION : cette partie suppose l'existence de la table `uploads`
    // et d'une colonne `file_id` dans `messages` (probablement NULLable).
    // Si ces éléments n'existent pas encore dans votre schéma actuel,
    // commentez temporairement cette section pour éviter une erreur SQL.
    let orphaned_uploads: Vec<Upload> = sqlx::query_as(
        r#"
        SELECT u.* FROM uploads u
        LEFT JOIN messages m ON m.file_id = u.id
        WHERE m.id IS NULL
        "#,
    )
    .fetch_all(pool)
    .await?;

    // Supprimer les fichiers physiques
    for upload in &orphaned_uploads {
        let file_path = Path::new(&upload.file_path);
        if file_path.exists() {
            if let Err(e) = fs::remove_file(file_path).await {
                eprintln!(
                    "[Prune] Erreur suppression fichier physique {} : {}",
                    upload.id, e
                );
            } else {
                println!("[Prune] Fichier physique supprimé : {}", upload.file_path);
            }
        }
    }

    // Supprimer les lignes en base de données
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

    println!(
        "[Prune] {} uploads orphelins supprimés de la DB",
        deleted_uploads
    );

    // 3. Supprimer les conversations devenues vides
    let deleted_convos = sqlx::query(
        r#"
        DELETE FROM conversations
        WHERE NOT EXISTS (
            SELECT 1 FROM messages WHERE messages.conversation_id = conversations.id
        )
        RETURNING id
        "#,
    )
    .execute(pool)
    .await?
    .rows_affected();

    println!("[Prune] {} conversations vides supprimées", deleted_convos);

    // 4. Nettoyer les participants orphelins
    // → Nom de table corrigé : conversation_members (dans votre schéma actuel)
    //     au lieu de l'ancien conversation_participants
    sqlx::query(
        r#"
        DELETE FROM conversation_members
        WHERE NOT EXISTS (
            SELECT 1 FROM conversations WHERE conversations.id = conversation_members.conversation_id
        )
        "#
    )
    .execute(pool)
    .await?;

    Ok(())
}
