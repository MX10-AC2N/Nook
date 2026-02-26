// frontend/src/lib/chessStore.svelte.ts
// Store Svelte 5 (runes) pour le jeu d'échecs multi-joueurs
// Gestion : état de la partie, coups, WebSocket temps réel

import { browser } from '$app/environment';
import { authStore } from './authStore.svelte.js';

// ════════════════════════════════════════════════════════════════
// TYPES
// ════════════════════════════════════════════════════════════════

export type PieceType = 'king' | 'queen' | 'rook' | 'bishop' | 'knight' | 'pawn';
export type PieceColor = 'white' | 'black' | 'red' | 'green';
export type GameStatus = 'waiting' | 'playing' | 'finished' | 'abandoned';

export interface ChessPiece {
  id: string;
  piece_type: PieceType;
  color: PieceColor;
  row: number;
  col: number;
  alive: boolean;
  moved: boolean;
}

export interface ChessMove {
  player_slot: number;
  piece_id: string;
  from_row: number;
  from_col: number;
  to_row: number;
  to_col: number;
  captured_piece_id: string | null;
  timestamp: number;
}

export interface ChessGame {
  id: string;
  created_by: string;
  player_count: number;
  player1_id: string | null;
  player2_id: string | null;
  player3_id: string | null;
  player4_id: string | null;
  player1_color: PieceColor;
  player2_color: PieceColor;
  player3_color: PieceColor;
  player4_color: PieceColor;
  current_turn: number;
  status: GameStatus;
  board_state: ChessPiece[];
  move_history: ChessMove[];
  eliminated: number[];
  winner_id: string | null;
  created_at: number;
  updated_at: number;
}

export interface GameListItem {
  id: string;
  created_by: string;
  creator_name: string;
  player_count: number;
  status: GameStatus;
  current_turn: number;
  updated_at: number;
}

export type ValidMove = { row: number; col: number; capture: boolean };

// ════════════════════════════════════════════════════════════════
// LOGIQUE DE VALIDATION CÔTÉ CLIENT
// ════════════════════════════════════════════════════════════════

// Zones hors-jeu sur le plateau 14×14 (coins 4×4 retirés)
function isOutOfBounds(row: number, col: number, playerCount: number): boolean {
  if (playerCount === 2) return row < 0 || row > 7 || col < 0 || col > 7;
  const size = 14;
  if (row < 0 || row >= size || col < 0 || col >= size) return true;
  // Coins retirés
  if (row < 4 && col < 4) return true;
  if (row < 4 && col > 9) return true;
  if (row > 9 && col < 4) return true;
  if (row > 9 && col > 9) return true;
  return false;
}

function pathClear(
  fromRow: number, fromCol: number,
  toRow: number, toCol: number,
  pieces: ChessPiece[]
): boolean {
  const dr = Math.sign(toRow - fromRow);
  const dc = Math.sign(toCol - fromCol);
  let r = fromRow + dr;
  let c = fromCol + dc;
  while (r !== toRow || c !== toCol) {
    if (pieces.some(p => p.row === r && p.col === c && p.alive)) return false;
    r += dr;
    c += dc;
  }
  return true;
}

