/**
 * chatStore.svelte.ts — Store du chat, Svelte 5 Runes.
 *
 * RÈGLE SVELTE 5 : un objet $state exporté depuis un module .svelte.ts
 * est réactif partout où il est lu. On accède à chatStore.messages dans
 * les templates — jamais via un faux .subscribe().
 *
 * Chiffrement : envoi en clair pour l'instant (encrypted: false).
 * Le chiffrement E2E (libsodium) sera activé quand les clés par-user
 * seront gérées — les fonctions acceptent déjà les paramètres de clés.
 */

// -----------------------------------------------------------------
// 1️⃣ Types
// -----------------------------------------------------------------

export interface ChatMessage {
  id: string;
  conversation_id: string;
  sender_id: string;
  sender_name: string;   // COALESCE(users.name, users.username) via JOIN
  content: string;
  message_type: string;
  file_id: string | null;
  encrypted: boolean;
  nonce: string | null;
  timestamp: number;
  created_at: number;
  edited_at: number | null;
}

export interface ChatState {
  messages: ChatMessage[];
  connectionError: string | null;
  gifResults: any[];
  showGifs: boolean;
  gifLoading: boolean;
}

// -----------------------------------------------------------------
// 2️⃣ État réactif — UN objet $state, jamais réassigné
//    Le template lit chatStore.messages, chatStore.showGifs, etc.
// -----------------------------------------------------------------
export const chatStore = $state<ChatState>({
  messages: [],
  connectionError: null,
  gifResults: [],
  showGifs: false,
  gifLoading: false,
});

// -----------------------------------------------------------------
// 3️⃣ Actions
// -----------------------------------------------------------------

export function toggleGifs(): void {
  chatStore.showGifs = !chatStore.showGifs;
  if (!chatStore.showGifs) chatStore.gifResults = [];
}

// Alias pour compatibilité avec conversationStore qui l'importe
export function setConnectionError(err: string | null): void {
  chatStore.connectionError = err;
}

export function resetChat(): void {
  chatStore.messages = [];
  chatStore.connectionError = null;
  chatStore.showGifs = false;
  chatStore.gifResults = [];
  chatStore.gifLoading = false;
}

// -----------------------------------------------------------------
// 4️⃣ Formatage horodatage
// -----------------------------------------------------------------
export function formatTimestamp(ts: number): string {
  const date = new Date(ts * 1000); // secondes → ms
  const now  = new Date();
  if (date.toDateString() === now.toDateString()) {
    return date.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' });
  }
  if (date.getFullYear() === now.getFullYear()) {
    return date.toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
  }
  return date.toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' });
}

// -----------------------------------------------------------------
// 5️⃣ API — loadMessages
// -----------------------------------------------------------------
export async function loadMessages(conversationId: string): Promise<void> {
  try {
    const res = await fetch(
      `/api/conversations/${conversationId}/messages`,
      { credentials: 'include' }
    );
    if (!res.ok) throw new Error(`HTTP ${res.status}`);

    const data = await res.json();
    // Backend retourne Vec<MessageWithSender> — tableau direct
    const msgs: ChatMessage[] = Array.isArray(data) ? data : (data.messages ?? []);
    // Tri ASC par sécurité (le backend fait ORDER BY created_at ASC)
    msgs.sort((a, b) => a.created_at - b.created_at);
    chatStore.messages = msgs;
    chatStore.connectionError = null;
  } catch (err) {
    chatStore.connectionError = 'Erreur de chargement des messages';
    console.error('[Chat] loadMessages:', err);
  }
}

// -----------------------------------------------------------------
// 6️⃣ API — sendMessage (E2EE activé si cryptoStore.ready)
// -----------------------------------------------------------------
export async function sendMessage(
  content: string,
  conversationId: string
): Promise<void> {
  if (!content.trim()) return;
  try {
    const { cryptoStore: cs, encryptMessage } = await import('$lib/cryptoStore.svelte');
    let body: Record<string, unknown>;
    if (cs.ready) {
      try {
        const enc = await encryptMessage(content.trim(), conversationId);
        body = {
          content:        enc.ciphertext,
          encrypted:      true,
          nonce:          enc.nonce,
          encrypted_keys: enc.encryptedKeys,
        };
      } catch (e) {
        console.warn('[Chat] Chiffrement échoué, envoi en clair:', e);
        body = { content: content.trim(), encrypted: false };
      }
    } else {
      body = { content: content.trim(), encrypted: false };
    }
    const res = await fetch(`/api/conversations/${conversationId}/messages`, {
      method:      'POST',
      headers:     { 'Content-Type': 'application/json' },
      credentials: 'include',
      body:        JSON.stringify(body),
    });
    if (!res.ok) {
      const txt = await res.text().catch(() => '');
      throw new Error(`HTTP ${res.status}: ${txt}`);
    }
    await loadMessages(conversationId);
    chatStore.connectionError = null;
  } catch (err) {
    chatStore.connectionError = "Erreur lors de l'envoi du message";
    console.error('[Chat] sendMessage:', err);
  }
}

// -----------------------------------------------------------------
// 7️⃣ API — sendGif
// -----------------------------------------------------------------
export async function sendGif(
  gifUrl: string,
  conversationId: string
): Promise<void> {
  const content = `<img src="${gifUrl}" alt="GIF" class="chat-gif" />`;
  await sendMessage(content, conversationId);
}

// -----------------------------------------------------------------
// 8️⃣ API — searchGifs (Tenor)
// -----------------------------------------------------------------
export async function searchGifs(query: string): Promise<void> {
  if (!query.trim()) return;
  try {
    chatStore.gifLoading = true;
    chatStore.gifResults = [];
    const res = await fetch(
      `https://tenor.googleapis.com/v2/search?q=${encodeURIComponent(query)}&key=LIVDSRZULELA&client_key=nook&limit=12`
    );
    if (!res.ok) throw new Error('Tenor error');
    const data = await res.json();
    chatStore.gifResults = data.results ?? [];
  } catch (err) {
    console.error('[Chat] searchGifs:', err);
  } finally {
    chatStore.gifLoading = false;
  }
}
