# Protocole du workspace commun — agents Nook

> v3 — 2026-10-25. Amendé après la **2e** revue `@architect` de PR#48 (**REQUEST_CHANGES**,
> carte `t_35ea07c7`) : le point restant (NB 5 — le « 33 » n'était pas reproductible par la commande
> citée) est corrigé, et la vérification a été étendue à **toutes** les valeurs chiffrées du tableau
> ci-dessous. Désormais **chaque affirmation sur l'état du repo est suivie de la commande qui la
> reproduit**, et chaque valeur affichée est la sortie **exacte** de la commande affichée
> (mesures du 2026-10-25, exécutées dans `/opt/data/nook-repo`).
>
> **Source de vérité : ce document** (versionné dans le repo, à relire avant de dispatcher ou de
> briefer une carte qui touche au repo).
> La skill `nook-worker-supervision` (profil `orchestrator`) en porte une **copie d'accompagnement**
> : `skills/nook/nook-worker-supervision/references/workspace-protocol.md`. **En cas de divergence,
> ce document prime** — la skill est mise à jour *depuis* ici. Les deux portent les mêmes 9 règles ;
> les contrôles d'exécution propres à la skill (test FF, garde `push.default`) sont repris ci-dessous
> § Publication.
> La copie de la skill portait encore, au 2026-10-25, la **règle 6 v1 (fausse)** et le compteur
> « 22/23 workflows manuels » → alignement suivi par la carte `t_e90110f9` (profil `orchestrator`).

## Contexte

Tous les workers Nook partagent **un seul working tree** : `/opt/data/nook-repo`.
Il n'y a **pas** de worktree par worker — le partage est voulu, mais il exige un protocole.

**Problèmes mesurés qui motivent ce protocole** (mesures du 2026-10-25, repo `/opt/data/nook-repo`) :

| Constat | Preuve (commande) | Valeur mesurée |
|---|---|---|
| `git add -A` embarque le travail de tous | `git fsck --no-reflogs --dangling \| awk '$2=="commit"{print $3}' \| while read s; do n=$(git show --stat --format='' $s \| tail -1 \| grep -oE '[0-9]+ files? changed' \| grep -oE '^[0-9]+'); [ -n "$n" ] && [ "$n" -ge 100 ] && echo "$n $s"; done \| sort -rn` | **5** commits dangling de ≥100 fichiers, dont **3** de **125-126 fichiers** : `ed4d960f` (125), `60b5a583` (126), `28cca557` (126) ; les 2 plus gros font **739** (`a721a1cf`) et **733** (`73ff44cc`) fichiers |
| `backend/nook.db` embarqué par `git add -A` | `git show --name-only --format='' ed4d960f \| grep '\.db$'` | `backend/nook.db`, `nook-data/nook.db` |
| `backend/nook.db` **ni suivi ni ignoré** | `git ls-files \| grep -c '\.db$'` → `0` ; `git log --all --oneline -- '*.db' \| wc -l` → `0` ; `git check-ignore backend/nook.db` → `rc=1` | non suivi **et** non ignoré |
| Commits abandonnés | `git fsck --dangling \| awk '$2=="commit"' \| wc -l` → **22** ; `git fsck --no-reflogs --dangling \| awk '$2=="commit"' \| wc -l` → **34** ; `git fsck --unreachable \| awk '$2=="commit"' \| wc -l` → **33** ; `git fsck --dangling \| awk '$2=="commit"{print $3}' \| while read s; do git log -1 --format='%s' $s; done \| grep -c '^WIP on develop'` → **9** | **22** commits dangling (avec reflogs) ; **34** (`--no-reflogs --dangling`) ; **33** (`--unreachable`) ; dont **9** `WIP on develop` |
| Branches mortes | `git ls-remote --heads origin \| wc -l` ; `git for-each-ref refs/remotes/origin \| wc -l` | **3** branches sur origin ; **3** refs locales de suivi au 2026-10-25 — décompte **prune-sensible** : il valait **4** avant le `git fetch --prune` du 2026-10-25 (dont la ref obsolète `origin/pr-48`, depuis supprimée) |
| Travail jamais poussé | `git for-each-ref --format='%(refname:short)' refs/heads/` comparé à `git ls-remote --heads origin` | **0** branche locale sans équivalent distant au 2026-10-25. Le phénomène est historique (signalé par des workers entre 2026-08 et 2026-10) — non reproductible a posteriori |
| Collision de suffixe | `git reflog --date=short \| grep -i checkout` | checkout vers `origin/fix/e2ee-admin-gate-t_83868105` le **2026-10-24** |