/// Calcule les cases accessibles par une pièce pour le highlighting du plateau
export function getValidMoves(
  piece: ChessPiece,
  pieces: ChessPiece[],
  playerCount: number
): ValidMove[] {
  const moves: ValidMove[] = [];
  const boardSize = playerCount === 2 ? 8 : 14;

  const tryAdd = (row: number, col: number) => {
    if (isOutOfBounds(row, col, playerCount)) return;
    const target = pieces.find(p => p.row === row && p.col === col && p.alive);
    if (target) {
      if (target.color !== piece.color) moves.push({ row, col, capture: true });
      // propre pièce = bloquant, on ne continue pas
    } else {
      moves.push({ row, col, capture: false });
    }
  };

  const trySlide = (dr: number, dc: number) => {
    let r = piece.row + dr;
    let c = piece.col + dc;
    while (!isOutOfBounds(r, c, playerCount)) {
      const target = pieces.find(p => p.row === r && p.col === c && p.alive);
      if (target) {
        if (target.color !== piece.color) moves.push({ row: r, col: c, capture: true });
        break; // bloqué
      }
      moves.push({ row: r, col: c, capture: false });
      r += dr;
      c += dc;
    }
  };

  switch (piece.piece_type) {
    case 'king':
      for (let dr = -1; dr <= 1; dr++)
        for (let dc = -1; dc <= 1; dc++)
          if (dr !== 0 || dc !== 0) tryAdd(piece.row + dr, piece.col + dc);
      break;

    case 'queen':
      for (const [dr, dc] of [[-1,0],[1,0],[0,-1],[0,1],[-1,-1],[-1,1],[1,-1],[1,1]])
        trySlide(dr, dc);
      break;

    case 'rook':
      for (const [dr, dc] of [[-1,0],[1,0],[0,-1],[0,1]])
        trySlide(dr, dc);
      break;

    case 'bishop':
      for (const [dr, dc] of [[-1,-1],[-1,1],[1,-1],[1,1]])
        trySlide(dr, dc);
      break;

    case 'knight':
      for (const [dr, dc] of [[-2,-1],[-2,1],[-1,-2],[-1,2],[1,-2],[1,2],[2,-1],[2,1]])
        tryAdd(piece.row + dr, piece.col + dc);
      break;

    case 'pawn': {
      // Direction selon couleur
      const isLateral = piece.color === 'red' || piece.color === 'green';
      const forward = piece.color === 'white' || piece.color === 'green' ? -1 : 1;

      const advRow = isLateral ? 0 : forward;
      const advCol = isLateral ? forward : 0;
      const sideAx = isLateral ? 'row' : 'col';

      // Avance simple
      const r1 = piece.row + advRow;
      const c1 = piece.col + advCol;
      if (!isOutOfBounds(r1, c1, playerCount) && !pieces.find(p => p.row === r1 && p.col === c1 && p.alive)) {
        moves.push({ row: r1, col: c1, capture: false });
        // Double avance si premier coup
        if (!piece.moved) {
          const r2 = piece.row + advRow * 2;
          const c2 = piece.col + advCol * 2;
          if (!isOutOfBounds(r2, c2, playerCount) && !pieces.find(p => p.row === r2 && p.col === c2 && p.alive)) {
            moves.push({ row: r2, col: c2, capture: false });
          }
        }
      }
      // Captures diagonales
      for (const side of [-1, 1]) {
        const cr = piece.row + advRow + (isLateral ? side : 0);
        const cc = piece.col + advCol + (isLateral ? 0 : side);
        const target = pieces.find(p => p.row === cr && p.col === cc && p.alive);
        if (target && target.color !== piece.color) {
          moves.push({ row: cr, col: cc, capture: true });
        }
      }
      break;
    }
  }

  return moves;
}

// ════════════════════════════════════════════════════════════════
// STORE
// ════════════════════════════════════════════════════════════════

class ChessStore {
  // ── État réactif ──────────────────────────────────────────────
  gameList     = $state<GameListItem[]>([]);
  currentGame  = $state<ChessGame | null>(null);
  mySlot       = $state<number | null>(null);   // slot du joueur courant (1-4)
  myColor      = $state<PieceColor | null>(null);

  selectedPiece = $state<ChessPiece | null>(null);
  validMoves    = $state<ValidMove[]>([]);

  loading      = $state(false);
  error        = $state<string | null>(null);
  lastMoveHighlight = $state<{ from: [number,number]; to: [number,number] } | null>(null);

  // ── Dérivés ───────────────────────────────────────────────────
  isMyTurn = $derived(
    this.currentGame !== null &&
    this.mySlot !== null &&
    this.currentGame.current_turn === this.mySlot &&
    this.currentGame.status === 'playing'
  );

  isPlaying = $derived(this.currentGame?.status === 'playing');
  isWaiting = $derived(this.currentGame?.status === 'waiting');
  isFinished = $derived(this.currentGame?.status === 'finished');

  boardSize = $derived(
    this.currentGame?.player_count === 2 ? 8 : 14
  );

  // Pièces vivantes par couleur pour affichage des prises
  capturedBy = $derived(() => {
    if (!this.currentGame) return {};
    const captured: Record<PieceColor, ChessPiece[]> = { white: [], black: [], red: [], green: [] };
    for (const p of this.currentGame.board_state) {
      if (!p.alive) captured[p.color].push(p);
    }
    return captured;
  });

