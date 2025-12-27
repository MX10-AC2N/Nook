import { writable, derived, get } from 'svelte/store';
import { browser } from '$app/environment';
import { authStore } from './authStore';
import { encryptForRecipients, decryptMessage, getStoredKeys, decryptPrivateKey } from './crypto';
import type { Message } from './types';

export const currentUser = writable<{ id: string; name: string; username: string; publicKey?: Uint8Array } | null>(null);
export const messages = writable<Message[]>([]);

let chatState = $state({
	messages: [] as Message[],
	connectionError: null as string | null,
	gifResults: [] as any[],
	showGifs: false,
	gifLoading: false
});

export const connectionError = derived({ subscribe: () => get(messages) }, () => chatState.connectionError);
export const gifResults = derived({ subscribe: () => get(messages) }, () => chatState.gifResults);
export const showGifs = derived({ subscribe: () => get(messages) }, () => chatState.showGifs);
export const gifLoading = derived({ subscribe: () => get(messages) }, () => chatState.gifLoading);

export function setMessages(msgs: Message[]) {
	chatState.messages = msgs;
	messages.set(msgs);
}

export function addMessage(msg: Message) {
	chatState.messages = [...chatState.messages, msg];
	messages.update(msgs => [...msgs, msg]);
}

export function setConnectionError(error: string | null) {
	chatState.connectionError = error;
}

export function setGifResults(results: any[]) {
	chatState.gifResults = results;
}

export function toggleGifs() {
	chatState.showGifs = !chatState.showGifs;
	if (!chatState.showGifs) chatState.gifResults = [];
}

export function setGifLoading(loading: boolean) {
	chatState.gifLoading = loading;
}

export async function loadMessages(conversationId: string) {
	try {
		const response = await fetch(`/api/conversations/${conversationId}/messages`, { credentials: 'include' });
		if (!response.ok) throw new Error('Impossible de charger les messages');
		const data = await response.json();
		setMessages(data.messages || []);
		setConnectionError(null);
	} catch (err) {
		setConnectionError('Erreur de chargement des messages');
		console.error('Erreur chargement messages:', err);
	}
}

export async function sendMessage(content: string, conversationId: string, recipientPublicKeys: Uint8Array[], senderPrivateKey: Uint8Array) {
	try {
		const encrypted = await encryptForRecipients(content, recipientPublicKeys, senderPrivateKey);
		const messageData = {
			conversation_id: conversationId,
			content: Array.from(encrypted.encryptedContent),
			encrypted_keys: Object.fromEntries(Object.entries(encrypted.encryptedKeys).map(([userId, key]) => [userId, Array.from(key)])),
			nonce: Array.from(encrypted.nonce),
			media_type: null,
			media_url: null
		};
		const response = await fetch('/api/messages', { method: 'POST', headers: { 'Content-Type': 'application/json' }, credentials: 'include', body: JSON.stringify(messageData) });
		if (!response.ok) throw new Error("Erreur lors de l'envoi du message");
		await loadMessages(conversationId);
		setConnectionError(null);
	} catch (err) {
		setConnectionError("Erreur lors de l'envoi du message");
		console.error('Erreur envoi message:', err);
	}
}

export async function sendGif(gifUrl: string, conversationId: string, recipientPublicKeys: Uint8Array[], senderPrivateKey: Uint8Array) {
	try {
		const encrypted = await encryptForRecipients(`[GIF]${gifUrl}`, recipientPublicKeys, senderPrivateKey);
		const messageData = {
			conversation_id: conversationId,
			content: Array.from(encrypted.encryptedContent),
			encrypted_keys: Object.fromEntries(Object.entries(encrypted.encryptedKeys).map(([userId, key]) => [userId, Array.from(key)])),
			nonce: Array.from(encrypted.nonce),
			media_type: 'gif',
			media_url: gifUrl
		};
		const response = await fetch('/api/messages', { method: 'POST', headers: { 'Content-Type': 'application/json' }, credentials: 'include', body: JSON.stringify(messageData) });
		if (!response.ok) throw new Error("Erreur lors de l'envoi du GIF");
		await loadMessages(conversationId);
		setConnectionError(null);
	} catch (err) {
		setConnectionError("Erreur lors de l'envoi du GIF");
		console.error('Erreur envoi GIF:', err);
	}
}

export async function searchGifs(query: string) {
	try {
		setGifLoading(true);
		setGifResults([]);
		const response = await fetch(`https://tenor.googleapis.com/v2/search?q=${encodeURIComponent(query)}&key=LIVDSRZULELA&client_key=nook&limit=12`);
		if (!response.ok) throw new Error('Erreur lors de la recherche de GIFs');
		const data = await response.json();
		setGifResults(data.results || []);
		setGifLoading(false);
	} catch (err) {
		setGifLoading(false);
		console.error('Erreur recherche GIFs:', err);
	}
}

export async function addReaction(messageId: string, emoji: string) {
	try {
		const response = await fetch('/api/messages/reaction', { method: 'POST', headers: { 'Content-Type': 'application/json' }, credentials: 'include', body: JSON.stringify({ message_id: messageId, emoji }) });
		if (!response.ok) throw new Error("Erreur lors de l'ajout de la réaction");
		setMessages(get(messages).map(msg => {
			if (msg.id === messageId) {
				const reactions = { ...msg.reactions } || {};
				reactions[emoji] = (reactions[emoji] || 0) + 1;
				return { ...msg, reactions };
			}
			return msg;
		}));
	} catch (err) {
		console.error('Erreur ajout réaction:', err);
	}
}

export async function decryptMessageContent(message: Message, privateKey: Uint8Array, senderPublicKey: Uint8Array): Promise<string> {
	try {
		if (message.media_type === 'gif') return `[GIF]${message.media_url}`;
		const encryptedContent = new Uint8Array(message.content);
		const userId = get(authStore).user?.id || '';
		const encryptedKeyData = new Uint8Array(message.encrypted_keys[userId] || []);
		const nonce = new Uint8Array(message.nonce);
		return await decryptMessage(encryptedContent, encryptedKeyData, senderPublicKey, privateKey, nonce);
	} catch (err) {
		console.error('Erreur déchiffrement message:', err);
		return '[Message illisible]';
	}
}

export function formatTimestamp(timestamp: string): string {
	const date = new Date(timestamp);
	const now = new Date();
	if (date.toDateString() === now.toDateString()) return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
	if (date.getFullYear() === now.getFullYear()) return date.toLocaleDateString([], { month: 'short', day: 'numeric' });
	return date.toLocaleDateString([], { year: 'numeric', month: 'short', day: 'numeric' });
}

export async function initUserKeys() {
	const user = get(authStore).user;
	if (!user) return null;
	const storedKeys = await getStoredKeys(user.id);
	if (!storedKeys) return null;
	try {
		const password = user.password || (browser && prompt('Entrez votre mot de passe pour déchiffrer vos messages:'));
		if (!password) return null;
		const privateKey = await decryptPrivateKey(storedKeys.encryptedPrivateKey, password);
		return { privateKey, publicKey: storedKeys.publicKey };
	} catch (err) {
		setConnectionError('Erreur de déchiffrement des clés - vérifiez votre mot de passe');
		console.error('Erreur déchiffrement clés:', err);
		return null;
	}
}
