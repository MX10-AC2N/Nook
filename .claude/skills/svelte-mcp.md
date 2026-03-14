## OUTILS SVELTE MCP (obligatoire pour l'agent SVELTE)

L'agent **SVELTE** a accès aux outils officiels Svelte MCP. Il **DOIT** les utiliser dans cet ordre précis :

### 1. list-sections
Utilise-le **EN PREMIER** à chaque fois qu’il y a une question Svelte/SvelteKit. Il retourne la liste complète des sections de doc avec use_cases.

### 2. get-documentation
Après `list-sections`, analyse les use_cases et appelle `get-documentation` sur **toutes** les sections pertinentes (runes, reactivity, stores, components, SvelteKit routing, etc.).

### 3. svelte-autofixer
**OBLIGATOIRE** avant d’envoyer ou de modifier tout code Svelte (.svelte, .svelte.ts, runes, etc.).  
Appelle-le en boucle jusqu’à ce qu’il ne reste **plus aucune suggestion ni erreur**.  
Puis seulement affiche le code corrigé.

### 4. playground-link
Après avoir terminé un composant, propose à l’utilisateur : « Veux-tu un lien Svelte Playground ? ». Ne l’utilise jamais pour du code écrit directement dans le repo.

**Règle non-négociable** : L’agent SVELTE ne peut jamais proposer du code Svelte brut sans avoir passé `svelte-autofixer` à zéro warning.