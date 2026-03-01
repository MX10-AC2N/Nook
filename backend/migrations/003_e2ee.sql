-- migrations/003_e2ee.sql
-- Chiffrement de bout en bout — architecture "clé de session par message"
--
-- Modèle :
--   • messages.nonce          : nonce XSalsa20 (base64, 24 bytes) — NULL = message en clair
--   • message_keys            : clé de session chiffrée par destinataire (crypto_box_easy)
--   • users.public_key        : déjà présent en 001_initial.sql (TEXT, base64)
--
-- Flux d'envoi :
--   1. Expéditeur génère une session_key (32 bytes aléatoires)
--   2. Chiffre le texte : crypto_secretbox_easy(msg, nonce, session_key)
--   3. Pour chaque destinataire : crypto_box_easy(session_key, asymNonce, recipientPubkey, senderPrivkey)
--   4. POST /api/conversations/{id}/messages avec content=ciphertext, nonce=nonce, encrypted_keys={uid: ...}
--   5. Backend insère dans messages + message_keys
--
-- Flux de réception :
--   1. GET /api/conversations/{id}/my-encrypted-key/{msg_id} → encrypted_key pour moi
--   2. Déchiffrer session_key avec ma clé privée
--   3. Déchiffrer le message avec session_key + nonce

-- ─────────────────────────────────────────────────────────────────────────────
-- 1. Ajouter nonce sur messages (NULL = message legacy en clair)
-- ─────────────────────────────────────────────────────────────────────────────
ALTER TABLE messages ADD COLUMN nonce TEXT;

-- ─────────────────────────────────────────────────────────────────────────────
-- 2. Table message_keys — clé de session chiffrée par destinataire
--    encrypted_key = base64(asymNonce[24] || crypto_box_easy_ciphertext)
--    Taille totale : 24 + 32 + 16 (overhead box) = 72 bytes → base64 ~96 chars
-- ─────────────────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS message_keys (
    message_id     TEXT    NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    recipient_id   TEXT    NOT NULL REFERENCES users(id)    ON DELETE CASCADE,
    encrypted_key  TEXT    NOT NULL,  -- base64(asymNonce || box_ciphertext)
    PRIMARY KEY (message_id, recipient_id)
);

CREATE INDEX IF NOT EXISTS idx_message_keys_recipient
    ON message_keys (recipient_id, message_id);
