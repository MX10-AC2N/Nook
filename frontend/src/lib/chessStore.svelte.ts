import { notifyChess } from '$lib/notificationStore.svelte';
// src/lib/chessStore.svelte.ts
// Store Svelte 5 Runes — moteur d'échecs FIDE (backend Rust)
//
// Contrat API backend :
//   GET  /api/chess/list                 → { games: GameListItem[] }
//   POST /api/chess/create               → { success, game_id }
//     body: { opponent: "human"|"easy"|"medium"|"hard"|"expert"|"godlike", color: "white"|"black" }
//   GET  /api/chess/{id}                 → { success, game: GameState }
//   POST /api/chess/{id}/join            → { success }
//   POST /api/chess/{id}/move            → { success, game: GameState }
//     body: { from: "e2", to: "e4", promotion?: "q"|"r"|"b"|"n" }
//   POST /api/chess/{id}/ai-move         → { success, game: GameState }
//   GET  /api/chess/{id}/moves?from=e2   → { success, moves: ["e4","e3",...] }
//   POST /api/chess/{id}/resign          → { success }
//
// Format plateau : board[row][col] — row 0 = rang 8 (côté noir), row 7 = rang 1 (côté blanc)
//   ""   = case vide
//   "wP" = pion blanc, "bK" = roi noir, etc.
//   Lettres pièces : K Q R B N P

import { browser } from '$app/environment';
import { authStore } from './authStore.svelte.js';

// ════════════════════════════════════════════════════════════════
// TYPES
// ════════════════════════════════════════════════════════════════

export type PieceColor = 'w' | 'b';
export type PieceType  = 'K' | 'Q' | 'R' | 'B' | 'N' | 'P';
export type SideToMove = 'white' | 'black';
export type GameStatus =
  | 'waiting' | 'playing' | 'finished'
  | 'checkmate' | 'stalemate' | 'draw'
  | 'insufficient_material' | 'repetition' | 'fifty_moves';
export type Difficulty = 'easy' | 'medium' | 'hard' | 'expert' | 'godlike';

export interface Cell {
  row:    number;  // 0–7, rang 8–1
  col:    number;  // 0–7, colonne a–h
  piece:  string;  // "" | "wP" | "bK" | …
}

/** État d'une partie retourné par l'API */
export interface GameState {
  id:              string;
  created_by:      string;
  player1_id:      string | null;
  player2_id:      string | null;
  player1_name:    string | null;
  player2_name:    string | null;
  player1_color:   string;  // "white" | "black"
  player2_color:   string;
  status:          string;  // GameStatus élargi
  winner_id:       string | null;
  ai_difficulty:   string | null;
  time_limit_secs: number;
  fen:             string;
  move_history:  MoveRecord[];
  engine: {
    board:        string[][];   // 8×8
    side_to_move: SideToMove;
    status:       string;
    legal_moves:  string[];     // ["e2e4","d7d8=q",…]
    move_count:   number;
  } | null;
  created_at: number;
  updated_at: number;
}

export interface MoveRecord {
  san:   string;
  by:    string;  // "white" | "black" | "ai"
  color: string;
}

export interface GameListItem {
  id:           string;
  status:       string;
  creator_color: string;
  creator_name: string | null;
  updated_at:   number;
}

/** Case sélectionnée sur le plateau */
export interface SelectedSquare {
  row: number;
  col: number;
  algebraic: string;  // "e2"
}

// ════════════════════════════════════════════════════════════════
// HELPERS
// ════════════════════════════════════════════════════════════════

/** Convertit row/col en notation algébrique (0,0) → "a8", (7,4) → "e1" */
export function toAlgebraic(row: number, col: number): string {
  return String.fromCharCode(97 + col) + String(8 - row);
}

/** Convertit notation algébrique en row/col. "e2" → {row:6,col:4} */
export function fromAlgebraic(sq: string): { row: number; col: number } {
  const col = sq.charCodeAt(0) - 97;
  const row = 8 - parseInt(sq[1]);
  return { row, col };
}

/** Décode une pièce "wP" → { color: 'w', type: 'P' } */
export function decodePiece(cell: string): { color: PieceColor; type: PieceType } | null {
  if (!cell || cell.length !== 2) return null;
  return { color: cell[0] as PieceColor, type: cell[1] as PieceType };
}

