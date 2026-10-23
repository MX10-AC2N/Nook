# ADR-019 : Topologie de déploiement — services, budget de ressources, ingress TLS

**Date** : 2026-10-22 · **Status** : Accepté · **Supersede** : ADR-001, ADR-007
**Owner** : architect · **Carte** : kanban `t_107d5ece` · **Gate** : 4 (Docker / CI / Déploiement)

**Gate 4** : `@architect` ✅ (décideur de l'ADR) · `@ci-monitor` / `@deployer` : **sans objet** —
aucune modification de `Dockerfile*`, `docker-compose*` ni workflow ; l'ADR enregistre un état
**déjà en production**, il ne le change pas.

---

## Contexte

Le tableau `.hermes/profiles/architect/SOUL.md:36-52` liste deux décisions de déploiement
(`ADR-001`, `ADR-007`) au statut `Accepted`. L'inventaire de la carte `t_db6f9481` a établi que
**ces deux ADR n'ont jamais été rédigés** (`docs/adr/README.md` §4.1) : la décision est réelle, le
document ne l'est pas. En l'état, les deux formulations sont **fausses dans les faits** :

| ADR | Formulation | Mesure (2026-10-22) |
|---|---|---|
| **001** | « Single Docker container deployment » | `docker-compose.yml` déploie **3 services** |
| **007** | « Nginx Proxy Manager for LAN/WAN TLS » | le proxy LAN est **`nginx-local`** ; NPM n'est référencé par **aucun** artefact déployé |

Pour ADR-001, la contradiction est déjà consignée par l'Architecte lui-même
(`docs/adr/018-agent-memory-storage.md:122`).

**Précision sur ADR-007** — la formulation « aucun fichier ne référence Nginx Proxy Manager » est
**imprécise**. NPM est cité comme *recommandation WAN* dans `README.md:108,188,199`, `AGENTS.md:47`,
`docs/nginx-local.md:57-69` (branche WAN du schéma), `.hermes/roles/architect.md:140` et en
commentaires dans `backend/src/auth.rs:92,214`. Ce qui est faux, ce n'est pas « NPM est mentionné » :
c'est « NPM est **le** proxy de Nook ». Aucun service, aucune image, aucun workflow, aucun
certificat NPM n'existe dans le dépôt.

**Pourquoi trancher maintenant.** Ces deux formulations servent d'**arguments de rejet**
(`architect/SOUL.md:177` : « Add microservice → Rejected. ADR-001 »). Une contrainte fausse figée
dans un tableau n'est plus opposable : n'importe quelle carte peut répondre « ADR-001 est mort,
3 services existent déjà ». La garde-fou de simplicité ne se supprime pas — elle se **reformule
sur une contrainte mesurable**.

## Options considérées

**A. Marquer ADR-001 et ADR-007 « Obsolète », sans remplacement.**
→ **Rejeté.** La garde-fou « est-ce que ça rend l'installation Docker plus complexe ? » reste
nécessaire (`architect/SOUL.md:177`, *Mandatory Behaviors* §6). Retirer la contrainte laisse le
registre sans ancre : la prochaine carte « ajoutons Redis pour le cache » n'aurait plus aucun texte
à opposer, et l'installation Docker redeviendrait un chantier sans garde-fou.

**B. Recréer rétroactivement des ADR-001 et ADR-007.**
→ **Rejeté.** Décision déjà actée (`docs/adr/README.md` §4.3) : le contexte d'origine
(2026-01 / 2026-03) est perdu ; fabriquer une justification est pire qu'un trou.

**C. Reformuler : un ADR-019 qui supersède ADR-001 et ADR-007 et énonce la contrainte réelle.**
→ **Retenu.** L'ADR-019 enregistre une décision **prise maintenant**, fondée sur des **mesures**
(≠ la mémoire), et remplace les deux formulations fausses par une contrainte **vérifiable**.

**D. Deux ADR (019 = conteneurs/budget, 020 = TLS).**
→ **Rejeté.** Les deux corrections décrivent la **même** topologie et reposent sur les **mêmes**
artefacts (`docker-compose.yml`, `nginx-local.conf`, `Dockerfile.nginx`). Les scinder duplique la
preuve pour un gain de lisibilité nul — et consomme un numéro pour rien.

## Décision

**ADR-019 supersède ADR-001 et ADR-007.** La contrainte de déploiement de Nook est désormais :

1. **Un seul hôte, trois services conteneurisés** : `nook` (application), `nginx-local`
   (reverse proxy HTTPS LAN), `turn` (TURN/STUN pour WebRTC), sur le réseau `nook-network`.
   La règle « un seul conteneur » est **supprimée** : elle n'a jamais été vraie et le code la
   réfute.
   ← Code: `docker-compose.yml:2,58,88` (services `nook`, `nginx-local`, `turn`), `docker-compose.yml:115` (réseau) · `docker-compose.ci.yml:2,58,86`

2. **La contrainte de simplicité porte sur le budget de ressources, pas sur le nombre de conteneurs.**
   Le plafond est porté par le service applicatif : `cpus` `${CPUS_LIMIT:-1.5}` et `memory`
   `${MEMORY_LIMIT:-1028M}`, réservations `0.5` CPU / `256M`. Les deux services d'infrastructure
   (`nginx-local`, `turn`) ne sont pas plafonnés.
   ← Code: `docker-compose.yml:39-46`
   → C'est **ce** budget qu'un ajout de service doit justifier : un conteneur de plus est acceptable
   s'il tient dans l'enveloppe de l'hôte ; il est refusé s'il exige de la relever. La garde-fou
   demeure, son critère devient **chiffré** au lieu d'être un décompte.

3. **TLS LAN = `nginx-local`.** Terminaison HTTPS avec certificat auto-signé **auto-généré**
   (validité 10 ans, CN = IP de la machine, SAN = IP + `localhost` + `nook.local`), proxy vers
   `nook:3000`, injection de `X-Forwarded-Proto: https`, redirection HTTP→HTTPS, publication du CA
   sur `/ca`. Port `${NGINX_HTTPS_PORT:-6443}`.
   ← Code: `nginx-local.conf` (`listen 443 ssl`, `ssl_certificate /etc/nginx/ssl/nook.crt`, `proxy_pass http://nook:3000`) · `Dockerfile.nginx` (`nginx:alpine3.21`) · `gen-cert.sh` · `nginx-entrypoint.sh` · `docs/nginx-local.md`

4. **TLS WAN = reverse proxy externe, hors périmètre Nook.** NPM, Caddy ou Traefik, au choix de
   l'utilisateur. Nook ne le déploie pas, ne le configure pas, ne possède aucun artefact à son nom.
   Sa seule obligation est d'**honorer `X-Forwarded-Proto`** pour choisir `SameSite=None; Secure`
   (WAN) plutôt que `SameSite=Lax` (LAN).
   ← Code: `backend/src/auth.rs:184-216`, `backend/src/main.rs:716,725` · recommandation documentée : `README.md:108,188,199`

**Statut des deux ADR remplacées** : ADR-001 → `Superseded` (formulation morte, contrainte
reformulée par le point 2) ; ADR-007 → `Superseded` (nom de composant faux, topologie réelle
documentée par les points 3 et 4).

## Conséquences

**+** La garde-fou de simplicité redevient opposable, sur une base **mesurable** (`MEMORY_LIMIT`,
`CPUS_LIMIT`) au lieu d'un décompte de conteneurs.
**+** Le registre ne contient plus aucune décision fausse : une revue de design peut lier `ADR-019`
au lieu de citer un ADR inexistant.
**+** La topologie réelle est documentée **avec le code pour preuve**, conforme au principe de
`docs/adr/HISTORY.md:6` (« une ligne sans `← Code:` n'est pas une décision, c'est une rumeur »).
**−** Un ADR de plus ; les décisions 001 et 007 n'existent toujours pas comme fichiers — conforme à
`docs/adr/README.md` §4.3 (non-recréation rétroactive).
**−** Les documents qui présentent NPM comme **le** proxy de Nook (LAN compris) restent à aligner :
`AGENTS.md:47`, `.hermes/rules/architecture.md:163`, `.hermes/rules/workflows.md:39`,
`.hermes/rules/frontend-and-business.md:176`, `.hermes/rules/debug-and-glossary.md:209`,
`.hermes/templates/profile-template/SOUL.md:48`. Ce sont de la **doc**, pas des artefacts déployés.
→ action §7.
**→ Agents impactés** : `@architect` (garde-fou reformulée), `@ci-monitor` + `@deployer` (Gate 4 du
présent enregistrement — sans objet, aucune modification), `@docs-writer` (index).

## Plan d'implémentation

**Aucune modification de code** : l'ADR-019 enregistre un état déjà en production.

| # | Action | Profil | Bloquant ? |
|---|---|---|---|
| 1 | Marquer 001 et 007 `Superseded` dans `docs/adr/README.md` §4.5 et ajouter la §4.6 | `@architect` | Non |
| 2 | Ajouter l'ADR-019 à l'index §2, aux numéros réservés §6, au graphe §8 | `@architect` | Non |
| 3 | Aligner les doc qui présentent NPM comme le proxy LAN (`AGENTS.md`, `.hermes/rules/*.md`, template) | `@docs-writer` | Non |
| 4 | Reformuler `architect/SOUL.md:36-52` (tableau ADR) et le pushback trigger `:177` — fichier protégé → carte dédiée | `@architect` | Non |

## Voir aussi

- `docs/adr/README.md` — §4 (inventaire et décision de non-recréation), §4.5 (preuve par le code), §4.6 (cette décision), §6 (numéros réservés)
- `docs/adr/HISTORY.md` — sections 2026-01 et 2026-03, section 2026-10 (ADR-019)
- `docs/adr/018-agent-memory-storage.md:122` — constat d'origine de la contradiction ADR-001
- `docs/nginx-local.md` — proxy LAN réel
- Cartes kanban : `t_db6f9481` (inventaire), `t_107d5ece` (cette décision), `t_a7907ce2` (sync SOUL)
