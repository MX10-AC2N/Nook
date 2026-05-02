# ♟️ Rôle : Ingénieur Moteur d'Échecs — Nook

> Spécialiste du moteur d'échecs Rust pur intégré à Nook.
> Activer ce rôle pour : corrections moteur, IA, règles échecs, API parties, frontend chess.

---

## 🎯 Périmètre exclusif

```
backend/src/chess_engine/
├── mod.rs          → Point d'entrée, exports publics
├── types.rs        → Piece, Color, Square, Move, GameState
├── board.rs        → Représentation du plateau (bitboards ou array 8x8)
├── movegen.rs      → Génération des coups légaux
├── attacks.rs      → Tables d'attaques précalculées
├── evaluation.rs   → Évaluation statique de position
├── ai_engine.rs    → Minimax + Alpha-Beta pruning
├── game.rs         → Logique de partie (check, mat, pat, 50 coups)
├── zobrist.rs      → Hashing Zobrist (transposition table)
├── san.rs          → Notation Algébrique Standard (parsing + génération)
└── pgn.rs          → Import/export PGN

backend/src/chess.rs → API HTTP des parties (routes Axum)

frontend/src/routes/chess/+page.svelte → UI Svelte
frontend/src/lib/chessStore.svelte.ts  → State partie côté client
```

---

## 🏗️ Architecture du moteur

### Représentation plateau

```rust
// types.rs — types fondamentaux
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Piece { Pawn, Knight, Bishop, Rook, Queen, King }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Color { White, Black }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Square(pub u8);  // 0-63, a1=0, h8=63

#[derive(Clone, Debug)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<Piece>,
    pub is_en_passant: bool,
    pub is_castle: bool,
}

// board.rs — état complet de la position
pub struct Board {
    pub pieces: [[Option<(Piece, Color)>; 8]; 8],
    pub active_color: Color,
    pub castling_rights: CastlingRights,
    pub en_passant_square: Option<Square>,
    pub halfmove_clock: u32,    // règle des 50 coups
    pub fullmove_number: u32,
}
```

### Algorithme IA

```rust
// ai_engine.rs — Minimax + Alpha-Beta
pub fn find_best_move(board: &Board, depth: u8) -> Option<Move> {
    // Alpha-Beta pruning standard
    // depth = 3-5 selon niveau de difficulté
    // Zobrist hashing pour éviter les positions répétées (transposition table)
}

// evaluation.rs — évaluation statique
fn evaluate(board: &Board) -> i32 {
    // Material balance (pièces)
    // Positional bonus (tables PST - Piece Square Tables)
    // Mobilité
    // Structure de pions
    // Sécurité du roi
}
```

---

## 🌐 API HTTP — chess.rs

```
GET  /api/chess/games              → liste les parties actives/terminées
POST /api/chess/games              → créer une nouvelle partie
GET  /api/chess/games/{id}         → état de la partie (FEN ou position complète)
POST /api/chess/games/{id}/move    → jouer un coup { from, to, promotion? }
GET  /api/chess/games/{id}/moves   → historique des coups (PGN ou liste)
POST /api/chess/games/{id}/resign  → abandonner
GET  /api/chess/games/{id}/status  → en_cours, échec, mat, pat, nulle
```

### Schéma DB (migration 002_chess_fide.sql)

```sql
chess_games(
  id,
  white_player_id,
  black_player_id,
  fen,                -- position actuelle en notation FEN
  status,             -- 'active' | 'checkmate' | 'stalemate' | 'draw' | 'resigned'
  winner_id,          -- NULL si nulle ou en cours
  created_at,
  updated_at
)

chess_moves(
  id,
  game_id,
  player_id,
  move_san,           -- notation algébrique standard (ex: "e4", "Nf3", "O-O")
  move_uci,           -- notation UCI (ex: "e2e4", "g1f3")
  fen_after,          -- FEN après le coup
  played_at
)
```

---

## ⚡ Points critiques du moteur

### Génération de coups légaux

```rust
// movegen.rs — règles spéciales à implémenter correctement
// 1. En passant : le pion adverse a bougé de 2 cases au coup précédent
// 2. Roque : roi pas en échec, cases traversées pas attaquées, droits présents
// 3. Promotion : pion atteint la 8ème rangée → choisir pièce
// 4. Clouage : coup légal uniquement si ne met pas son propre roi en échec

// Pattern de vérification
pub fn is_legal(board: &Board, mv: &Move) -> bool {
    let mut board_after = board.clone();
    board_after.apply_move(mv);
    !is_in_check(&board_after, board.active_color)
}
```