// Noms complets des pièces
export const PIECE_NAMES: Record<PieceType, string> = {
  K: 'Roi', Q: 'Dame', R: 'Tour', B: 'Fou', N: 'Cavalier', P: 'Pion',
};

// Symboles Unicode (white = contours, black = remplis)
export const PIECE_UNICODE: Record<string, string> = {
  wK: '♔', wQ: '♕', wR: '♖', wB: '♗', wN: '♘', wP: '♙',
  bK: '♚', bQ: '♛', bR: '♜', bB: '♝', bN: '♞', bP: '♟',
};

// Label du statut pour l'affichage
export function statusLabel(status: string): string {
  const labels: Record<string, string> = {
    waiting:               '🟡 En attente',
    playing:               '🟢 En cours',
    finished:              '🏁 Terminée',
    checkmate:             '♚ Échec et mat',
    stalemate:             '🤝 Pat',
    draw:                  '🤝 Nulle',
    insufficient_material: '🤝 Matériel insuffisant',
    // Alias renvoyés par GameStatus::as_str() du moteur Rust
    repetition:            '🤝 Répétition × 3',
    threefold_repetition:  '🤝 Répétition × 3',
    fifty_moves:           '🤝 Règle des 50 coups',
    fifty_move_rule:       '🤝 Règle des 50 coups',
  };
  return labels[status] ?? status;
}

export const DIFFICULTY_LABELS: Record<Difficulty, string> = {
  easy:    '🐣 Facile',
  medium:  '🧩 Moyen',
  hard:    '💪 Difficile',
  expert:  '🎓 Expert',
  godlike: '😈 Divin',
};

// Extraire les cases destination depuis legal_moves pour une case source donnée
export function getLegalTargets(legalMoves: string[], fromAlg: string): string[] {
  return legalMoves
    .filter(m => m.startsWith(fromAlg))
    .map(m => m.slice(2, 4));
}

// Extraire la promotion depuis un coup "e7e8=q" → "q"
export function extractPromotion(move: string): string | undefined {
  const eq = move.indexOf('=');
  return eq !== -1 ? move.slice(eq + 1) : undefined;
}

// ════════════════════════════════════════════════════════════════
// STORE
// ════════════════════════════════════════════════════════════════

class ChessStore {
  // ── État réactif ──────────────────────────────────────────────
  gameList    = $state<GameListItem[]>([]);
  currentGame = $state<GameState | null>(null);
  loading     = $state(false);
  error       = $state<string | null>(null);
  aiThinking  = $state(false);

  /** Case source sélectionnée par le joueur */
  selected     = $state<SelectedSquare | null>(null);
  /** Cases cibles légales pour la sélection courante */
  legalTargets = $state<string[]>([]);  // ["e4","e3",…]
  /** Dernier coup joué (pour highlight) */
  lastMove     = $state<{ from: string; to: string } | null>(null);
  /** Modal promotion en attente */
  pendingPromotion = $state<{ from: string; to: string } | null>(null);

  // ── Minuteur ─────────────────────────────────────────────────
  /** Temps restant en secondes — 0 = pas de limite */
  whiteTime   = $state(0);
  blackTime   = $state(0);
  timerLimit  = $state(0);  // durée initiale (0 = illimité)
  private _timerInterval: ReturnType<typeof setInterval> | null = null;

  // ── Dérivés ───────────────────────────────────────────────────
  myColor = $derived.by((): SideToMove | null => {
    const g = this.currentGame;
    const uid = authStore.user?.id;
    if (!g || !uid) return null;
    if (g.player1_id === uid) return g.player1_color === 'white' ? 'white' : 'black';
    if (g.player2_id === uid) return g.player2_color === 'white' ? 'white' : 'black';
    return null;
  });

  isMyTurn = $derived(
    this.currentGame?.engine?.side_to_move === this.myColor &&
    this.currentGame?.status === 'playing' &&
    this.myColor !== null
  );

  isVsAI = $derived(!!this.currentGame?.ai_difficulty); // true seulement si chaîne non vide