**Lacune identifiée — à corriger hors de ce document** : `backend/nook.db` n'est pas dans
`.gitignore` → non suivi *et* non ignoré, le pire cas pour `git add -A` (il est embarqué puis
perdu). Carte de suivi créée : `t_586756e1` (ajout de `backend/nook.db` et `nook-data/` au
`.gitignore`).

## Règles dures — **cibles**, pas états acquis

Le reflog prouve que les règles 1 et 2 ne sont **pas** encore respectées partout :
`git reflog --date=short | grep -i checkout` → `checkout: moving from develop to
origin/fix/e2ee-admin-gate-t_83868105` (2026-10-24). Elles sont énoncées comme des **objectifs**.

1. **HEAD partagé = `develop`.** Avant toute opération git, `git branch --show-current` doit renvoyer
   `develop` ; sinon `git checkout develop` avant de continuer.
   Contrôle : `git rev-parse --short=8 HEAD` (au 2026-10-25 : `21b0d93a`).
2. **Aucun worker ne fait `git checkout` de branche**, aucun ne commite directement sur `develop`.
3. **Jamais de `git add -A` global** — l'index partagé capture le travail de tout le monde.
   **Pathspecs explicites uniquement.**
4. **Index isolé** : `GIT_INDEX_FILE` + `git read-tree HEAD`.
5. **Le push est le travail de github-manager seul** (carte de publication groupée).
6. **Branches de travail en `w/`** : `w/<carte>-<slug>-<uuid8>`. Pousser une branche `w/*` ne
   déclenche **pas** la CI **sauf** si le commit touche un chemin filtré par l'un des **4 workflows
   sans filtre `branches:`** (audit du 2026-10-25, commande §« Audit CI ») :
   - `services/turn-rs/**` → `turn.yml`, `test-turn.yml` (build/test, **sans écriture**) ;
   - `backend/Cargo.toml` → `update-cargo-lock.yml` (**commit + force-push sur la branche**) ;
   - `frontend/package.json` → `update-frontend-lock.yml` (**commit + force-push sur la branche**).

   Les deux derniers **réécrivent la branche `w/` sous le worker**
   (`git push --force-with-lease origin HEAD:${{ github.ref_name }}`) → ils violent la règle 5.
   **Conséquence pratique** : ne pas commiter `backend/Cargo.toml` ni `frontend/package.json` sur
   une branche `w/` (les laisser à `develop`) ; à défaut, accepter que le workflow réécrive la
   branche — le CAS `update-ref` protège le commit *local*, pas le *remote*.

   Repère chiffré : sur **23** workflows, **8** ont un trigger `push:` (les 4 ci-dessus +
   `Backend.yml`, `bundle-analysis.yml`, `generate-pwa-icons.yml`, `sqlx-prepare.yml`, qui ont un
   filtre `branches:`) ; **22** sont déclenchables manuellement (`workflow_dispatch`).
7. **Suffixe unique par branche** — le `<uuid8>` est **aléatoire** : `openssl rand -hex 4`.
   ⚠️ **Ne pas** utiliser `$(git rev-parse --short=8 HEAD)` : sous la règle 1 (`HEAD = develop`),
   ce suffixe vaut **toujours** le SHA de `develop` (mesuré `21b0d93a`) → deux branches de même
   carte et de même slug auraient un **nom identique**, et le 2e `update-ref` écraserait le 1er.
   Preuve (mesurée) :
   ```bash
   CARD=t_83868105
   for i in 1 2; do echo "w/${CARD}-fix-e2ee-access-$(git rev-parse --short=8 HEAD)"; done
   # → 2 fois : w/t_83868105-fix-e2ee-access-21b0d93a   (COLLISION)
   for i in 1 2; do echo "w/${CARD}-fix-e2ee-access-$(openssl rand -hex 4)"; done
   # → 2 noms distincts (validé : le snippet ci-dessous crée 2 branches différentes)
   ```
