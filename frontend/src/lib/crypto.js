import sodium from 'libsodium-wrappers';

// Initialiser libsodium
await sodium.ready;

// Fonction d'initialisation (maintenue pour compatibilité)
export async function initSodium() {
  return sodium.ready;
}

/**
 * Génère une paire de clés cryptographiques (clé publique/privée)
 * @returns {Promise<{publicKey: string, privateKey: string}>}
 */
export async function generateKeyPair() {
    try {
        // Générer une paire de clés X25519 pour l'échange de clés
        const keypair = sodium.crypto_box_keypair();
        
        return {
            publicKey: sodium.to_base64(keypair.publicKey, sodium.base64_variants.URLSAFE_NO_PADDING),
            privateKey: sodium.to_base64(keypair.privateKey, sodium.base64_variants.URLSAFE_NO_PADDING)
        };
    } catch (error) {
        console.error('Erreur génération clés:', error);
        throw error;
    }

/**
 * Chiffre un message avec la clé publique du destinataire
 * @param {string} message - Message à chiffrer
 * @param {string} recipientPublicKey - Clé publique du destinataire (base64)
 * @param {string} senderPrivateKey - Clé privée de l'expéditeur (base64)
 * @returns {Promise<string>} Message chiffré en base64
 */
export async function encryptMessage(message, recipientPublicKey, senderPrivateKey) {
    try {
        const publicKey = sodium.from_base64(recipientPublicKey, sodium.base64_variants.URLSAFE_NO_PADDING);
        const privateKey = sodium.from_base64(senderPrivateKey, sodium.base64_variants.URLSAFE_NO_PADDING);
        
        // Générer un nonce aléatoire
        const nonce = sodium.randombytes_buf(sodium.crypto_box_NONCEBYTES);
        
        // Chiffrer le message
        const ciphertext = sodium.crypto_box_easy(
            sodium.from_string(message),
            nonce,
            publicKey,
            privateKey
        );
        
        // Combiner nonce + ciphertext et encoder en base64
        const combined = new Uint8Array(nonce.length + ciphertext.length);
        combined.set(nonce);
        combined.set(ciphertext, nonce.length);
        
        return sodium.to_base64(combined, sodium.base64_variants.URLSAFE_NO_PADDING);
    } catch (error) {
        console.error('Erreur chiffrement:', error);
        throw error;
    }
}

/**
 * Déchiffre un message avec la clé privée du destinataire
 * @param {string} encryptedMessage - Message chiffré (base64)
 * @param {string} senderPublicKey - Clé publique de l'expéditeur (base64)
 * @param {string} recipientPrivateKey - Clé privée du destinataire (base64)
 * @returns {Promise<string>} Message déchiffré
 */
export async function decryptMessage(encryptedMessage, senderPublicKey, recipientPrivateKey) {
    try {
        const publicKey = sodium.from_base64(senderPublicKey, sodium.base64_variants.URLSAFE_NO_PADDING);
        const privateKey = sodium.from_base64(recipientPrivateKey, sodium.base64_variants.URLSAFE_NO_PADDING);
        
        // Décoder le message
        const combined = sodium.from_base64(encryptedMessage, sodium.base64_variants.URLSAFE_NO_PADDING);
        
        // Extraire le nonce et le ciphertext
        const nonce = combined.slice(0, sodium.crypto_box_NONCEBYTES);
        const ciphertext = combined.slice(sodium.crypto_box_NONCEBYTES);
        
        // Déchiffrer
        const decrypted = sodium.crypto_box_open_easy(
            ciphertext,
            nonce,
            publicKey,
            privateKey
        );
        
        return sodium.to_string(decrypted);
    } catch (error) {
        console.error('Erreur déchiffrement:', error);
        throw error;
    }
}

/**
 * Stocke les clés de manière sécurisée dans localStorage
 * @param {string} memberId - ID du membre
 * @param {object} keys - Clés à stocker
 */
export function storeKeys(memberId, keys) {
    try {
        const data = {
            ...keys,
            timestamp: Date.now(),
            memberId
        };
        localStorage.setItem(`nook_keys_${memberId}`, JSON.stringify(data));
        console.log('Clés stockées pour:', memberId);
    } catch (error) {
        console.error('Erreur stockage clés:', error);
    }
}

/**
 * Récupère les clés depuis localStorage
 * @param {string} memberId - ID du membre
 * @returns {object|null} Clés stockées
 */
export function getStoredKeys(memberId) {
    try {
        const stored = localStorage.getItem(`nook_keys_${memberId}`);
        if (!stored) return null;
        
        return JSON.parse(stored);
    } catch (error) {
        console.error('Erreur récupération clés:', error);
        return null;
    }
}

/**
 * Génère un hash de mot de passe avec argon2 (via libsodium)
 * @param {string} password - Mot de passe à hasher
 * @returns {Promise<string>} Hash en base64
 */
export async function hashPassword(password) {
    try {
        // Utiliser pwhash avec des paramètres sécurisés
        const hash = sodium.crypto_pwhash_str(
            password,
            sodium.crypto_pwhash_OPSLIMIT_INTERACTIVE,
            sodium.crypto_pwhash_MEMLIMIT_INTERACTIVE
        );
        
        return hash;
    } catch (error) {
        console.error('Erreur hash password:', error);
        throw error;
    }
}

/**
 * Vérifie un mot de passe contre un hash
 * @param {string} hash - Hash stocké
 * @param {string} password - Mot de passe à vérifier
 * @returns {Promise<boolean>}
 */
export async function verifyPassword(hash, password) {
    try {
        return sodium.crypto_pwhash_str_verify(hash, password);
    } catch (error) {
        console.error('Erreur vérification password:', error);
        return false;
    }
}