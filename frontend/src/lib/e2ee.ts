// src/lib/e2ee.ts
//
// Gestion des clés de groupe (Group Key) pour conversations de groupe.
//
// Architecture E2EE Nook — deux niveaux :
//   1. Niveau groupe  (ce module) : clé symétrique partagée par tous les membres
//      Distribuée via crypto_box_seal (chiffrement asynchrone X25519).
//      Utilisée pour qualifier les messages (sender_key_version, ré-encryption).
//   2. Niveau message (crypto.ts) : clé de session éphémère par message,
//      chiffrée pour chaque destinataire via crypto_box_easy.
//
// Ce module remplace l'ancien prototype qui utilisait des imports directs sodium,
// idb-keyval et crypto_aead_xchacha20poly1305.
//
// API publique maintenue pour compatibilité :
//   e2ee.getGroupKey(convId)     → Uint8Array | undefined
//   e2ee.loadGroupKey(convId)    → Promise<Uint8Array>
//   e2ee.currentVersion(convId)  → number
//   e2ee.distributeGroupKey(convId, memberPubkeys)
//   e2ee.addMemberToConversation(convId, newPubkeyB64, newUserId)
//   e2ee.rotateGroupKey(convId, remainingMembers)
// ─────────────────────────────────────────────────────────────────────────────

import { apiFetch } from './api';

// Lazy-loaded sodium via shared helper
import { waitForSodium, isSodiumReady, getSodiumInstance } from './sodium.svelte.js';

// Type Sodium — le type réel est fourni par @types/libsodium-wrappers
// qui exporte chaque fonction individuellement. Pour nos appels asynchrones
// on utilise un type approximatif.
// eslint-disable-next-line @typescript-eslint/no-explicit-any
type SodiumType = any;

async function ensureSodium(): Promise<SodiumType> {
  const existing = getSodiumInstance();
  if (existing) return existing;
  return await waitForSodium();
}

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

export interface KeyPair {
  publicKey: Uint8Array;
  privateKey: Uint8Array;
}

interface GroupKeyEntry {
  key: Uint8Array;
  version: number;
}

// ─────────────────────────────────────────────────────────────────────────────
// Classe E2EE — clés de groupe
// ─────────────────────────────────────────────────────────────────────────────

export class E2EE {
  private myKeys: KeyPair | null = null;
  private groupKeys = new Map<string, GroupKeyEntry>(); // convoId → { key, version }
  private myUserId: string | null = null;

  // ── Initialisation ────────────────────────────────────────────────────────
  async init(userId: string): Promise<void> {
    await ensureSodium();
    this.myUserId = userId;
    // On ne stocke pas nos clés ici — elles sont gérées par cryptoStore/crypto.ts
    // via IndexedDB. Si cryptoStore est ready, on peut les emprunter.
    console.info('[e2ee] Initialisé pour', userId);
  }

  /** Injecte la paire de clés depuis cryptoStore (appelé après unlockCrypto) */
  setKeyPair(kp: KeyPair): void {
    this.myKeys = kp;
  }

  /** Vérifie si les clés sont chargées */
  hasKeys(): boolean {
    return this.myKeys !== null;
  }

  // ── API publique pour file-transfer.svelte.ts ────────────────────────────
  // Ces méthodes sont utilisées par le module de transfert de fichiers.

  /** Retourne la clé de groupe en cache pour une conversation */
  getGroupKey(convoId: string): Uint8Array | undefined {
    return this.groupKeys.get(convoId)?.key;
  }

  /** Charge la clé de groupe depuis le serveur (si pas en cache) */
  async loadGroupKey(convoId: string): Promise<Uint8Array> {
    const cached = this.groupKeys.get(convoId);
    if (cached) return cached.key;

    const na = await ensureSodium();
    const data = await apiFetch<{ encryptedKey: string; keyVersion: number }>(
      `/api/conversations/${convoId}/my-key`
    );

    if (!this.myKeys) throw new Error('[e2ee] loadGroupKey: clés non chargées');

    const sealed = na.from_base64(data.encryptedKey, na.base64_variants.ORIGINAL);
    const groupKey = na.crypto_box_seal_open(
      sealed,
      this.myKeys.publicKey,
      this.myKeys.privateKey
    );

    this.groupKeys.set(convoId, { key: groupKey, version: data.keyVersion });
    return groupKey;
  }