8. **Claim file avant le commit**, mis à jour après. Un écrivain par fichier.
9. **1 worker Kanban à la fois** — l'index partagé n'est jamais en contention.

## Snippet de commit

```bash
set -euo pipefail
REPO=/opt/data/nook-repo
CARD=t_83868105
BRANCH="w/${CARD}-fix-e2ee-access-$(openssl rand -hex 4)"   # uuid8 ALÉATOIRE (règle 7)
cd "$REPO"

[ "$(git branch --show-current)" = "develop" ] || { echo "HEAD partage n'est pas develop"; exit 1; }

PATHSPECS=(backend/src/e2ee.rs CHANGELOG.md)

export GIT_INDEX_FILE="$REPO/.git/index.${CARD}.$$"
trap 'rm -f "$GIT_INDEX_FILE"' EXIT
git read-tree HEAD
git add -A -- "${PATHSPECS[@]}"
TREE=$(git write-tree)

PARENT=$(git rev-parse develop)
COMMIT=$(git commit-tree "$TREE" -p "$PARENT" -m "fix(scope): message de la carte")

OLD=$(git rev-parse --verify -q "refs/heads/$BRANCH" || true)
if [ -n "$OLD" ]; then
  git update-ref "refs/heads/$BRANCH" "$COMMIT" "$OLD"
else
  git update-ref "refs/heads/$BRANCH" "$COMMIT"
fi

git merge-base --is-ancestor "$PARENT" "$COMMIT"
[ "$(git rev-parse HEAD)" = "$PARENT" ]
git rev-parse "refs/heads/$BRANCH"
```

**Validé empiriquement** (clone jetable de `/opt/data/nook-repo`, 2026-10-25) : deux exécutions sur
la même carte créent **deux branches distinctes** (une exécution a produit `…-e2f8b78f` puis
`…-d095e811` ; une autre `…-9cbb0435` puis `…-eb6faa14`), `HEAD` reste sur `develop` (`21b0d93a`),
`develop` est inchangé, le parent du commit vaut `develop`.
⚠️ Ces suffixes sont **aléatoires** : ils ne sont pas reproductibles à l'identique — chaque exécution
produit un nom différent, c'est précisément le comportement attendu par la règle 7.

**Pièges confirmés (ne pas réintroduire) :**

- `git write-tree` échoue sur index non résolu → `TREE` recevrait le message d'erreur.
- `update-ref` sans old-value = écrasement inconditionnel.
- `commit-tree` contourne les hooks et ne vérifie pas l'ancestralité.
- `update-ref refs/heads/develop` est possible même en detached HEAD → se limiter à `refs/heads/w/*`.
- `core.fileMode=true` : un `chmod +x` devient une modification détectée.
- `commit-tree` exige une identité git (`user.name`/`user.email`) : mesuré
  `github-manager (Hermes) <github-manager@nook.local>` dans `/opt/data/nook-repo`
  (`git config --local user.name`).

**Confirmé** : en detached HEAD, ce plumbing **ne déplace pas HEAD**.

## Audit CI — quels workflows partent sur `w/*`

```bash
cd /opt/data/nook-repo
for f in .github/workflows/*.yml; do
  blk=$(awk '/^on:/{ino=1;next} ino&&/^[^ \t]/{ino=0} ino&&/^  push:/{ip=1;next} ip&&/^  [a-zA-Z_]+:/{ip=0} ip{print}' "$f")
  [ -z "$blk" ] && continue
  grep -q 'branches:' <<<"$blk" && continue
  echo "SANS filtre branches: $f"
  grep -A3 'paths:' <<<"$blk" | grep -E '^[[:space:]]*-' | sed 's/^[[:space:]]*/  /'
done
```

