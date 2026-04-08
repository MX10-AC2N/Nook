// frontend/src/lib/notificationStore.svelte.ts
// Système de notifications in-app — bulles toast + son + titre page
// Fonctionne sur HTTP/LAN (pas besoin de HTTPS)
//
// Usage :
//   import { notify } from '$lib/notificationStore.svelte';
//   notify('💬 Nouveau message', 'Alice: Coucou !', { href: '/chat', type: 'message' });

export type NotificationType = 'message' | 'chess' | 'poll' | 'calendar' | 'admin' | 'call' | 'info';

export interface AppNotification {
  id:        string;
  title:     string;
  body:      string;
  type:      NotificationType;
  href:      string | null;     // lien au clic
  icon:      string;            // emoji
  timestamp: number;
  read:      boolean;
}

// ── État ──────────────────────────────────────────────────────────

export const notifications = $state<{ items: AppNotification[] }>({ items: [] });

let _idCounter = 0;
const _originalTitle = typeof document !== 'undefined' ? document.title : 'Nook';

// ── Icônes par type ───────────────────────────────────────────────

const ICONS: Record<NotificationType, string> = {
  message:  '💬',
  chess:    '♟',
  poll:     '📊',
  calendar: '📅',
  admin:    '🛡️',
  call:     '📞',
  info:     'ℹ️',
};

// ── Son de notification (AudioContext — HTTP OK) ──────────────────

function _playSound(type: NotificationType): void {
  try {
    const ctx = new AudioContext();
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.connect(gain);
    gain.connect(ctx.destination);

    // Mélodie différente selon le type
    const melodies: Record<NotificationType, [number, number][]> = {
      message:  [[880, 0], [1100, 0.08]],
      chess:    [[523, 0], [659, 0.1], [784, 0.2]],
      poll:     [[660, 0], [880, 0.1]],
      calendar: [[784, 0], [988, 0.1]],
      admin:    [[440, 0], [554, 0.08], [659, 0.16]],
      call:     [[880, 0], [1100, 0.1], [880, 0.2], [1100, 0.3]],
      info:     [[660, 0]],
    };

    const notes = melodies[type] ?? melodies.info;
    osc.type = 'sine';
    for (const [freq, time] of notes) {
      osc.frequency.setValueAtTime(freq, ctx.currentTime + time);
    }
    gain.gain.setValueAtTime(0.25, ctx.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.01, ctx.currentTime + notes.length * 0.1 + 0.1);
    osc.start(ctx.currentTime);
    osc.stop(ctx.currentTime + notes.length * 0.1 + 0.15);
  } catch { /* silent */ }
}

// ── Badge titre ───────────────────────────────────────────────────

function _updateTitle(): void {
  if (typeof document === 'undefined') return;
  const unread = notifications.items.filter(n => !n.read).length;
  document.title = unread > 0 ? `(${unread}) ${_originalTitle}` : _originalTitle;
}

// ── API publique ──────────────────────────────────────────────────

/**
 * Affiche une notification toast + son + badge titre.
 * @param title  Titre court (ex: "💬 Nouveau message")
 * @param body   Corps du message (ex: "Alice: Coucou !")
 * @param opts   Options : type, href (lien au clic), duration (ms, défaut 5000)
 */
export function notify(
  title: string,
  body: string,
  opts: { type?: NotificationType; href?: string | null; duration?: number } = {},
): void {
  const { type = 'info', href = null, duration = 5000 } = opts;

  const notif: AppNotification = {
    id:        `notif-${++_idCounter}-${Date.now()}`,
    title,
    body: body.slice(0, 120),
    type,
    href,
    icon:      ICONS[type],
    timestamp: Date.now(),
    read:      false,
  };

  // Son
  _playSound(type);

  // Ajouter en tête
  notifications.items.unshift(notif);

  // Limiter à 20 notifications max
  if (notifications.items.length > 20) {
    notifications.items = notifications.items.slice(0, 20);
  }

  // Badge titre
  _updateTitle();

  // Auto-dismiss du toast après duration
  if (duration > 0) {
    setTimeout(() => {
      const idx = notifications.items.findIndex(n => n.id === notif.id);
      if (idx !== -1) {
        notifications.items[idx].read = true;
        _updateTitle();
      }
    }, duration);
  }
}

/** Marquer toutes les notifications comme lues */
export function markAllRead(): void {
  for (const n of notifications.items) {
    n.read = true;
  }
  _updateTitle();
}

/** Supprimer une notification */
export function dismissNotification(id: string): void {
  notifications.items = notifications.items.filter(n => n.id !== id);
  _updateTitle();
}

/** Nombre de non-lues */
export function getUnreadCount(): number {
  return notifications.items.filter(n => !n.read).length;
}

// ── Helpers rapides ───────────────────────────────────────────────

export function notifyMessage(sender: string, body: string, href: string = '/chat'): void {
  notify(`${ICONS.message} ${sender}`, body, { type: 'message', href });
}

export function notifyChess(title: string, body: string, gameId?: string): void {
  notify(`${ICONS.chess} ${title}`, body, { type: 'chess', href: gameId ? `/chess/${gameId}` : '/chess' });
}

export function notifyPoll(title: string, body: string): void {
  notify(`${ICONS.poll} ${title}`, body, { type: 'poll', href: '/polls' });
}

export function notifyCalendar(title: string, body: string): void {
  notify(`${ICONS.calendar} ${title}`, body, { type: 'calendar', href: '/calendar' });
}

export function notifyCall(caller: string): void {
  notify(`${ICONS.call} Appel entrant`, `${caller} vous appelle`, { type: 'call', href: '/call', duration: 30000 });
}

export function notifyAdmin(title: string, body: string): void {
  notify(`${ICONS.admin} ${title}`, body, { type: 'admin', href: '/admin' });
}