### Zobrist Hashing

```rust
// zobrist.rs — clé unique par position
// Utilisé pour :
// 1. Détecter les positions répétées (nulle par répétition × 3)
// 2. Transposition table dans Alpha-Beta (évite recalcul)

pub struct ZobristTable {
    piece_keys: [[[u64; 64]; 6]; 2],  // [color][piece][square]
    black_to_move: u64,
    castling_keys: [u64; 16],
    en_passant_keys: [u64; 8],
}
```

### Notation SAN — Cas difficiles

```rust
// san.rs — parsing et génération de la notation algébrique
// Cas difficiles à gérer :
// - Disambiguation : Rdd1 (deux tours peuvent aller en d1)
// - Check   : e4+
// - Checkmate : Qh7#
// - Promotion : e8=Q
// - En passant : exd6 e.p. ou juste exd6
// - Roque : O-O (petit) | O-O-O (grand)
```

---

## 🔄 Temps réel — TODO critique

```
État actuel :
- POST /api/chess/games/{id}/move → enregistre le coup en DB ✅
- Mais l'adversaire ne le voit PAS sans rafraîchissement ❌

Solution à implémenter :
Option A (simple) : Polling côté client toutes les 2s
  - chessStore.svelte.ts → setInterval(fetchGameState, 2000)
  - Simple mais ~2s de latence

Option B (recommandé) : WebSocket dédié chess
  - Nouveau endpoint WS : GET /api/chess/games/{id}/ws
  - Backend push le coup joué à l'adversaire
  - Frontend reçoit et met à jour chessStore
  - Utiliser le registry WS déjà en place dans webrtc.rs

Option C : Server-Sent Events (SSE)
  - Plus simple que WS pour du unidirectionnel
  - GET /api/chess/games/{id}/events → text/event-stream
```

---

## 🐛 Bugs chess connus

| Bug | Symptôme | Priorité |
|-----|----------|----------|
| Pas de temps réel | Adversaire doit refresh | 🔴 Haute |
| IA bloque le thread | Alpha-Beta sans timeout | 🟡 Moyenne |
| FEN stocké, pas recalculé | Désync possible | 🟡 Moyenne |
| Nulle par répétition | Non implémentée ? | 🟢 Basse |

---

## 🎮 Frontend chess — chessStore.svelte.ts

```typescript
interface ChessState {
  game: ChessGame | null;
  moves: ChessMove[];
  selectedSquare: string | null;
  legalMoves: string[];     // cases accessibles depuis selectedSquare
  isMyTurn: boolean;
  status: 'active' | 'checkmate' | 'stalemate' | 'draw' | 'resigned';
}

export const chessStore = $state<ChessState>({
  game: null,
  moves: [],
  selectedSquare: null,
  legalMoves: [],
  isMyTurn: false,
  status: 'active'
});

// Actions
export async function selectSquare(square: string): Promise<void> { ... }
export async function playMove(from: string, to: string): Promise<void> { ... }
export async function fetchGame(gameId: string): Promise<void> { ... }
```

---

## 🤝 Flux inter-agents

```
← 🦀 RUST / 🎨 SVELTE : infra WS si temps réel retenu
→ 🦀 RUST              : API HTTP chess (endpoints, structs, schéma DB)
→ 🎨 SVELTE            : types TS board/moves, data-testid UI chess, format FEN
→ 🧪 E2E               : scénarios (créer partie, jouer coup, fin de partie)
```

---

## 📚 Apprentissages

> *Section mise à jour à chaque session.*

### [APP-CHESS-01] Chess page — strict mode violation h1 — Session 17

La page chess avait une violation de strict mode sur un élément `h1` (doublon dans le DOM).
Fix : vérifier qu'un seul `h1` existe par page en mode strict Playwright.
Status : Résolu. Pattern : une seule balise `h1` par page.

### [APP-CHESS-02] Temps réel non implémenté — État actuel

L'adversaire ne voit pas les coups sans refresh manuel.
Trois options identifiées (voir section TODO temps réel).
→ Décision d'architecture requise (📐 ARCHITECT) avant implémentation.
Priorité : DT-02 (dette technique haute).

### [APP-CHESS-03] IA bloquante sans timeout

`find_best_move()` avec depth élevée peut bloquer le thread Tokio.
→ Utiliser `tokio::task::spawn_blocking()` pour les calculs IA.
→ Ajouter un timeout (ex: 5s max par coup IA).
Status : À implémenter.
