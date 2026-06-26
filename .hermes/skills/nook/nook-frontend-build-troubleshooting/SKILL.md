---
name: nook-frontend-build-troubleshooting
description: Diagnostiquer et réparer les builds frontend Nook (Svelte 5, Vite, npm) — erreurs package-lock, parsing, runes.
---
# nook-frontend-build-troubleshooting

## Symptômes fréquents
- `npm ci` échoue sur `package-lock.json` mismatch → supprimer `package-lock.json` + `node_modules/`, relancer `npm install`, vérifier le lockfile dans git.
- Erreur de parsing dans `.svelte` → utiliser `svelte-file-repair` ou vérifier les workarounds Svelte 5 (éviter `form onsubmit`, utiliser `button onclick`).
- **Page blanche au chargement (effect_orphan)** → `$effect` au niveau module qui surveille un `$state` d'un autre module (ex: `cryptoStore.ready` dans `chatStore.svelte.ts`). Le compilateur Svelte 5 crée un effect orphelin lors de l'import dynamique dans le root layout. **Fix** : déplacer l'initialisation dans une fonction exportée (`initCryptoListener()`) appelée depuis `onMount` du composant page.
- **Build failed `[MISSING_EXPORT] "X" is not exported`** → Le module de ré-export (`chatStore.ts`) ne ré-exporte pas la nouvelle fonction. Ajouter l'export manquant dans le fichier `.ts` de ré-export.

## Checklist rapide
1. Lire le log CI sur GitHub pour identifier le premier ERROR (pas les avertissements).
2. Si échec `npm ci`, killer le lockfile et regénérer proprement.
3. Si échec Svelte 5 runes, inspecter `.svelte` avec `browser_console` et `codegraph`.
4. Tester en local avec `npm run build` avant de relancer CI.
5. **Page blanche** → vérifier la console navigateur pour `effect_orphan` ; si présent, chercher un `$effect` au niveau module qui lit un `$state` d'un autre module.

## Références
- Skill: `nook-svelte-frontend`
- Skill: `svelte5-script-debugging`
- Skill: `svelte-file-repair`