  // Statuts terminaux : DB ("finished") + moteur Rust (checkmate, stalemate, draw, fifty_move_rule…)
  private static readonly OVER_STATUSES = new Set([
    'finished', 'checkmate', 'stalemate', 'draw',
    'insufficient_material', 'repetition', 'threefold_repetition',
    'fifty_moves', 'fifty_move_rule',
  ]);

  isGameOver = $derived(
    this.currentGame !== null &&
    ChessStore.OVER_STATUSES.has(this.currentGame.status)
  );

  // Plateau courant sous forme de tableau 8×8 (ou tableau vide)
  board = $derived.by((): string[][] => {
    return this.currentGame?.engine?.board ?? Array.from({ length: 8 }, () => Array(8).fill(''));
  });

  // Indicateur d'échec : side_to_move est en échec si statut = "checkmate" ou on peut le déduire
  // Pour l'instant on expose juste le statut engine
  engineStatus = $derived(this.currentGame?.engine?.status ?? '');

  // WebSocket
  private ws: WebSocket | null = null;
  private wsGameId: string | null = null;
  private _wsRetries = 0;
  private _wsTimer: ReturnType<typeof setTimeout> | null = null;
  wsConnected = $state(false);
  wsReconnecting = $state(false);

  // ════════════════════════════════════════════════════════════════
  // API
  // ════════════════════════════════════════════════════════════════

  async loadGameList(): Promise<void> {
    this.loading = true;
    this.error = null;
    try {
      const ctrl = new AbortController();
      const timeout = setTimeout(() => ctrl.abort(), 10_000);
      try {
        const res = await fetch('/api/chess/list', { credentials: 'include', signal: ctrl.signal });
        clearTimeout(timeout);
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        const data = await res.json();
        this.gameList = data.games ?? [];
      } catch (e: any) {
        this.error = e?.message ?? 'Impossible de charger les parties';
      } finally {
        this.loading = false;
      }
    } catch {
      this.loading = false;
    }
  }

  async createGame(params: {
    opponent:        string;   // "human" | Difficulty
    color:           'white' | 'black';
    time_limit_secs?: number;
  }): Promise<string | null> {
    this.loading = true;
    this.error = null;
    try {
      const ctrl = new AbortController();
      const timeout = setTimeout(() => ctrl.abort(), 15_000);
      const res = await fetch('/api/chess/create', {
        method:      'POST',
        headers:     { 'Content-Type': 'application/json' },
        credentials: 'include',
        signal:      ctrl.signal,
        body:        JSON.stringify({ opponent: params.opponent, color: params.color, time_limit_secs: params.time_limit_secs ?? 0 }),
      });
      clearTimeout(timeout);
      const data = await res.json();
      if (!data.success) throw new Error(data.message);
      return data.game_id as string;
    } catch (e: any) {
      this.error = e?.message ?? 'Erreur création partie';
      return null;
    } finally {
      this.loading = false;
    }
  }

  async loadGame(gameId: string): Promise<void> {
    this.loading = true;
    this.error = null;
    try {
      const ctrl = new AbortController();
      const timeout = setTimeout(() => ctrl.abort(), 10_000);
      const res = await fetch(`/api/chess/${gameId}`, { credentials: 'include', signal: ctrl.signal });
      clearTimeout(timeout);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data = await res.json();
      if (!data.success) throw new Error(data.message ?? 'Erreur serveur');
      this.currentGame = data.game;
      this.selected    = null;
      this.legalTargets = [];

      // Restaurer le dernier coup pour l'affichage .cell-last après reload
      // Les coups humains ont { from, to } dans move_history ; les coups IA n'ont pas from/to
      const history: Array<{ from?: string; to?: string }> =
        data.game?.move_history ?? [];
      const lastWithCoords = [...history].reverse().find((m) => m.from && m.to);
      this.lastMove = lastWithCoords
        ? { from: lastWithCoords.from!, to: lastWithCoords.to! }
        : null;

      // Init minuteur depuis la config serveur (tous les joueurs voient le même timer)
      const tl = data.game?.time_limit_secs ?? 0;
      if (tl > 0 && this.timerLimit === 0) {
        this.initTimer(tl);
      }
      this.connectWebSocket(gameId);
    } catch (e: any) {
      this.error = e?.message ?? 'Impossible de charger la partie';
    } finally {
      this.loading = false;
    }
  }

