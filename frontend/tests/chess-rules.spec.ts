import {
  test,
  expect,
  type APIRequestContext,
  type Browser,
  type Page,
} from "@playwright/test";
import { BASE, E2E_USER, E2E_PASS } from "./helpers";

// Deux utilisateurs E2E existants (voir backend E2E_SETUP) :
//   e2e_ci  / E2eTest123!  → créé par helpers.ts
//   hermes-bot / Hermes2026! → utilisé par comprehensive-chess.spec.ts
const BLACK_USER = "hermes-bot";
const BLACK_PASS = "Hermes2026!";

// ─── Helpers API (sans navigation UI) ──────────────────────────────
async function loginApi(
  api: APIRequestContext,
  u: string,
  p: string,
): Promise<void> {
  const r = await api.post(`${BASE}/auth/login`, {
    data: { username: u, password: p },
  });
  if (!r.ok()) throw new Error(`login(${u}) failed: HTTP ${r.status()}`);
}

async function createGame(
  api: APIRequestContext,
  opponent: "human" | "easy",
  color = "white",
) {
  const r = await api.post(`${BASE}/chess/create`, {
    data: { color, opponent },
  });
  expect([200, 201]).toContain(r.status());
  const body = await r.json();
  return body.game_id as string;
}

async function joinGame(api: APIRequestContext, gameId: string) {
  const r = await api.post(`${BASE}/chess/${gameId}/join`);
  expect([200]).toContain(r.status());
}

async function mv(
  api: APIRequestContext,
  gameId: string,
  from: string,
  to: string,
  promotion?: string,
) {
  const data: Record<string, string> = { from, to };
  if (promotion) data.promotion = promotion;
  return api.post(`${BASE}/chess/${gameId}/move`, { data });
}

async function ai(api: APIRequestContext, gameId: string) {
  const r = await api.post(`${BASE}/chess/${gameId}/ai-move`, { data: {} });
  expect(r.status()).toBe(200);
}

async function getBoard(api: APIRequestContext, gameId: string) {
  const r = await api.get(`${BASE}/chess/${gameId}`);
  expect(r.status()).toBe(200);
  const g = (await r.json()).game;
  return {
    board: g.engine.board as string[][],
    fen: g.fen as string,
    status: g.status as string,
    engineStatus: g.engine.status as string,
    sideToMove: g.engine.side_to_move as string,
    winner: g.winner_id as string | null,
  };
}

// row 0 = rang 8 (noir), row 7 = rang 1 (blanc) → board[row][col] ;
// a8 = board[0][0], g1 = board[7][6], d6 = board[2][3], etc.

// ─── Deux joueurs authentifiés (blancs / noirs) ────────────────────
async function twoPlayers({ page, browser }: { page: Page; browser: Browser }) {
  const blackPage = await browser.newPage();
  await loginApi(page.request, E2E_USER, E2E_PASS);
  await loginApi(blackPage.request, BLACK_USER, BLACK_PASS);
  return {
    white: page.request,
    black: blackPage.request,
    blackPage,
  };
}

// ════════════════════════════════════════════════════════════════
// 1. Legal move highlighting — GET /api/chess/{id}/moves?from=
// ════════════════════════════════════════════════════════════════
test.describe("Chess — coups légaux (highlight + endpoint)", () => {
  test("GET /moves renvoie les coups légaux et une case invalide → 400", async ({
    page,
  }) => {
    await loginApi(page.request, E2E_USER, E2E_PASS);
    const id = await createGame(page.request, "easy");

    const r = await page.request.get(`${BASE}/chess/${id}/moves?from=g1`);
    expect(r.status()).toBe(200);
    const moves = (await r.json()) as string[];
    // Au placement, le cavalier de g1 peut aller en f3 et h3.
    expect(moves).toContain("g1f3");
    expect(moves).toContain("g1h3");

    const bad = await page.request.get(`${BASE}/chess/${id}/moves?from=zz`);
    expect(bad.status()).toBe(400);
  });
});

// ════════════════════════════════════════════════════════════════
// 2. Messages d'erreur explicites pour tout coup illégal
// ════════════════════════════════════════════════════════════════
test.describe("Chess — explications de coups illégaux", () => {
  test("Coup illégal → message clair (déplacement impossible / pièce amie / tour)", async ({
    page,
  }) => {
    await loginApi(page.request, E2E_USER, E2E_PASS);
    const id = await createGame(page.request, "easy"); // blancs, c'est nous

    // Pion e2 → e5 : 3 cases, illégal → "ne se déplace pas comme ça"
    const r1 = await mv(page.request, id, "e2", "e5");
    expect(r1.status()).toBe(400);
    expect((await r1.json()).message).toContain("Coup illégal");
    expect((await r1.json()).message).toContain("ne se déplace pas comme ça");

    // e2 → e1 : case occupée par notre propre roi → "propre"
    const r2 = await mv(page.request, id, "e2", "e1");
    expect(r2.status()).toBe(400);
    expect((await r2.json()).message).toContain("propre");

    // e2e4 est valide → maintenant c'est le tour des noirs
    const ok = await mv(page.request, id, "e2", "e4");
    expect(ok.status()).toBe(200);

    // e7e5 demandé par les BLANCS → "trait aux noirs"
    const r3 = await mv(page.request, id, "e7", "e5");
    expect(r3.status()).toBe(400);
    expect((await r3.json()).message).toContain("trait aux noirs");
  });
});

