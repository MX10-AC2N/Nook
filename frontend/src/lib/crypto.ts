// Utility functions for cryptography and security
import libsodium from 'libsodium-wrappers';

// Types
export interface KeyPair {
    publicKey: string;
    privateKey: string;
}

export interface StoredKeys extends KeyPair {
    timestamp: number;
    memberId: string;
}

// Initialize libsodium
let sodiumInitialized = false;

/**
 * Initialize libsodium - must be called before any crypto operations
 */
export async function initSodium(): Promise<void> {
    if (!sodiumInitialized) {
        await libsodium.ready;
        sodiumInitialized = true;
    }
}

/**
 * Get initialized sodium instance
 */
async function getSodium() {
    await initSodium();
    return libsodium;
}

/**
 * Generate a new X25519 key pair for E2EE
 */
export async function generateKeyPair(): Promise<KeyPair> {
    const sodium = await getSodium();
    const keypair = sodium.crypto_box_keypair();
    return {
        publicKey: sodium.to_base64(keypair.publicKey, sodium.base64_variants.URLSAFE_NO_PADDING),
        privateKey: sodium.to_base64(keypair.privateKey, sodium.base64_variants.URLSAFE_NO_PADDING)
    };
}

/**
 * Encrypt a message for a recipient using their public key
 */
export async function encryptMessage(
    plaintext: string,
    recipientPublicKey: string,
    senderPrivateKey: string
): Promise<string> {
    const sodium = await getSodium();
    const recipientKey = sodium.from_base64(recipientPublicKey, sodium.base64_variants.URLSAFE_NO_PADDING);
    const senderKey = sodium.from_base64(senderPrivateKey, sodium.base64_variants.URLSAFE_NO_PADDING);
    
    const nonce = sodium.randombytes_buf(sodium.crypto_box_NONCEBYTES);
    const ciphertext = sodium.crypto_box_easy(
        sodium.from_string(plaintext),
        nonce,
        recipientKey,
        senderKey
    );
    
    const combined = new Uint8Array(nonce.length + ciphertext.length);
    combined.set(nonce);
    combined.set(ciphertext, nonce.length);
    
    return sodium.to_base64(combined, sodium.base64_variants.URLSAFE_NO_PADDING);
}

/**
 * Decrypt a message using the recipient's private key
 */
export async function decryptMessage(
    encryptedMessage: string,
    senderPublicKey: string,
    recipientPrivateKey: string
): Promise<string> {
    const sodium = await getSodium();
    const senderKey = sodium.from_base64(senderPublicKey, sodium.base64_variants.URLSAFE_NO_PADDING);
    const recipientKey = sodium.from_base64(recipientPrivateKey, sodium.base64_variants.URLSAFE_NO_PADDING);
    
    const combined = sodium.from_base64(encryptedMessage, sodium.base64_variants.URLSAFE_NO_PADDING);
    const nonce = combined.slice(0, sodium.crypto_box_NONCEBYTES);
    const ciphertext = combined.slice(sodium.crypto_box_NONCEBYTES);
    
    const plaintext = sodium.crypto_box_open_easy(
        ciphertext,
        nonce,
        senderKey,
        recipientKey
    );
    
    return sodium.to_string(plaintext);
}

/**
 * Hash a password for storage (using Argon2id via libsodium)
 */
export async function hashPassword(password: string): Promise<string> {
    const sodium = await getSodium();
    return sodium.crypto_pwhash_str(
        password,
        sodium.crypto_pwhash_OPSLIMIT_INTERACTIVE,
        sodium.crypto_pwhash_MEMLIMIT_INTERACTIVE
    );
}

/**
 * Verify a password against a stored hash
 */
export async function verifyPassword(hash: string, password: string): Promise<boolean> {
    const sodium = await getSodium();
    return sodium.crypto_pwhash_str_verify(hash, password);
}

/**
 * Store keys securely in localStorage
 */
export function storeKeys(memberId: string, keys: KeyPair): void {
    const data: StoredKeys = {
        ...keys,
        timestamp: Date.now(),
        memberId
    };
    localStorage.setItem(`nook_keys_${memberId}`, JSON.stringify(data));
}

/**
 * Retrieve stored keys from localStorage
 */
export function getStoredKeys(memberId: string): KeyPair | null {
    const stored = localStorage.getItem(`nook_keys_${memberId}`);
    return stored ? JSON.parse(stored) : null;
}