---
type: procedural
scope: global
domain: nook.architect.adr-recovery
tags: [nook, architect, 2026-10-22, adr-017, adr-governance, reply-to, recovery-provenance, plur-convention]
statement: "Récupération d'ADR perdu (ADR-017 reply-to, carte t_0b0decb6) : un ADR disparu du disque n'est réécrit que si un artefact DATÉ contient le texte original — ici le dump de session coder du 2026-08-28 (profiles/coder/sessions/request_dump_20260828_171413_*.json, résultat brut d'un read_file de 430 lignes), retrouvé à l'identique à une ligne près (Status: Proposé -> Accepté, acté sur preuve de code : backend/migrations/024_reply_to.sql, backend/src/db.rs:415/769/812, frontend/src/lib/chatStore.svelte.ts:558, commit eca8093e). Si aucune source datée n'existe, on documente la perte dans docs/adr/README.md plutôt que de fabriquer le texte. Référence cassée corrigée au passage : docs/adr/018-agent-memory-storage.md:8 (« non encore commitée » -> lien vers 017-reply-to-feature.md). Écart consigné : la migration 024 ajoute idx_messages_reply_to, non prévu par la spec §1."
---