// ════════════════════════════════════════════════════════════════
// 3. Roque
// ════════════════════════════════════════════════════════════════
test.describe("Chess — roque", () => {
  test("Le roque côté blanc (e1g1) est joué et le plateau est mis à jour", async ({
    page,
    browser,
  }) => {
    const { white, black } = await twoPlayers({ page, browser });
    const id = await createGame(white, "human"); // créateur = blancs
    await joinGame(black, id); // hermes-bot rejoint = noirs

    await mv(white, id, "e2", "e4"); // e4 — blanc
    await mv(black, id, "a7", "a6"); // a6 — noir (lointain)
    await mv(white, id, "g1", "f3"); // Nf3 — libère g1
    await mv(black, id, "h7", "h6"); // h6 — noir (lointain)
    await mv(white, id, "f1", "c4"); // Bc4 — libère f1
    await mv(black, id, "h6", "h5"); // h5 — noir

    const r = await mv(white, id, "e1", "g1"); // roque côté
    expect(r.status()).toBe(200);

    const { board } = await getBoard(white, id);
    // g1 = board[7][6] (roi blanc), f1 = board[7][5] (tour déplacée)
    expect(board[7][6]).toBe("wK");
    expect(board[7][5]).toBe("wR");
    // e1 & h1 vides (pièces sont parties)
    expect(board[7][4]).toBe("");
    expect(board[7][7]).toBe("");
  });
});

// ════════════════════════════════════════════════════════════════
// 4. Prise en passant
// ════════════════════════════════════════════════════════════════
test.describe("Chess — prise en passant", () => {
  test("exd6 (prise en passant) est acceptée et appliquée", async ({
    page,
    browser,
  }) => {
    const { white, black } = await twoPlayers({ page, browser });
    const id = await createGame(white, "human");
    await joinGame(black, id);

    await mv(white, id, "e2", "e4"); // e4
    await mv(black, id, "h7", "h6"); // loin
    await mv(white, id, "e4", "e5"); // e5 (pion blanc sur e5)
    await mv(black, id, "d7", "d5"); // d7d5 → cible e.p. = d6
    const r = await mv(white, id, "e5", "d6"); // exd6 e.p.
    expect(r.status()).toBe(200);

    const { board } = await getBoard(white, id);
    expect(board[2][3]).toBe("wP"); // d6 → pion blanc
    expect(board[3][3]).toBe(""); // d5 → ancien pion noir supprimé
  });
});

// ════════════════════════════════════════════════════════════════
// 5. Promotion
// ════════════════════════════════════════════════════════════════
test.describe("Chess — promotion", () => {
  test("a7a8=q promeut le pion en dame", async ({ page, browser }) => {
    const { white, black } = await twoPlayers({ page, browser });
    const id = await createGame(white, "human");
    await joinGame(black, id);

    await mv(white, id, "b2", "b4"); // b4
    await mv(black, id, "a7", "a6"); // a6
    await mv(white, id, "b4", "b5"); // b5
    await mv(black, id, "h7", "h6"); // loin
    await mv(white, id, "b5", "a6"); // bxa6 (capture) — pion blanc sur a6
    await mv(black, id, "h6", "h5");
    await mv(white, id, "a6", "a7"); // a7
    await mv(black, id, "h5", "h4");
    const r = await mv(white, id, "a7", "a8", "q"); // promotion en dame
    expect(r.status()).toBe(200);

    const { board } = await getBoard(white, id);
    expect(board[0][0]).toBe("wQ"); // a8 = board[0][0]

    // sans précision de promotion, la case d8 doit exiger le choix
    // (rejoué sur une nouvelle partie pour le message)
    const id2 = await createGame(white, "human");
    await joinGame(black, id2);
    // mise en place rapide d'un pion sur a7 (même séquence)
    await mv(white, id2, "b2", "b4");
    await mv(black, id2, "a7", "a6");
    await mv(white, id2, "b4", "b5");
    await mv(black, id2, "h7", "h6");
    await mv(white, id2, "b5", "a6");
    await mv(black, id2, "h6", "h5");
    await mv(white, id2, "a6", "a7");
    await mv(black, id2, "h5", "h4");
    const r2 = await mv(white, id2, "a7", "a8"); // sans promotion
    expect(r2.status()).toBe(400);
    expect((await r2.json()).message).toContain("promotion");
  });
});

