import sodium from 'libsodium-wrappers';
import { get, set } from 'idb-keyval'; // npm i idb-keyval (ou ton wrapper)

let ready = false;
async function initSodium() {
  if (!ready) { await sodium.ready; ready = true; }
}

export interface KeyPair {
  publicKey: Uint8Array;
  privateKey: Uint8Array;
}

export class E2EE {
  private myKeys: KeyPair | null = null;
  private groupKeys = new Map<string, Uint8Array>(); // convoId → groupKey (en mémoire)

  async init(userId: string) {
    await initSodium();
    const stored = await get(`e2ee_keys_${userId}`);
    if (stored) {
      this.myKeys = stored;
    } else {
      const kp = sodium.crypto_box_keypair();
      this.myKeys = kp;
      await set(`e2ee_keys_${userId}`, kp);
      await this.uploadPublicKey(kp.publicKey);
    }
  }

  private async uploadPublicKey(pub: Uint8Array) {
    await fetch('/api/auth/public-key', {
      method: 'POST',
      body: pub,
      headers: { 'Content-Type': 'application/octet-stream' }
    });
  }

  // === Distribution clé de groupe (appelée par créateur ou admin) ===
  async distributeGroupKey(convoId: string, memberPubkeys: Record<string, string>) { // userId → base64 pubkey
    await initSodium();
    const groupKey = sodium.randombytes_buf(32);

    const distributions: Record<string, string> = {};
    for (const [uid, pubB64] of Object.entries(memberPubkeys)) {
      const sealed = sodium.crypto_box_seal(groupKey, sodium.from_base64(pubB64));
      distributions[uid] = sodium.to_base64(sealed);
    }

    await fetch(`/api/conversations/${convoId}/keys`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ distributions, keyVersion: 1 })
    });

    this.groupKeys.set(convoId, groupKey);
    return groupKey;
  }

  // Récupérer sa clé de groupe (au chargement d’une convo)
  async loadGroupKey(convoId: string) {
    if (this.groupKeys.has(convoId)) return this.groupKeys.get(convoId)!;

    const res = await fetch(`/api/conversations/${convoId}/my-key`);
    const data = await res.json(); // {encryptedKey: base64, keyVersion: number}
    const sealed = sodium.from_base64(data.encryptedKey);

    if (!this.myKeys) throw new Error('No keys');
    const groupKey = sodium.crypto_box_seal_open(sealed, this.myKeys.publicKey, this.myKeys.privateKey);

    this.groupKeys.set(convoId, groupKey);
    return groupKey;
  }

  // === Chiffrement / déchiffrement message ===
  async encryptMessage(text: string, convoId: string): Promise<{ciphertext: string; nonce: string}> {
    const groupKey = this.groupKeys.get(convoId) || await this.loadGroupKey(convoId);
    await initSodium();

    const msg = new TextEncoder().encode(text);
    const nonce = sodium.randombytes_buf(sodium.crypto_aead_xchacha20poly1305_ietf_NPUBBYTES);

    const ct = sodium.crypto_aead_xchacha20poly1305_ietf_encrypt(
      msg, null, null, nonce, groupKey
    );

    return {
      ciphertext: sodium.to_base64(ct),
      nonce: sodium.to_base64(nonce)
    };
  }

  async decryptMessage(ciphertextB64: string, nonceB64: string, convoId: string): Promise<string> {
    const groupKey = this.groupKeys.get(convoId) || await this.loadGroupKey(convoId);
    await initSodium();

    const ct = sodium.from_base64(ciphertextB64);
    const nonce = sodium.from_base64(nonceB64);

    const decrypted = sodium.crypto_aead_xchacha20poly1305_ietf_decrypt(
      null, ct, null, nonce, groupKey
    );
    return new TextDecoder().decode(decrypted);
  }
async addMemberToConversation(convoId: string, newMemberPubkeyB64: string, newMemberUserId: string) {
  await initSodium();
  const groupKey = this.groupKeys.get(convoId) || await this.loadGroupKey(convoId);

  const sealed = sodium.crypto_box_seal(groupKey, sodium.from_base64(newMemberPubkeyB64));
  const payload = {
    userId: newMemberUserId,
    encryptedKey: sodium.to_base64(sealed),
    keyVersion: this.currentVersion(convoId) // à stocker aussi
  };

  await fetch(`/api/conversations/${convoId}/add-member-key`, {
    method: 'POST',
    body: JSON.stringify(payload)
  });
}

async rotateGroupKey(convoId: string, remainingMembers: Record<string, string>) {
  const newGroupKey = sodium.randombytes_buf(32);
  // même logique de distribution que avant, mais avec newGroupKey
  // + mise à jour de la version
  this.groupKeys.set(convoId, newGroupKey);
}


}
// Utilisation dans ton store chat :
const e2ee = new E2EE();
// Dans +layout.svelte ou après login : await e2ee.init(currentUser.id);