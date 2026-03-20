-- 006_push.sql — Abonnements push PWA
-- Session 37 : notifications push via Web Push API
-- Chaque device subscribes une fois → stocké ici → backend envoie via web-push
--
-- Pas de dépendance aux autres tables (sauf users).
-- TTL géré par prune.rs (supprimer les subscriptions > 90 jours sans activité).

CREATE TABLE IF NOT EXISTS push_subscriptions (
    id          TEXT PRIMARY KEY,          -- UUID v4
    user_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    endpoint    TEXT NOT NULL UNIQUE,      -- URL push unique par device
    p256dh      TEXT NOT NULL,             -- clé publique du browser (base64url)
    auth        TEXT NOT NULL,             -- secret d'auth du browser (base64url)
    user_agent  TEXT,                      -- info device (optionnel, debug)
    created_at  INTEGER NOT NULL,          -- Unix timestamp
    last_used   INTEGER                    -- dernière notification envoyée
);

CREATE INDEX IF NOT EXISTS idx_push_user_id ON push_subscriptions(user_id);

-- Table des préférences de notification par utilisateur
CREATE TABLE IF NOT EXISTS push_preferences (
    user_id         TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    enabled         INTEGER NOT NULL DEFAULT 1,  -- 1 = notifs actives
    quiet_start     TEXT DEFAULT '22:00',         -- début période silencieuse (HH:MM)
    quiet_end       TEXT DEFAULT '07:00',         -- fin période silencieuse
    on_message      INTEGER NOT NULL DEFAULT 1,  -- notif sur nouveau message
    on_mention      INTEGER NOT NULL DEFAULT 1,  -- notif si mentionné (@user)
    updated_at      INTEGER NOT NULL
);