Sortie mesurée le 2026-10-25 : `test-turn.yml` (`services/turn-rs/**`), `turn.yml`
(`services/turn-rs/**`), `update-cargo-lock.yml` (`backend/Cargo.toml`),
`update-frontend-lock.yml` (`frontend/package.json`).

Compteurs : `ls .github/workflows/*.yml | wc -l` → **23** ; workflows avec `push:` → **8** ;
avec `workflow_dispatch` → **22**.

## Publication (github-manager)

`git branch -vv` et `rev-list --count` mentent (patch déjà présent, upstream configuré sur `develop`).
Utiliser le **patch-id** :

```bash
cd /opt/data/nook-repo && git fetch --prune origin
for b in $(git for-each-ref --format='%(refname:short)' refs/heads/); do
  [ "$b" = "develop" ] && continue
  ahead=$(git cherry -v origin/develop "$b" | grep -c '^+' || true)
  [ "${ahead:-0}" -gt 0 ] && echo "A POUSSER: $b ($ahead patch(s) absent(s))"
done
```

Garde anti-commit-cassé + test de fast-forward, puis push atomique :

```bash
SHA=$(git rev-parse <branche>)
git cat-file -e "$SHA^{commit}" || exit 1
git fsck --no-dangling "$SHA"
ORIGIN=$(git ls-remote origin "refs/heads/<branche>" | cut -f1)
[ -z "$ORIGIN" ] && echo "nouvelle branche"
git merge-base --is-ancestor "$ORIGIN" "$SHA" && echo "FF OK" || echo "NON-FF: rebase obligatoire"
```

⚠️ **Jamais de `git push` nu** : `push.default=simple` + HEAD dégradé + `merge=develop` peuvent
pousser sur `develop`. Toujours :

```bash
git push --force-with-lease=<branche>:<ORIGIN> origin "$SHA":refs/heads/<branche>
git ls-remote origin refs/heads/<branche>   # doit renvoyer $SHA
```

Mise à jour de `develop` (working tree propre requis) :

```bash
git stash push -u -m "workspace-cleanup" && git fetch origin && git merge --ff-only origin/develop
```

## Claim file

Répertoire : `/opt/data/home/.hermes/kanban/boards/nook/workspace-claims/t_<card>.json`

```json
{
  "card": "t_xxxxxxxx",
  "assignee": "coder",
  "branch": "w/t_xxxxxxxx-fix-truc-abc12345",
  "base": "develop",
  "base_sha": "<sha de develop au moment du claim>",
  "head_sha": "<sha du commit local>",
  "expected_parent": "<sha parent — doit valoir base_sha>",
  "files": ["backend/src/e2ee.rs"],
  "status": "in_progress | committed_local | handed_to_publisher | closed",
  "claim_owner": "<worker_pid>",
  "started_at": "...",
  "updated_at": "..."
}
```

Gardes : `git rev-parse <branch>` doit valoir `head_sha` ; le publisher ne clos qu'après
`git ls-remote` confirmant le SHA distant.

## Secrets

Le token **ne doit pas** vivre dans `remote.origin.url` (visible dans `ps`) ni dans
`.git-credentials`. Utiliser `GITHUB_TOKEN` en environnement.
(Mesuré en 2026-08 : le token en dur dans `hosts.yml` était invalide — 401 — alors que la variable
d'environnement fonctionnait.)

⚠️ **Écart constaté le 2026-10-25** : `git remote get-url origin` sur `/opt/data/nook-repo`
contient bien un token (`x-access-token`) → cette règle n'est **pas** appliquée par le repo lui-même.
Correction hors périmètre de ce document (modif de la remote = infra).

## Références

← Code : `.github/workflows/{turn,test-turn,update-cargo-lock,update-frontend-lock}.yml` (déclencheurs `push:`)
← Skill : `nook-worker-supervision` → `references/workspace-protocol.md` (profil `orchestrator`)
← Profil : `@architect` (revue PR#48), `@github-manager` (publication)
