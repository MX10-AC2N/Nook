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

    // ─── 5. Parties d'échecs anciennes terminées/abandonnées (> 7 jours) ──────
    let deleted_chess = sqlx::query(
        r#"
        DELETE FROM chess_games
        WHERE (status = 'finished' OR status = 'abandoned')
          AND updated_at < ?
        "#
    )
    .bind(seven_days_ago)
    .execute(pool)
    .await?
    .rows_affected();

    if deleted_chess > 0 {
        tracing::info!(
            count = deleted_chess,
            "Prune : parties d'échecs terminées/abandonnées supprimées"
        );
    }

    // ─── 6. Purge des anciennes versions de clés de groupe (E2EE / DT-05, carte t_1862cb6f) ──
    // Politique de rétention (NOUVELLE décision, citée ADR-016 §QO#2 + DECISIONS.md §2) :
    // conserver les 3 versions les plus récentes par conversation ET les versions de moins de
    // 21 jours. Au-delà, suppression SI les 3 garanties de sécurité suivantes sont réunies :
    //   (a) ce n'est PAS la version courante (max) pour la conversation  → jamais supprimer la courante
    //   (b) ce n'est PAS l'une des 3 versions les plus récentes           → borne de profondeur
    //   (c) aucune version de message non purgé ne la référence           → cohérence fenêtre avec §2 (messages→7j)
    // SAFETY : (a) et (c) garantissent qu'aucun membre courant ne perd la capacité de
    // déchiffrer un message survivant. Une purge qui violerait (c) briserait l'historique.
    let now = chrono::Utc::now().timestamp();
    let key_retention_days: i64 = 21;
    let key_retention_versions: i64 = 3;
    let key_cutoff = now - key_retention_days * 24 * 3600;
    // Fenêtre de référence = fenêtre de purge des messages (§2, 7 j). Un message dont
    // created_at >= seven_days_ago est considéré comme survivant → sa clé de groupe est nécessaire.
    let deleted_recipients = sqlx::query(
        r#"
        DELETE FROM conversation_key_recipients
        WHERE (conversation_id, version) IN (
            SELECT ck.conversation_id, ck.version
            FROM conversation_keys ck
            WHERE ck.created_at < ?
              AND ck.version < (
                  SELECT MAX(ck2.version)
                  FROM conversation_keys ck2
                  WHERE ck2.conversation_id = ck.conversation_id
              )
              AND ck.version NOT IN (
                  SELECT version FROM conversation_keys ck3
                  WHERE ck3.conversation_id = ck.conversation_id
                  ORDER BY version DESC
                  LIMIT ?
              )
              AND NOT EXISTS (
                  SELECT 1 FROM messages m
                  WHERE m.conversation_id = ck.conversation_id
                    AND m.group_key_version = ck.version
                    AND m.created_at >= ?
              )
        )"#,
    )
    .bind(key_cutoff)
    .bind(key_retention_versions)
    .bind(seven_days_ago)
    .execute(pool)
    .await?
    .rows_affected();

    let deleted_keys = sqlx::query(
        r#"
        DELETE FROM conversation_keys
        WHERE (conversation_id, version) IN (
            SELECT ck.conversation_id, ck.version
            FROM conversation_keys ck
            WHERE ck.created_at < ?
              AND ck.version < (
                  SELECT MAX(ck2.version)
                  FROM conversation_keys ck2
                  WHERE ck2.conversation_id = ck.conversation_id
              )
              AND ck.version NOT IN (
                  SELECT version FROM conversation_keys ck3
                  WHERE ck3.conversation_id = ck.conversation_id
                  ORDER BY version DESC
                  LIMIT ?
              )
              AND NOT EXISTS (
                  SELECT 1 FROM messages m
                  WHERE m.conversation_id = ck.conversation_id
                    AND m.group_key_version = ck.version
                    AND m.created_at >= ?
              )
        )"#,
    )
    .bind(key_cutoff)
    .bind(key_retention_versions)
    .bind(seven_days_ago)
    .execute(pool)
    .await?
    .rows_affected();

    if deleted_recipients > 0 || deleted_keys > 0 {
        tracing::info!(
            recipients = deleted_recipients,
            keys = deleted_keys,
            version_window = key_retention_versions,
            time_window_days = key_retention_days,
            "Prune : versions de clés de groupe obsolètes purgées (cohérence 7j messages)"
        );
    }

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests — purge des versions de clé de groupe (E2 correctif, carte t_1862cb6f)
// ─────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use sqlx::SqlitePool;
    use sqlx::sqlite::SqliteConnectOptions;
    use std::str::FromStr;

    fn now() -> i64 {
        chrono::Utc::now().timestamp()
    }
    const DAY: i64 = 24 * 3600;

    async fn setup_test_db() -> SqlitePool {
        let opts = SqliteConnectOptions::from_str("sqlite::memory:")
            .unwrap()
            .create_if_missing(true)
            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
            .foreign_keys(false);
        let pool = SqlitePool::connect_with(opts).await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        pool
    }

    async fn seed_conv(pool: &SqlitePool, conv_id: &str) {
        let t = now();
        sqlx::query(
            "INSERT INTO conversations (id, name, is_group, created_by, created_at, updated_at)
             VALUES (?, ?, 1, ?, ?, ?)",
        )
        .bind(conv_id)
        .bind("G")
        .bind(conv_id)
        .bind(t)
        .bind(t)
        .execute(pool)
        .await
        .unwrap();
    }

    /// Insère une version de clé de groupe (created_at contrôlé) + un recipient pour "user_1".
    async fn add_key_version(pool: &SqlitePool, conv_id: &str, version: i32, created_at: i64) {
        // creator_id / user_id sont des UUIDs factices : les FK SQLite ne sont pas
        // activées dans la DB de test (aucun PRAGMA foreign_keys dans les migrations).
        let creator = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO conversation_keys (conversation_id, version, creator_id, created_at)
             VALUES (?, ?, ?, ?)",
        )
        .bind(conv_id)
        .bind(version)
        .bind(&creator)
        .bind(created_at)
        .execute(pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO conversation_key_recipients
                (conversation_id, version, user_id, encrypted_key, user_key_version, distribution_status)
             VALUES (?, ?, ?, ?, ?, 'delivered')",
        )
        .bind(conv_id)
        .bind(version)
        .bind("user_1")
        .bind(format!("enc-v{}", version))
        .bind(1)
        .execute(pool)
        .await
        .unwrap();
    }

    async fn add_message(
        pool: &SqlitePool,
        msg_id: &str,
        conv_id: &str,
        created_at: i64,
        group_key_version: Option<i32>,
    ) {
        sqlx::query(
            "INSERT INTO messages
                (id, conversation_id, sender_id, content, message_type, encrypted, timestamp, created_at, group_key_version)
             VALUES (?, ?, ?, ?, 'text', 0, ?, ?, ?)",
        )
        .bind(msg_id)
        .bind(conv_id)
        .bind("sender_x")
        .bind("m")
        .bind(created_at)
        .bind(created_at)
        .bind(group_key_version)
        .execute(pool)
        .await
        .unwrap();
    }

    async fn remaining_versions(pool: &SqlitePool, conv_id: &str) -> Vec<i32> {
        sqlx::query_scalar::<_, i32>("SELECT version FROM conversation_keys WHERE conversation_id = ? ORDER BY version")
            .bind(conv_id)
            .fetch_all(pool)
            .await
            .unwrap()
    }

    async fn recipient_exists(
        pool: &SqlitePool,
        conv_id: &str,
        version: i32,
        user_id: &str,
    ) -> bool {
        let count: Option<(i64,)> = sqlx::query_as(
            "SELECT COUNT(*) FROM conversation_key_recipients
             WHERE conversation_id = ? AND version = ? AND user_id = ?",
        )
        .bind(conv_id)
        .bind(version)
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .unwrap();
        count.map(|(c,)| c > 0).unwrap_or(false)
    }

    // ── Cas 1 : purge ciblée tout en préservant la sécurité ─────────────────
    #[tokio::test]
    async fn test_prune_purges_old_unreferenced_versions_but_safe() {
        let pool = setup_test_db().await;
        let conv_id = "conv-prune";
        seed_conv(&pool, conv_id).await;

        // 6 versions, TOUTES créées 30 jours plus tôt (au-delà de la borne de 21 j).
        // latest = 6, top-3 = {4,5,6} → versions éligibles (hors top-3, anciennes, non-latest) = {1,2,3}
        for v in 1..=6i32 {
            add_key_version(&pool, conv_id, v, now() - 30 * DAY).await;
        }

        // Message SURVIVANT (1j) référençant v2 → v2 doit SURVIVRE (gardien de sécurité)
        add_message(&pool, "m_alive", conv_id, now() - DAY, Some(2)).await;
        // Message ANCIEN (30j → purgé par la section 2) référençant v3 → v3 devient
        // sans référence vivante → PURGÉE
        add_message(&pool, "m_dead", conv_id, now() - 30 * DAY, Some(3)).await;

        crate::prune::prune_old_data(&pool).await.unwrap();

        let remaining = remaining_versions(&pool, conv_id).await;
        // v2 : éligible mais référencé par un message survivant → conservé
        assert!(remaining.contains(&2), "v2 doit survivre — référencé par un message vivant");
        // v4, v5, v6 : top-3 / latest → conservés
        assert!(remaining.contains(&4));
        assert!(remaining.contains(&5));
        assert!(remaining.contains(&6), "la version courante (latest) doit toujours survivre");
        // v1, v3 : anciennes, hors top-3, non référencées par un message vivant → purgées
        assert!(!remaining.contains(&1), "v1 (ancienne, non référencée, hors top-3) doit être purgée");
        assert!(
            !remaining.contains(&3),
            "v3 (référence uniquement par un message déjà purgé) doit être purgée"
        );

        // Les recipients suivent les clés purgées
        assert!(
            !recipient_exists(&pool, conv_id, 1, "user_1").await,
            "recipients de la version purgée v1 doivent disparaître"
        );
        assert!(
            recipient_exists(&pool, conv_id, 2, "user_1").await,
            "recipients de v2 (conservé) doivent rester"
        );
    }

    // ── Cas 2 : jamais purger la version courante ────────────────────────
    #[tokio::test]
    async fn test_prune_never_deletes_latest_version() {
        let pool = setup_test_db().await;
        let conv_id = "conv-latest";
        seed_conv(&pool, conv_id).await;
        // Une seule version, ancienne → conservée car c'est la version courante (max)
        add_key_version(&pool, conv_id, 1, now() - 30 * DAY).await;
        add_message(&pool, "m1", conv_id, now() - 30 * DAY, Some(1)).await;
        crate::prune::prune_old_data(&pool).await.unwrap();
        let remaining = remaining_versions(&pool, conv_id).await;
        assert_eq!(remaining, vec![1], "la version courante doit survivre même si ancienne");
    }

    // ── Cas 3 : clés récentes intouchées ─────────────────────────────────
    #[tokio::test]
    async fn test_prune_leaves_recent_keys_untouched() {
        let pool = setup_test_db().await;
        let conv_id = "conv-fresh";
        seed_conv(&pool, conv_id).await;
        add_key_version(&pool, conv_id, 1, now()).await; // fraîche
        crate::prune::prune_old_data(&pool).await.unwrap();
        let remaining = remaining_versions(&pool, conv_id).await;
        assert_eq!(remaining, vec![1], "clé récente doit rester intouchée");
    }
}