// ════════════════════════════════════════════════════════════════
// 6. Échec & mat (fools mate) + échec simple + évasion
// ════════════════════════════════════════════════════════════════
test.describe("Chess — échec, échec&mat et évasion", () => {
  test("fools mate (f3 e5 g4 Qh4#) → checkmate", async ({ page, browser }) => {
    const { white, black } = await twoPlayers({ page, browser });
    const id = await createGame(white, "human");
    await joinGame(black, id);

    await mv(white, id, "f2", "f3");
    await mv(black, id, "e7", "e5");
    await mv(white, id, "g2", "g4");
    const r = await mv(black, id, "d8", "h4"); // Qh4#
    expect(r.status()).toBe(200);

    const b = await getBoard(white, id);
    expect(b.engineStatus).toBe("checkmate");
    expect(b.status).toBe("finished");
    expect(b.winner).toBeTruthy(); // le joueur qui a maté (noirs)
  });

  test("échec simple + obligation de parer", async ({ page, browser }) => {
    const { white, black } = await twoPlayers({ page, browser });
    const id = await createGame(white, "human");
    await joinGame(black, id);

    await mv(white, id, "e2", "e4");
    await mv(black, id, "e7", "e5");
    await mv(white, id, "d1", "h5"); // Qh5 menace f7
    await mv(black, id, "b8", "c6");
    await mv(white, id, "h5", "f7"); // Qxf7+ échec

    const b = await getBoard(white, id);
    expect(b.engineStatus).toBe("check"); // échec signalé par le moteur
    expect(b.status).toBe("playing"); // la partie continue

    // Le noir tente un coup qui ne pare pas l'échec
    const r = await mv(black, id, "d8", "e7"); // Qd8-e7 : ne parle pas l'échec
    expect(r.status()).toBe(400);
    expect((await r.json()).message).toContain("vous êtes en échec");
  });
});

// ════════════════════════════════════════════════════════════════
// 7. Clouage (pièce clouée → roi exposé)
// ════════════════════════════════════════════════════════════════
test.describe("Chess — clouage", () => {
  test('un pion cloué ne peut pas bouger → "le roi serait en échec"', async ({
    page,
    browser,
  }) => {
    const { white, black } = await twoPlayers({ page, browser });
    const id = await createGame(white, "human");
    await joinGame(black, id);

    await mv(white, id, "e2", "e4");
    await mv(black, id, "b8", "c6"); // Nc6
    await mv(white, id, "f1", "b5"); // Bb5 cloue le cavalier de c6 (b5-c6-d7-e8)

    // Le cavalier c6 est cloué : le déplacer exposerait le roi en échec
    const r = await mv(black, id, "c6", "a5");
    expect(r.status()).toBe(400);
    expect((await r.json()).message).toContain("le roi serait en échec");
  });
});

// ════════════════════════════════════════════════════════════════
// 8. Pièce adverse — ne pas déplacer une pièce ennemie
// ════════════════════════════════════════════════════════════════
test.describe("Chess — pièce adverse", () => {
  test("Coup sur une pièce ennemie → \"cette pièce ne vous appartient pas\"", async ({
    page,
  }) => {
    await loginApi(page.request, E2E_USER, E2E_PASS);
    const id = await createGame(page.request, "easy"); // blancs, c'est nous — tour blanc

    // e7 est un pion noir → pièce ennemie (c'est notre tour)
    const r = await mv(page.request, id, "e7", "e5");
    expect(r.status()).toBe(400);
    expect((await r.json()).message).toContain(
      "cette pièce ne vous appartient pas",
    );
  });
});

// ════════════════════════════════════════════════════════════════
// 9. Partie terminée — refus de jouer après la fin
// ════════════════════════════════════════════════════════════════
test.describe("Chess — partie terminée", () => {
  test("Après mat, un nouveau coup → \"Partie terminée\"", async ({
    page,
    browser,
  }) => {
    const { white, black } = await twoPlayers({ page, browser });
    const id = await createGame(white, "human");
    await joinGame(black, id);

    // fool's mate : f3 e5 g4 Qh4#
    await mv(white, id, "f2", "f3");
    await mv(black, id, "e7", "e5");
    await mv(white, id, "g2", "g4");
    const mate = await mv(black, id, "d8", "h4"); // Qh4#
    expect(mate.status()).toBe(200);

    const b = await getBoard(white, id);
    expect(b.status).toBe("finished");
    expect(b.engineStatus).toBe("checkmate");

    // Tentative de coup après la fin
    const r = await mv(white, id, "e2", "e3");
    expect(r.status()).toBe(400);
    expect((await r.json()).message).toContain("Partie terminée");
  });
});