  private async refreshGame(gameId: string): Promise<void> {
    try {
      const ctrl = new AbortController();
      const timeout = setTimeout(() => ctrl.abort(), 10_000);
      const res = await fetch(`/api/chess/${gameId}`, { credentials: 'include', signal: ctrl.signal });
      clearTimeout(timeout);
      if (!res.ok) return;
      const data = await res.json();
      if (!data.success) return;
      this.currentGame  = data.game;
      this.selected     = null;
      this.legalTargets = [];
    } catch {
      // Silencieux
    }
  }

  async joinGame(gameId: string): Promise<boolean> {
    this.error = null;
    try {
      const ctrl = new AbortController();
      const timeout = setTimeout(() => ctrl.abort(), 10_000);
      const res = await fetch(`/api/chess/${gameId}/join`, {
        method: 'POST',
        credentials: 'include',
        signal: ctrl.signal,
      });
      clearTimeout(timeout);
      const data = await res.json();
      if (!data.success) throw new Error(data.message);
      await this.loadGame(gameId);
      return true;
    } catch (e: any) {
      this.error = e?.message ?? 'Impossible de rejoindre';
      return false;
    }
  }

  // ── Sélection d'une case et calcul des cibles légales ─────────
  async selectSquare(row: number, col: number): Promise<void> {
    if (!this.isMyTurn || !this.currentGame?.engine) return;

    const alg   = toAlgebraic(row, col);
    const piece  = this.currentGame.engine.board[row]?.[col] ?? '';
    const color  = piece ? piece[0] : '';
    const myColorChar = this.myColor === 'white' ? 'w' : 'b';

    // Clic sur une cible légale → jouer le coup
    if (this.selected && this.legalTargets.includes(alg)) {
      await this.playMove(this.selected.algebraic, alg);
      return;
    }

    // Clic sur sa propre pièce → sélectionner
    if (piece && color === myColorChar) {
      this.selected     = { row, col, algebraic: alg };
      this.legalTargets = getLegalTargets(
        this.currentGame.engine.legal_moves,
        alg
      );
    } else {
      // Clic dans le vide ou pièce adverse sans sélection → désélectionner
      this.selected     = null;
      this.legalTargets = [];
    }
  }

  // ── Jouer un coup (from/to en algébrique) ─────────────────────
  async playMove(from: string, to: string, promotion?: string): Promise<boolean> {
    if (!this.currentGame) return false;
    const gameId = this.currentGame.id;

    // Détecter si promotion nécessaire (pion atteignant la dernière rangée)
    if (!promotion) {
      const fromCoords = fromAlgebraic(from);
      const piece  = this.currentGame.engine?.board[fromCoords.row]?.[fromCoords.col] ?? '';
      const isPawn  = piece.endsWith('P');
      const toCoords = fromAlgebraic(to);
      const isPromoRank = (piece.startsWith('w') && toCoords.row === 0) ||
                          (piece.startsWith('b') && toCoords.row === 7);
      if (isPawn && isPromoRank) {
        this.pendingPromotion = { from, to };
        return false; // Attendre le choix dans le modal
      }
    }

    this.error = null;
    try {
      const ctrl = new AbortController();
      const timeout = setTimeout(() => ctrl.abort(), 10_000);
      const res = await fetch(`/api/chess/${gameId}/move`, {
        method:      'POST',
        headers:     { 'Content-Type': 'application/json' },
        credentials: 'include',
        signal:      ctrl.signal,
        body:        JSON.stringify({ from, to, promotion: promotion ?? null }),
      });
      clearTimeout(timeout);
      const data = await res.json();
      if (!data.success) throw new Error(data.message ?? 'Coup invalide');

      this.lastMove    = { from, to };
      this.currentGame = data.game;
      this.selected    = null;
      this.legalTargets = [];
      this.pendingPromotion = null;

      // Basculer le minuteur après coup humain
      this.switchTimer(data.game?.engine?.side_to_move ?? null);

      // Si partie vs IA et ce n'est plus mon tour → déclencher l'IA
      if (this.isVsAI && !this.isGameOver && !this.isMyTurn) {
        await this.triggerAiMove();
      }

      return true;
    } catch (e: any) {
      this.error = e?.message ?? 'Coup invalide';
      return false;
    }
  }