  /** Version actuelle de la clé de groupe pour une conversation */
  currentVersion(convoId: string): number {
    return this.groupKeys.get(convoId)?.version ?? 1;
  }

  // ── Distribution initiale ────────────────────────────────────────────────
  /**
   * Distribue une nouvelle clé de groupe à tous les membres.
   * Appelée par le créateur/admin au démarrage d'une conversation de groupe.
   */
  async distributeGroupKey(
    convoId: string,
    memberPubkeys: Record<string, string> // userId → base64 pubKey
  ): Promise<Uint8Array> {
    const na = await ensureSodium();
    const groupKey = na.randombytes_buf(32);

    const version = (this.groupKeys.get(convoId)?.version ?? 0) + 1;
    const distributions: Record<string, string> = {};

    for (const [uid, pubB64] of Object.entries(memberPubkeys)) {
      try {
        const pub = na.from_base64(pubB64, na.base64_variants.ORIGINAL);
        const sealed = na.crypto_box_seal(groupKey, pub);
        distributions[uid] = na.to_base64(sealed, na.base64_variants.ORIGINAL);
      } catch (e) {
        // BUG-006 FIX: retourner erreur explicite si membre sans clé invalide
        throw new Error(`[e2ee] Échec distribution clé pour ${uid} — clé publique invalide ou manquante : ${e instanceof Error ? e.message : String(e)}`);
      }
    }

    await apiFetch(`/api/conversations/${convoId}/keys`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ distributions, keyVersion: version }),
    });

    this.groupKeys.set(convoId, { key: groupKey, version });
    return groupKey;
  }

  // ── Ajout d'un membre ────────────────────────────────────────────────────
  /**
   * Ajoute un nouveau membre à une conversation en lui partageant la clé de groupe.
   */
  async addMemberToConversation(
    convoId: string,
    newMemberPubkeyB64: string,
    newMemberUserId: string
  ): Promise<void> {
    const na = await ensureSodium();
    const groupKey = this.groupKeys.get(convoId)?.key ?? await this.loadGroupKey(convoId);

    const pub = na.from_base64(newMemberPubkeyB64, na.base64_variants.ORIGINAL);
    const sealed = na.crypto_box_seal(groupKey, pub);

    await apiFetch(`/api/conversations/${convoId}/add-member-key`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        userId: newMemberUserId,
        encryptedKey: na.to_base64(sealed, na.base64_variants.ORIGINAL),
        keyVersion: this.currentVersion(convoId),
      }),
    });

    console.info('[e2ee] Membre ajouté:', newMemberUserId, 'version:', this.currentVersion(convoId));
  }

  // ── Rotation de la clé de groupe ─────────────────────────────────────────
  /**
   * Rotation de la clé de groupe pour une conversation.
   * Génère une nouvelle clé et la distribue aux membres restants.
   * Utile quand un membre quitte ou que la clé est compromise.
   */
  async rotateGroupKey(
    convoId: string,
    remainingMembers: Record<string, string> // userId → base64 pubKey
  ): Promise<Uint8Array> {
    const na = await ensureSodium();
    const newGroupKey = na.randombytes_buf(32);

    const version = (this.groupKeys.get(convoId)?.version ?? 0) + 1;
    const distributions: Record<string, string> = {};

    for (const [uid, pubB64] of Object.entries(remainingMembers)) {
      try {
        const pub = na.from_base64(pubB64, na.base64_variants.ORIGINAL);
        const sealed = na.crypto_box_seal(newGroupKey, pub);
        distributions[uid] = na.to_base64(sealed, na.base64_variants.ORIGINAL);
      } catch (e) {
        console.warn('[e2ee] Échec rotation pour', uid, e);
      }
    }

    await apiFetch(`/api/conversations/${convoId}/keys`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ distributions, keyVersion: version }),
    });

    this.groupKeys.set(convoId, { key: newGroupKey, version });
    console.info('[e2ee] Rotation clé de groupe pour', convoId, 'version:', version);
    return newGroupKey;
  }

  // ── Utilitaires ──────────────────────────────────────────────────────────

  /** Efface les clés de groupe (logout) */
  clear(): void {
    this.myKeys = null;
    this.groupKeys.clear();
    this.myUserId = null;
  }
}

// ── Singleton exporté ─────────────────────────────────────────────────────────
// Compatibilité : utilisé par file-transfer.svelte.ts
const e2ee = new E2EE();
export { e2ee };
