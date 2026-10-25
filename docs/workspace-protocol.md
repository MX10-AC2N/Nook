# Protocole du workspace commun — agents Nook

> v5 — 2026-10-25. Amendé après la **4e** revue `@architect` de PR#48 (**REQUEST_CHANGES**, carte
> `t_35ea07c7`). La revue round 4 ne demandait qu'**une** correction, ligne « `git add -A` embarque le
> travail de tous » : la proposition « toute exécution du snippet ajoute un commit à ce compte » y était
> **fausse** alors qu'elle était présentée sous l'étiquette **live**. Elle est remplacée par le mécanisme
> exact, mesuré sur un clone jetable.
> (Historique : v4 distinguait les valeurs **live** des **snapshot** ⏱️ après que la revue round 3 eut
> exécuté `git prune --expire=now` — les objets que citaient les lignes « mesure » du tableau ci-dessous
> **n'existent plus**. v5 ne change rien à cette architecture, ni à la règle 6 ni à la règle 7.)
>
> Le tableau distingue deux catégories, **marquées en ligne** :
>
> * **live** — la valeur affichée est la sortie **exacte** de la commande affichée dans la même cellule ; un
>   lecteur qui l'exécute obtient le même nombre.
> * **snapshot** ⏱️ — instantané **daté et non reproductible** : la commande qui le produisait est conservée,
>   avec la raison de la non-reproductibilité.
>
> Repère de mesure : `/opt/data/nook-repo`, branche `develop` à `21b0d93a`, le 2026-10-25. Toutes les commandes du
> tableau sont **non destructrices** (aucune n'écrit dans le repo, aucune ne crée d'objet) — le seul bloc qui
> crée un objet est le snippet §« Snippet de commit ».
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
| **`backend/nook.db` embarqué par `git add -A`** — **live** | `timeout 120 git fsck --no-reflogs --dangling 2>&1 \| awk '$2=="commit"{print $3}' \| while read s; do git show --name-only --format='' $s 2>/dev/null \| grep -q '\.db$' && echo ${s:0:8}; done` → **`5d190f74`** | **live** : `5d190f74` contient `backend/nook.db + nook-data/nook.db` (`git show --name-only --format='' 5d190f74 \| grep '\.db$'` → `backend/nook.db + nook-data/nook.db`, **2** fichiers `.db`) — « embarqué puis perdu ». Aucun ref ne le contient (`git for-each-ref --format='%(refname)' \| while read r; do git merge-base --is-ancestor 5d190f74 $r 2>/dev/null && echo $r; done; echo fin` → **`fin`**, aucune ligne) → **inatteignable depuis `develop`** (`git log --all --oneline -- 'backend/nook.db' \| wc -l` → **0**) : un `git add -A` le réembarque et le perd à nouveau |
| **`backend/nook.db` embarqué par `git add -A`** — ⏱️ snapshot | `git show --name-only --format='' ed4d960f \| grep '\.db$'` | ⏱️ **snapshot non reproductible** : `ed4d960f` **n'existe plus** (`git cat-file -t ed4d960f` → `fatal: Not a valid object name`). C'était un commit de **125** fichiers contenant `backend/nook.db` + `nook-data/nook.db`. Preuve **live** équivalente : ligne ci-dessus |
| **`backend/nook.db` ni suivi ni ignoré** — **live** | `( git ls-files \| grep -c '\.db$' ; echo " jamais suivi ; " ; git log --all --oneline -- '*.db' \| wc -l ; echo " jamais commité ; " ; git check-ignore backend/nook.db ; echo "check-ignore rc="$? ) \| tr "\n" " "` | **live** : **0** (jamais suivi), **0** (jamais commité), `check-ignore rc=1` (**non ignoré**) → non suivi *et* non ignoré, le pire cas pour `git add -A` |
| **Commits abandonnés** — **live** | `( echo "reflogs: " ; timeout 90 git fsck --dangling 2>&1 \| awk '$2=="commit"' \| wc -l ; echo " ; --no-reflogs --dangling: " ; timeout 120 git fsck --no-reflogs --dangling 2>&1 \| awk '$2=="commit"' \| wc -l ; echo " ; --unreachable: " ; timeout 90 git fsck --unreachable 2>&1 \| awk '$2=="commit"' \| wc -l ; echo " ; --no-reflogs --unreachable: " ; timeout 120 git fsck --no-reflogs --unreachable 2>&1 \| awk '$2=="commit"' \| wc -l ) \| tr "\n" " "` | **live** : **aucune valeur stable** — ce compteur dépend des **reflogs**, qu'un `git gc` réécrit tout seul : mesuré **0** puis **7** pour `--dangling` dans la journée du 2026-10-25, et **12** puis **11** pour `--no-reflogs --dangling` (voir la note de fraîcheur ci-dessous). **Lire la commande, pas le nombre.** Seule la variante **`--no-reflogs`** est stable sous `gc` ; les **11** objets restants sont listés ci-dessous, et leur nombre est **recomptable** à l'identique |
| **Commits abandonnés** — ⏱️ snapshot | ⏱️ `timeout 90 git fsck --dangling 2>&1 \| awk '$2=="commit"' \| wc -l` ; `… --no-reflogs --dangling …` ; `… --unreachable …` ; `… --no-reflogs --unreachable …` | ⏱️ **22** (reflogs) ; **34** (`--no-reflogs --dangling`) ; **33** (`--unreachable`) ; dont **9** `WIP on develop`. **Non reproductible** — voir la ligne « Objets cités supprimés » ci-dessous. **Live** : ligne ci-dessus |
| **`git add -A` embarque le travail de tous** — ⏱️ snapshot | ⏱️ `timeout 120 git fsck --no-reflogs --dangling 2>&1 \| awk '$2=="commit"{print $3}' \| while read s; do n=$(git show --stat --format='' $s 2>/dev/null \| tail -1 \| grep -oE '[0-9]+ files? changed' \| grep -oE '^[0-9]+'); [ -n "$n" ] && [ "$n" -ge 100 ] && echo "$n $s"; done \| sort -rn` | ⏱️ **5** commits dangling de ≥100 fichiers : **739** (`a721a1cf`), **733** (`73ff44cc`), **126** (`60b5a583`), **126** (`28cca557`), **125** (`ed4d960f`). **Non reproductible** : ces 5 objets ont disparu. **Live** : la même commande renvoie **aujourd'hui 0 ligne** — et un commit de snippet n'entre dans ce compte en aucune façon, pour deux raisons indépendantes. **1.** Il porte **≤2 fichiers** et le seuil de la ligne est **≥100 fichiers** : il n'y entre donc jamais, quoi qu'il arrive. **2.** Tant que sa branche `w/` existe, le commit est **atteignable** via `refs/heads/w/*` → il n'est **pas** compté par `git fsck --no-reflogs --dangling` ; il ne le devient qu'**après** la suppression de la branche (`git update-ref -d`), et même alors il reste sous le seuil. Mécanisme, mesuré sur clone jetable (forme `compteur avant, commit listé avant, compteur après, commit listé après`) : `( timeout 120 git fsck --no-reflogs --dangling 2>&1 \| awk '$2=="commit"' \| wc -l ; timeout 120 git fsck --no-reflogs --dangling 2>&1 \| awk '$2=="commit"{print $3}' \| grep -c '^<SHA-du-commit>$' ; git update-ref -d refs/heads/w/<slug>-<uuid8> ; timeout 120 git fsck --no-reflogs --dangling 2>&1 \| awk '$2=="commit"' \| wc -l ; timeout 120 git fsck --no-reflogs --dangling 2>&1 \| awk '$2=="commit"{print $3}' \| grep -c '^<SHA-du-commit>$' )` → `12 0 13 1` : `0` = non compté, `1` = compté après suppression (et le compteur ne peut croître que d'une unité, jamais d'un bloc) |
| **Objets cités supprimés** (cause des snapshots) | `for s in ed4d960f 60b5a583 28cca557 a721a1cf 73ff44cc; do printf '%s: %s \| ' $s "$(git cat-file -t $s 2>&1 \| head -1)"; done` | **live** : `ed4d960f: fatal: Not a valid object name ed4d960f \| 60b5a583: fatal: Not a valid object name 60b5a583 \| 28cca557: fatal: Not a valid object name 28cca557 \| a721a1cf: fatal: Not a valid object name a721a1cf \| 73ff44cc: fatal: Not a valid object name 73ff44cc \|` — les 5 objets **n'existent plus** dans le repo |
| **Branches mortes** — **live** | `( git ls-remote --heads origin \| wc -l ; echo " branches sur origin ; " ; git for-each-ref refs/remotes/origin \| wc -l ) \| tr "\n" " "` | **live** : **3** branches sur origin ; **3** refs locales de suivi. Décompte **prune-sensible** : un `git fetch --prune` retire les refs obsolètes (ex. `origin/pr-48`) |
| **Travail jamais poussé** — **live** | `comm -23 <(git for-each-ref --format='%(refname:short)' refs/heads/ \| sort) <(git ls-remote --heads origin 2>/dev/null \| sed 's\|.*refs/heads/\|\|' \| sort) \| wc -l` | **live** : **0** branche locale sans équivalent distant au 2026-10-25 |
| **Collision de suffixe** — **live** | `git reflog --date=short \| grep -i checkout \| head -2` | **live** : checkout vers `origin/fix/e2ee-admin-gate-t_83868105` le **2026-10-24** ; et le **2026-10-25** un retour vers `develop` **depuis** `8050ed53` (le reflog dit `moving from 8050ed53 to develop` : `8050ed53` est le point de **départ**, pas la destination) |


Objets restants au 2026-10-25 — sortie de `timeout 120 git fsck --no-reflogs --dangling 2>&1 | awk '$2=="commit"{print $3}' | while read s; do printf '%s %s files | %s\n' ${s:0:8} $(git show --stat --format='' $s 2>/dev/null | tail -1 | grep -oE '[0-9]+ files? changed' | grep -oE '^[0-9]+') "$(git log -1 --format='%s' $s | head -c 45)"; done` :

```
7e16d8a6 2 files | fix(chat): emoji picker always opens below me
b55e5076 1 files | fix(cache-control): replace global override w
dbb1245f 1 files | fix(chat): picker emojis reaction s'ouvre en 
75de8a5f 1 files | fix(chess): preserve player selection across 
1106aff7 1 files | fix(sw): network-first strategy for HTML/JS/C
6107278e 2 files | ci(e2e-targeted): replace cargo build with Do
5d190f74 24 files | fix(chat): picker emoji toujours en dessous +
8050ed53 3 files | feat(e2ee): enforce RFC-016 §5.4 group-key a
2b720ffb 1 files | fix(frontend): unblock build on develop — r
af943317 1 files | fix(chat): bouton Repondre visible pour tous 
5cc0a7c7 5 files | fix(Backend.yml): correct artifact upload pat
```

> ⚠️ **Objets cités supprimés le 2026-10-25** — les valeurs marquées ⏱️ sont des **snapshots** : les 5 commits qu'elles
> citent (`ed4d960f`, `60b5a583`, `28cca557`, `a721a1cf`, `73ff44cc`) **n'existent plus** dans le repo, supprimés
> par un `git prune --expire=now` exécuté lors de la revue `@architect` round 3. Ils étaient absents d'`origin`
> (HTTP 422) → **irrécupérables** : ne pas tenter de les restaurer. La preuve `backend/nook.db` **survit** sur le
> commit encore présent `5d190f74` (ligne « live » ci-dessus).

⚠️ **Fraîcheur du compteur « Commits abandonnés »** — ce compteur est **mtime-sensible**, pour deux
raisons indépendantes :

1. `git gc` part **automatiquement** lors d'un `push`/`fetch`. Il supprime les entrées de reflog plus
   anciennes que `gc.reflogExpireUnreachable` (**30** jours par défaut) et les objets non référencés plus
   anciens que `gc.pruneExpire` (**2 semaines** par défaut). Un commit abandonné a donc une **durée de vie
   finie**.
2. `git gc` **réécrit les reflogs** (le reflog de `HEAD` est passé de **171** à **152** entrées dans la
   journée du 2026-10-25), ce qui change ce que `fsck` considère comme atteignable.

Mesuré le 2026-10-25, dans la même journée :

```
git fsck --dangling                | commit ->  0  puis  7    (reflogs réécrits par gc)
git fsck --no-reflogs --dangling   | commit -> 12  puis 11    (objet 97ab9529 disparu de lui-même)
```

**Lire la commande, pas le nombre.** La seule variante **stable sous `gc`** est `--no-reflogs --dangling`
(elle ignore les reflogs) ; c'est celle à utiliser pour un recomptage reproductible. Toute valeur figée
dans ce document peut avoir bougé à la prochaine lecture.

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

- **Working tree propre ⇒ le snippet est un no-op.** `git add -A -- <pathspec>` ne Capture que des
  **modifications** : si aucun des pathspecs n'a changé, `git write-tree` rend le **même arbre** que
  `develop` et le commit produit est identique à `develop` (mesuré sur clone jetable :
  `git rev-parse <commit>^{tree}` = `e3a74634d3456f21fe65ef3fa727ff709f3bdf5c` = arbre de `develop`,
  `git diff-tree -r --root --name-only <commit>` → **0 ligne**). Il est quand même créé et référencé par
  `refs/heads/w/…`, donc **atteignable** → il ne figure dans aucun compteur `fsck --dangling` tant que la
  branche existe. Ne pas lire l'absence de sortie de `git show --stat` comme un échec du snippet.
- **Le seuil de fichiers du § mesures est un filtre, pas un compteur.** La ligne « `git add -A` embarque le
  travail de tous » ne compte que les commits de **≥100 fichiers** : un commit de snippet (1-2 fichiers) n'y
  entre jamais, qu'il soit ou non compté par ailleurs.

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

⚠️ **Jamais de `git push` nu** — *avertissement de comportement, pas un état mesuré* : `push.default=simple`
combiné à un HEAD dégradé, et `merge=develop`, peuvent faire partir un push sur `develop`. **Aucune de ces deux
configs n'est posée dans `/opt/data/nook-repo`** (`git config --get push.default` → *unset*,
`git config --get merge.develop` → *unset*, mesuré le 2026-10-25) : la garde reste de mise car un worker peut
les poser. Toujours :

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