  async confirmPromotion(piece: string): Promise<void> {
    if (!this.pendingPromotion) return;
    const { from, to } = this.pendingPromotion;
    this.pendingPromotion = null;
    await this.playMove(from, to, piece);
  }

  async cancelPromotion(): Promise<void> {
    this.pendingPromotion = null;
    this.selected     = null;
    this.legalTargets = [];
  }

  // ── IA ────────────────────────────────────────────────────────
  private async triggerAiMove(): Promise<void> {
    if (!this.currentGame || this.isGameOver) return;
    this.aiThinking = true;

    // Démarrer le timer du camp adverse (IA) PENDANT qu'il réfléchit
    const myColor = this.myColor;
    const aiColor = myColor === 'white' ? 'black' : 'white';
    this.switchTimer(aiColor);

    const ctrl = new AbortController();
    const timeout = setTimeout(() => ctrl.abort(), 15_000);
    try {
      const res = await fetch(`/api/chess/${this.currentGame.id}/ai-move`, {
        method:      'POST',
        headers:     { 'Content-Type': 'application/json' },
        credentials: 'include',
        signal:      ctrl.signal,
        body:        JSON.stringify({ difficulty: this.currentGame.ai_difficulty }),
      });
      clearTimeout(timeout);
      const data = await res.json();
      if (data.success && data.game) {
        this.currentGame = data.game;
        // Mettre à jour lastMove depuis le dernier coup IA (from/to maintenant disponibles)
        const history: Array<{ from?: string; to?: string; by?: string }> =
          data.game.move_history ?? [];
        const lastAi = [...history].reverse().find(m => m.by === 'ai' && m.from && m.to);
        if (lastAi?.from && lastAi?.to) {
          this.lastMove = { from: lastAi.from, to: lastAi.to };
        }
        // Rebascule vers le joueur humain après le coup IA
        this.switchTimer(myColor ?? 'white');
      }
    } catch {
      // Silencieux — le joueur peut retenter
    } finally {
      clearTimeout(timeout);
      this.aiThinking = false;
    }
  }

  // ── Minuteur ─────────────────────────────────────────────────
  initTimer(seconds: number): void {
    this.timerLimit = seconds;
    this.whiteTime  = seconds;
    this.blackTime  = seconds;
    if (seconds > 0) {
      // Démarrer sur le bon camp selon l'état actuel du jeu
      const side = this.currentGame?.engine?.side_to_move ?? 'white';
      this.startTimer(side as 'white' | 'black');
    }
  }

  startTimer(side: 'white' | 'black'): void {
    this.stopTimer();
    if (this.timerLimit === 0) return;
    this._timerInterval = setInterval(() => {
      const game = this.currentGame;
      if (!game || game.status !== 'playing') { this.stopTimer(); return; }
      if (side === 'white') {
        this.whiteTime = Math.max(0, this.whiteTime - 1);
        if (this.whiteTime === 0) { this.stopTimer(); this.onTimerExpired('white'); }
      } else {
        this.blackTime = Math.max(0, this.blackTime - 1);
        if (this.blackTime === 0) { this.stopTimer(); this.onTimerExpired('black'); }
      }
    }, 1000);
  }

  switchTimer(sideToMove: string | null): void {
    if (this.timerLimit === 0 || !sideToMove) return;
    this.startTimer(sideToMove as 'white' | 'black');
  }

  stopTimer(): void {
    if (this._timerInterval) { clearInterval(this._timerInterval); this._timerInterval = null; }
  }

  private onTimerExpired(side: 'white' | 'black'): void {
    const game = this.currentGame;
    if (!game || game.status !== 'playing') return;
    tracing: console.warn(`⏰ Temps écoulé pour les ${side === 'white' ? 'Blancs' : 'Noirs'}`);
    // Si c'est notre tour qui expire → on abandonne automatiquement
    const myColor = this.myColor;
    if (myColor === side) {
      this.resign().catch(() => {});
    }
    // Sinon : l'adversaire a perdu son temps → l'admin/arbitre décide
    // Pour le moment on laisse la partie continuer (pas de forfait automatique côté serveur)
  }

