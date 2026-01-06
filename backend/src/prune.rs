use sqlx::{SqlitePool, Error};
use tokio::fs;
use std::path::Path;
use crate::db::Upload;  // Importe ta struct Upload depuis db.rs

pub async fn prune_old_data(pool: &SqlitePool) -> Result<(), Error> {
    let seven_days_ago = chrono::Utc::now().timestamp() - (7 * 24 * 3600);

    // 1. Supprimer les messages > 7 jours (hard delete direct)
    let deleted_messages = sqlx::query("DELETE FROM messages WHERE timestamp < ?")
        .bind(seven_days_ago)
        .execute(pool)
        .await?
        .rows_affected();

    println!("[Prune] {} messages anciens supprimés (hard delete)", deleted_messages);

    // 2. Récupérer tous les uploads qui deviennent orphelins
    // (pas référencés par un message restant)
    let orphaned_uploads: Vec<Upload> = sqlx::query_as(
        r#"
        SELECT u.* FROM uploads u
        LEFT JOIN messages m ON m.file_id = u.id
        WHERE m.id IS NULL
        "#
    )
    .fetch_all(pool)
    .await?;

    // Supprimer les fichiers physiques
    for upload in &orphaned_uploads {
        let file_path = Path::new(&upload.file_path);
        if file_path.exists() {
            if let Err(e) = fs::remove_file(file_path).await {
                eprintln!("[Prune] Erreur suppression fichier physique {} : {}", upload.id, e);
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
        "#
    )
    .execute(pool)
    .await?
    .rows_affected();

    println!("[Prune] {} uploads orphelins supprimés de la DB", deleted_uploads);

    // 3. Supprimer les conversations devenues vides
    let deleted_convos = sqlx::query(
        r#"
        DELETE FROM conversations
        WHERE NOT EXISTS (
            SELECT 1 FROM messages WHERE messages.conversation_id = conversations.id
        )
        RETURNING id
        "#
    )
    .execute(pool)
    .await?
    .rows_affected();

    println!("[Prune] {} conversations vides supprimées", deleted_convos);

    // 4. Nettoyer les participants orphelins (bon pour la propreté)
    sqlx::query(
        r#"
        DELETE FROM conversation_participants
        WHERE NOT EXISTS (
            SELECT 1 FROM conversations WHERE conversations.id = conversation_participants.conversation_id
        )
        "#
    )
    .execute(pool)
    .await?;

    Ok(())
}
