# Protocole du workspace commun — agents Nook

> v1 — 2026-10-25. Amendé par 3 relecteurs critiques (architecte, coder, github-manager)
> après mesures sur le repo réel. Ce document est la **source de vérité** du protocole ;
> la skill `nook-worker-supervision` en contient une copie d'accompagnement.

## Contexte

Tous les workers Nook partagent **un seul working tree** : `/opt/data/nook-repo`.
Il n'y a **pas** de worktree par worker — le partage est voulu, mais il exige un protocole.

**Problèmes mesurés qui motivent ce protocole :**

- `git add -A` a embarqué **121 fichiers** modifiés appartenant à d'autres workers
  (dont `backend/nook.db`, une base SQLite *trackée*).
- **23 commits dangling** dont des `WIP on develop` : les workers stashent et abandonnent.
- **9 branches mortes** sur origin, ~27 commits qui n'étaient jamais retombés sur `develop`.
- **3 fois** : travail fait, commit local, carte `done`, mais **rien poussé** sur origin.
- Deux branches distinctes partageant le même suffixe de carte → le 2e `update-ref` écrasait le 1er.

## Règles dures

1. **HEAD partagé = `develop`.** Avant toute opération git, vérifier `git branch --show-current`.
   Si ce n'est pas `develop` → `git checkout develop` avant de continuer.
2. **Aucun worker ne fait `git checkout` de branche**, aucun ne commite directement sur `develop`.
3. **Jamais de `git add -A` global** — l'index partagé capture le travail de tout le monde.
   **Pathspecs explicites uniquement.**
4. **Index isolé** : `GIT_INDEX_FILE` + `git read-tree HEAD`.
5. **Le push est le travail de github-manager seul** (carte de publication groupée).
6. **Branches de travail en `w/`** : `w/<carte>-<slug>-<uuid8>`. Pousser une branche `w/*` ne
   déclenche aucune CI ; seule la fusion vers `develop` déclenche la CI.
7. **Suffixe unique par branche** — deux cartes d'une même tâche parente ne partagent pas le suffixe.
8. **Claim file avant le commit**, mis à jour après. Un écrivain par fichier.
9. **1 worker Kanban à la fois** — l'index partagé n'est jamais en contention.

## Snippet de commit

```bash
set -euo pipefail
REPO=/opt/data/nook-repo
CARD=t_83868105
BRANCH="w/${CARD}-fix-e2ee-access-$(git rev-parse --short=8 HEAD)"
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

**Pièges confirmés (ne pas réintroduire) :**

- `git write-tree` échoue sur index non résolu → `TREE` recevrait le message d'erreur.
- `update-ref` sans old-value = écrasement inconditionnel.
- `commit-tree` contourne les hooks et ne vérifie pas l'ancestralité.
- `update-ref refs/heads/develop` est possible même en detached HEAD → se limiter à `refs/heads/w/*`.
- `core.fileMode=true` : un `chmod +x` devient une modification détectée.

**Confirmé** : en detached HEAD, ce plumbing **ne déplace pas HEAD**.

## Publication (github-manager)

`git branch -vv` et `rev-list --count` mentent (patch déjà présent, upstream configuré sur develop).
Utiliser le **patch-id** :

```bash
cd /opt/data/nook-repo && git fetch --prune origin
for b in $(git for-each-ref --format='%(refname:short)' refs/heads/); do
  [ "$b" = "develop" ] && continue
  ahead=$(git cherry -v origin/develop "$b" | grep -c '^+' || true)
  [ "${ahead:-0}" -gt 0 ] && echo "A POUSSER: $b ($ahead patch(s) absent(s))"
done
```

Garde anti-commit-cassé, puis push atomique :

```bash
SHA=$(git rev-parse <branche>)
git cat-file -e "$SHA^{commit}" || exit 1
git fsck --no-dangling "$SHA"
ORIGIN=$(git ls-remote origin "refs/heads/<branche>" | cut -f1)
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
(Mesuré : le token en dur dans `hosts.yml` était invalide — 401 — alors que la variable
d'environnement fonctionnait.)
