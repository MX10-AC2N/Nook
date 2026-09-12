# 📋 Tâche Kanban — t_emoji-picker-flip

> **Nom**: FIX-EMOJI-PICKER-FLIP
> **Assigné**: coder
> **Status**: 🔄 TODO
> **Créé**: 2026-09-11
> **Priorité**: 🟡 MEDIUM

---

## Description

Le picker emoji de Nook s'ouvre **au-dessus** des messages au lieu de **en dessous**. Le bug est dans `openMsgEmojiPicker()` dans `frontend/src/routes/chat/+page.svelte`.

## Root Cause

Le code calcule `availableHeight` mais ne l'utilise PAS pour décider de positionner le picker au-dessus. `emojiPickerPos.top` est toujours fixé à `bottomEdge` (ligne 398) sans vérifier si `bottomEdge + pickerHeight > window.innerHeight`.

## Fix Requis

**Fichier**: `frontend/src/routes/chat/+page.svelte`
**Lignes**: ~397-402 (dans `openMsgEmojiPicker()`)

Le code actuel (lignes 397-402) :
```
emojiPickerPos = {
  top: bottomEdge,
  left,
  right: window.innerWidth - left - pickerWidth,
  maxHeight: finalPickerHeight,
};
```

Ajouter une logique de flip conditionnelle AVANT ce bloc :

```
// Calculer si le picker dépasse le bas de l'écran
const exceedsBottom = bottomEdge + pickerHeight > window.innerHeight - margin;
const canFlipAbove = msgRect.top > pickerHeight + margin;

if (exceedsBottom && canFlipAbove) {
  // Flip: positionner au-dessus du message
  emojiPickerPos = {
    top: msgRect.top - pickerHeight - margin,
    left,
    right: window.innerWidth - left - pickerWidth,
    maxHeight: finalPickerHeight,
  };
} else {
  // Positionnement normal (en dessous)
  emojiPickerPos = {
    top: bottomEdge,
    left,
    right: window.innerWidth - left - pickerWidth,
    maxHeight: finalPickerHeight,
  };
}
```

## Critères d'Acceptation

- Le picker s'ouvre en dessous du message quand il y a de la place
- Le picker s'ouvre au-dessus du message quand il n'y a pas de place en bas
- Le picker reste visible dans le viewport (pas de débordement)
- Le scroll de la page ne déplace pas le picker hors viewport
- Les tests Playwright de chat UI passent

## Fichiers Impliqués

- frontend/src/routes/chat/+page.svelte — openMsgEmojiPicker() (~ligne 355-403)
- frontend/src/routes/chat/+page.svelte — CSS .msg-emoji-picker (~ligne 3286)

## Tests

1. npx playwright test --list — valider la syntaxe
2. npx playwright test tests/chat-ui.spec.ts — tester le chat UI
3. npx playwright test tests/comprehensive-chat.spec.ts — test complet
4. Verifier manuellement sur 192.168.1.192:6300

## Notes Techniques

- emoji-picker-element est un web component custom
- Le picker est positionné via style="position: fixed; top: {emojiPickerPos.top}px; ..."
- emojiPickerPos est un $state Svelte 5
- La logique actuelle calcule availableHeight mais ne l'utilise pas pour le flip → gas

## Delegation

- Deleguer a: coder via delegate_task
- Type: code fix (2 lignes Svelte)
- Review: tester sur le homeserver 192.168.1.192:6300
- PLUR engram: après résolution