  // ── WebSocket ─────────────────────────────────────────────────
  private ws: WebSocket | null = null;
  private wsGameId: string | null = null;

  // ════════════════════════════════════════════════════════════════
  // API
  // ════════════════════════════════════════════════════════════════

  async loadGameList(): Promise<void> {
    this.loading = true;
    this.error = null;
    try {
      const res = await fetch('/api/chess/list', { credentials: 'include' });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data = await res.json();
      this.gameList = data.games ?? [];
    } catch (e: any) {
      this.error = e?.message ?? 'Impossible de charger les parties';
    } finally {
      this.loading = false;
    }
  }

  async createGame(playerCount: number, name?: string): Promise<string | null> {
    this.loading = true;
    this.error = null;
    try {
      const res = await fetch('/api/chess/create', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ player_count: playerCount, name }),
      });
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

  // Charge la partie ET (re)connecte le WebSocket — appel initial uniquement
  async loadGame(gameId: string): Promise<void> {
    this.loading = true;
    this.error = null;
    try {
      const res = await fetch(`/api/chess/${gameId}`, { credentials: 'include' });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data = await res.json();
      if (!data.success) throw new Error(data.message);
      this.currentGame = data.game;
      this.determineMySlot();
      this.connectWebSocket(gameId); // connecte le WS (guard interne évite les doublons)
    } catch (e: any) {
      this.error = e?.message ?? 'Impossible de charger la partie';
    } finally {
      this.loading = false;
    }
  }

  // Rafraîchit uniquement le board depuis le serveur — appelé depuis le WS onmessage
  // Ne rappelle PAS connectWebSocket pour éviter toute boucle infinie
  private async refreshBoard(gameId: string): Promise<void> {
    try {
      const res = await fetch(`/api/chess/${gameId}`, { credentials: 'include' });
      if (!res.ok) return;
      const data = await res.json();
      if (!data.success) return;
      this.currentGame = data.game;
      this.determineMySlot();
      // Annuler la sélection en cours (le coup adverse a peut-être pris notre pièce)
      this.selectedPiece = null;
      this.validMoves = [];
    } catch {
      // Silencieux — le refresh échoue gracieusement
    }
  }

  async joinGame(gameId: string): Promise<boolean> {
    this.loading = true;
    this.error = null;
    try {
      const res = await fetch(`/api/chess/${gameId}/join`, {
        method: 'POST',
        credentials: 'include',
      });
      const data = await res.json();
      if (!data.success) throw new Error(data.message);
      await this.loadGame(gameId);
      return true;
    } catch (e: any) {
      this.error = e?.message ?? 'Impossible de rejoindre';
      return false;
    } finally {
      this.loading = false;
    }
  }

  // ── Sélection d'une pièce + calcul des coups valides ──────────
  selectPiece(piece: ChessPiece | null): void {
    if (!piece || !this.currentGame) {
      this.selectedPiece = null;
      this.validMoves = [];
      return;
    }
    // On ne peut sélectionner que ses propres pièces pendant son tour
    if (!this.isMyTurn || piece.color !== this.myColor) {
      this.selectedPiece = null;
      this.validMoves = [];
      return;
    }
    this.selectedPiece = piece;
    this.validMoves = getValidMoves(piece, this.currentGame.board_state, this.currentGame.player_count);
  }

  // ── Jouer un coup ─────────────────────────────────────────────
  async playMove(toRow: number, toCol: number, promotion?: string): Promise<boolean> {
    if (!this.currentGame || !this.selectedPiece || !this.isMyTurn) return false;

    const gameId = this.currentGame.id;
    this.error = null;

    try {
      const res = await fetch(`/api/chess/${gameId}/move`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({
          piece_id: this.selectedPiece.id,
          to_row: toRow,
          to_col: toCol,
          promotion,
        }),
      });
      const data = await res.json();
      if (!data.success) throw new Error(data.message);

      // Highlight du coup
      this.lastMoveHighlight = {
        from: [this.selectedPiece.row, this.selectedPiece.col],
        to: [toRow, toCol],
      };

      this.selectedPiece = null;
      this.validMoves = [];

      // Recharger l'état de la partie (le WS le fera aussi)
      await this.loadGame(gameId);
      return true;
    } catch (e: any) {
      this.error = e?.message ?? 'Coup invalide';
      return false;
    }
  }

  async resign(): Promise<void> {
    if (!this.currentGame) return;
    try {
      await fetch(`/api/chess/${this.currentGame.id}/resign`, {
        method: 'POST',
        credentials: 'include',
      });
      await this.loadGame(this.currentGame.id);
    } catch (e: any) {
      this.error = e?.message ?? 'Erreur abandon';
    }
  }

  // ── Détermination du slot du joueur courant ───────────────────
  private determineMySlot(): void {
    const g = this.currentGame;
    const uid = authStore.user?.id;
    if (!g || !uid) { this.mySlot = null; this.myColor = null; return; }

    if (g.player1_id === uid) { this.mySlot = 1; this.myColor = g.player1_color; }
    else if (g.player2_id === uid) { this.mySlot = 2; this.myColor = g.player2_color; }
    else if (g.player3_id === uid) { this.mySlot = 3; this.myColor = g.player3_color; }
    else if (g.player4_id === uid) { this.mySlot = 4; this.myColor = g.player4_color; }
    else { this.mySlot = null; this.myColor = null; }
  }

  // ── WebSocket : mises à jour temps réel ──────────────────────
  connectWebSocket(gameId: string): void {
    if (!browser) return;
    if (this.wsGameId === gameId && this.ws?.readyState === WebSocket.OPEN) return;

    this.disconnectWebSocket();
    this.wsGameId = gameId;

    const wsUrl = `${window.location.protocol === 'https:' ? 'wss' : 'ws'}://${window.location.host}/ws`;
    this.ws = new WebSocket(wsUrl);

    this.ws.onopen = () => {
      console.log('[ChessStore] WebSocket connecté');
    };

    this.ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data);
        if (msg.type === 'chess_move' && msg.game_id === gameId) {
          // Recharger la partie depuis le serveur pour obtenir le board_state à jour.
          // On évite d'appliquer le coup localement pour rester en sync avec la source de vérité.
          this.refreshBoard(gameId).catch(console.error);
        }
      } catch {
        // message non-JSON (normal pour d'autres types de messages WebSocket)
      }
    };

    this.ws.onclose = () => {
      console.log('[ChessStore] WebSocket déconnecté');
    };

    this.ws.onerror = (e) => {
      console.error('[ChessStore] Erreur WebSocket', e);
    };
  }

  disconnectWebSocket(): void {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
      this.wsGameId = null;
    }
  }

  reset(): void {
    this.disconnectWebSocket();
    this.currentGame = null;
    this.mySlot = null;
    this.myColor = null;
    this.selectedPiece = null;
    this.validMoves = [];
    this.error = null;
    this.lastMoveHighlight = null;
  }
}

// Singleton
export const chessStore = new ChessStore();

// ════════════════════════════════════════════════════════════════
// UTILITAIRES D'AFFICHAGE
// ════════════════════════════════════════════════════════════════

export const PIECE_SYMBOLS: Record<PieceType, string> = {
  king:   '♔',
  queen:  '♕',
  rook:   '♖',
  bishop: '♗',
  knight: '♘',
  pawn:   '♙',
};

// Symboles noirs (remplis) pour les pièces sombres
export const PIECE_SYMBOLS_FILLED: Record<PieceType, string> = {
  king:   '♚',
  queen:  '♛',
  rook:   '♜',
  bishop: '♝',
  knight: '♞',
  pawn:   '♟',
};

export const COLOR_CLASSES: Record<PieceColor, string> = {
  white: 'piece-white',
  black: 'piece-black',
  red:   'piece-red',
  green: 'piece-green',
};

export const PLAYER_LABELS: Record<number, string> = {
  1: 'Blancs',
  2: 'Noirs',
  3: 'Rouges',
  4: 'Verts',
};

export const PLAYER_COLORS_HEX: Record<PieceColor, string> = {
  white: '#f0f0f0',
  black: '#2d2d2d',
  red:   '#c0392b',
  green: '#27ae60',
};
