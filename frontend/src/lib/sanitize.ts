// frontend/src/lib/sanitize.ts
// Fix SEC-01 : sanitisation DOMPurify pour {@html} dans le chat
// À importer dans chat/+page.svelte : import { sanitizeHtml } from '$lib/sanitize';

import DOMPurify from 'dompurify';

const ALLOWED_TAGS = ['b', 'i', 'em', 'strong', 'a', 'br', 'img', 'span', 'p', 'div', 'audio', 'video', 'source'];
const ALLOWED_ATTR = ['href', 'src', 'alt', 'class', 'loading', 'target', 'rel', 'title', 'controls', 'preload', 'download', 'type'];

/**
 * Sanitise une chaîne HTML avant injection via {@html}.
 * - Autorise les balises de formatage basique + images (pour GIFs et uploads)
 * - Bloque tout script, event handler, data: URI non image
 * - SSR-safe : retourne la chaîne brute si window n'est pas défini
 */
export function sanitizeHtml(html: string): string {
  if (typeof window === 'undefined') {
    // SSR : pas de DOM disponible — retourner texte brut encodé
    return html.replace(/[<>]/g, (c) => (c === '<' ? '&lt;' : '&gt;'));
  }
  return DOMPurify.sanitize(html, {
    ALLOWED_TAGS,
    ALLOWED_ATTR,
    ADD_ATTR: ['target'],
    FORCE_BODY: false,
    WHOLE_DOCUMENT: false,
  });
}

/**
 * Remplace @username par un span.highlight-mention dans le texte.
 * À appeler AVANT sanitizeHtml.
 */
export function highlightMentions(text: string): string {
  if (!text) return text;
  // Ne pas matcher dans les balises HTML existantes
  return text.replace(/@([\w.-]+)/g, '<span class="mention" data-user="$1">@$1</span>');
}