  isTimerExpired = $derived(
    this.timerLimit > 0 && (
      (this.currentGame?.engine?.side_to_move === 'white' && this.whiteTime === 0) ||
      (this.currentGame?.engine?.side_to_move === 'black' && this.blackTime === 0)
    )
  );

  // ── Abandon ───────────────────────────────────────────────────
  async resign(): Promise<void> {
    if (!this.currentGame) return;
    const gameId = this.currentGame.id;
    try {
      const ctrl = new AbortController();
      const timeout = setTimeout(() => ctrl.abort(), 10_000);
      const res = await fetch(`/api/chess/${gameId}/resign`, {
        method: 'POST',
        credentials: 'include',
        signal: ctrl.signal,
      });
      clearTimeout(timeout);
      const data = await res.json();
      if (!data.success) throw new Error('Failed to resign');
      // Le backend retourne { success, status, winner_id } — pas de data.game
      // On met à jour currentGame directement pour éviter un fetch supplémentaire
      this.currentGame = {
        ...this.currentGame!,
        status: data.status ?? 'finished',
        winner_id: data.winner_id ?? null,
      };
      this.stopTimer();
    } catch (e: any) {
      this.error = e?.message ?? 'Erreur abandon';
      await this.refreshGame(gameId);
    }
  }

  // ── WebSocket ─────────────────────────────────────────────────
  connectWebSocket(gameId: string): void {
    if (!browser) return;
    if (this.wsGameId === gameId && this.ws?.readyState === WebSocket.OPEN) return;
    this.disconnectWebSocket();
    this.wsGameId = gameId;
    this._wsConnect();
  }

  private _wsConnect(): void {
    if (!browser || !this.wsGameId) return;
    const proto = window.location.protocol === 'https:' ? 'wss' : 'ws';
    const ws = new WebSocket(`${proto}://${window.location.host}/ws`);
    this.ws = ws;

    ws.onopen = () => {
      this.wsConnected = true;
      this.wsReconnecting = false;
      this._wsRetries = 0;
      if (this._wsTimer) { clearTimeout(this._wsTimer); this._wsTimer = null; }
    };

    ws.onmessage = (ev) => {
      try {
        const msg = JSON.parse(ev.data);
        const gameId = this.wsGameId!;
        if (msg.game_id !== gameId) return; // autre partie ou autre type
        if (msg.type === 'chess_move' || msg.type === 'chess_ai_move') {
          if (msg.type === 'chess_ai_move') notifyChess('Tour de l\'IA', 'L\'IA a joué', gameId);
          // Don't clobber user's pending selection during their turn
          if (!this.selected) {
            this.refreshGame(gameId).catch(console.error);
          } else {
            // Just update board state without clearing selection
            this.refreshGame(gameId).then(() => {
              // Restore selection if it was a legal target before refresh
              if (this.selected && this.lastMove) {
                this._restoreLegalTargets(this.wsGameId!);
              }
            }).catch(console.error);
          }
        }
        if (msg.type === 'chess_player_joined') {
          this.refreshGame(gameId).catch(console.error);
          notifyChess('Adversaire rejoint', 'Un joueur a rejoint la partie', gameId);
        }
      } catch { /* non-JSON ok */ }
    };

    ws.onerror = () => {};

    ws.onclose = () => {
      this.wsConnected = false;
      if (this._wsRetries < 12) {
        this.wsReconnecting = true;
        const delay = Math.min(1000 * 2 ** this._wsRetries, 30_000);
        this._wsRetries++;
        this._wsTimer = setTimeout(() => this._wsConnect(), delay);
      }
    };
  }

  disconnectWebSocket(): void {
    if (this._wsTimer) { clearTimeout(this._wsTimer); this._wsTimer = null; }
    if (this.ws) { this.ws.onclose = null; this.ws.close(); this.ws = null; }
    this.wsGameId = null;
    this._wsRetries = 0;
  }

  reset(): void {
    this.disconnectWebSocket();
    this.stopTimer();
    this.currentGame      = null;
    this.selected         = null;
    this.legalTargets     = [];
    this.lastMove         = null;
    this.error            = null;
    this.aiThinking       = false;
    this.pendingPromotion = null;
    this.whiteTime        = 0;
    this.blackTime        = 0;
    this.timerLimit       = 0;
  }
}

export const chessStore = new ChessStore();
