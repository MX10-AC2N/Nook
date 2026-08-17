<script lang="ts">
  // -----------------------------------------------------------------
  // Imports du store de thème
  // -----------------------------------------------------------------
  import {
    availableThemes,
    getCurrentTheme,   // état réactif exporté depuis ThemeStore.ts
    setTheme,
    type Theme
  } from '$lib/ui/ThemeStore.svelte.ts';

  // -----------------------------------------------------------------
  // États locaux du composant (Svelte 5 runes)
  // -----------------------------------------------------------------
  // Ouverture/fermeture du dropdown
  let isOpen = $state(false);

  // Pas besoin de copier le thème : on utilise directement getCurrentTheme
  // Si vous avez besoin d’une valeur dérivée, utilisez $derived.
  // Exemple (non obligatoire ici) :
  // let selectedTheme = $derived(() => $currentTheme);

  // -----------------------------------------------------------------
  // Fonctions d’interaction
  // -----------------------------------------------------------------
  function toggleDropdown() {
    isOpen = !isOpen;
  }

  function closeDropdown() {
    isOpen = false;
  }

  function selectTheme(themeId: Theme) {
    setTheme(themeId);
    closeDropdown();
  }

  // Fermer le dropdown avec <Esc>
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape' && isOpen) {
      closeDropdown();
    }
  }

  // Fermer le dropdown lorsqu’on clique à l’extérieur du composant
  function handleClickOutside(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (!target.closest('.theme-switcher')) {
      closeDropdown();
    }
  }

  // -----------------------------------------------------------------
  // Valeur dérivée : l’objet complet du thème sélectionné
  // -----------------------------------------------------------------
  // `$currentTheme` contient l’identifiant du thème actif.
  // On le transforme en l’objet complet présent dans `availableThemes`.
  let currentThemeInfo = $derived(
    () => availableThemes.find(t => t.id === getCurrentTheme())
  );

  // -----------------------------------------------------------------
  // Gestion globale des événements (esc + click extérieur)
  // -----------------------------------------------------------------
  // (déclaré dans le markup via <svelte:window>)
</script>

<!-- -----------------------------------------------------------------
     Gestion globale des événements clavier / clic extérieur
----------------------------------------------------------------- -->
<svelte:window onkeydown={handleKeydown} onclick={handleClickOutside} />

<div class="theme-switcher">
  <!-- -------------------------------------------------------------
       Bouton principal du sélecteur de thème
       ------------------------------------------------------------- -->
  <button
    class="theme-switcher-trigger"
    onclick={toggleDropdown}
    aria-expanded={isOpen}
    aria-haspopup="listbox"
    aria-label="Changer de thème"
  >
    <span class="theme-icon" aria-hidden="true">
      {#if currentThemeInfo}{currentThemeInfo.icon}{:else}🎨{/if}
    </span>

    <span class="theme-name">
      {#if currentThemeInfo}{currentThemeInfo.name}{:else}Thème{/if}
    </span>

    <!-- Chevron qui pivote quand le menu est ouvert -->
    <svg
      class="chevron"
      class:open={isOpen}
      width="16"
      height="16"
      viewBox="0 0 16 16"
      fill="none"
      aria-hidden="true"
    >
      <path
        d="M4 6L8 10L12 6"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
  </button>

  <!-- -------------------------------------------------------------
       Dropdown contenant la liste des thèmes
       ------------------------------------------------------------- -->
  {#if isOpen}
    <div
      class="theme-dropdown"
      role="listbox"
      aria-label="Sélectionner un thème"
    >
      <div class="dropdown-header">
        <span>Choisir un thème</span>
      </div>

      <div class="themes-list" role="group">
        {#each availableThemes as theme (theme.id)}
          <button
            class="theme-option"
            class:active={getCurrentTheme() === theme.id}
            onclick={() => selectTheme(theme.id)}
            role="option"
            aria-selected={getCurrentTheme() === theme.id}
          >
            <!-- Aperçu colorimétrique du thème -->
            <div class="theme-preview" data-theme={theme.id}>
              <div class="preview-circle primary"></div>
              <div class="preview-circle secondary"></div>
              <div class="preview-circle accent"></div>
            </div>

            <!-- Infos du thème -->
            <div class="theme-info">
              <span class="theme-option-name">{theme.icon} {theme.name}</span>
              <span class="theme-option-description">{theme.description}</span>
            </div>

            <!-- Icône de validation si ce thème est sélectionné -->
            {#if getCurrentTheme() === theme.id}
              <svg
                class="check-icon"
                width="16"
                height="16"
                viewBox="0 0 16 16"
                fill="none"
                aria-hidden="true"
              >
                <path
                  d="M13 4L6 12L3 9"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                />
              </svg>
            {/if}
          </button>
        {/each}
      </div>

      <!-- Footer informatif -->
      <div class="dropdown-footer">
        <span class="footer-text">💡 Le thème système est détecté automatiquement</span>
      </div>
    </div>
  {/if}
</div>

<style>
  /* -----------------------------------------------------------------
     CONTENEUR PRINCIPAL
     ----------------------------------------------------------------- */
  .theme-switcher {
    position: relative;
    display: inline-block;
    font-family: var(--font-primary, sans-serif);
  }

  /* -----------------------------------------------------------------
     BOUTON PRINCIPAL
     ----------------------------------------------------------------- */
  .theme-switcher-trigger {
    display: flex;
    align-items: center;
    gap: var(--space-2, 0.5rem);
    padding: var(--space-2, 0.5rem) var(--space-3, 0.75rem);
    background-color: var(--bg-secondary, #f1f5f9);
    border: 1px solid var(--border, #e2e8f0);
    border-radius: var(--radius-lg, 0.75rem);
    cursor: pointer;
    font-size: var(--text-sm, 0.875rem);
    color: var(--text-primary, #1e293b);
    transition: all 150ms ease;
    min-width: 140px;
  }
  .theme-switcher-trigger:hover {
    background-color: var(--bg-tertiary, #e2e8f0);
    border-color: var(--accent, #4ade80);
  }
  .theme-switcher-trigger:focus {
    outline: none;
    box-shadow: 0 0 0 2px var(--accent, #4ade80);
  }
  .theme-icon {
    font-size: 1.125rem;
  }
  .theme-name {
    flex: 1;
    text-align: left;
  }
  .chevron {
    transition: transform 200ms ease;
    color: var(--text-secondary, #64748b);
  }
  .chevron.open {
    transform: rotate(180deg);
  }

  /* -----------------------------------------------------------------
     DROPDOWN
     ----------------------------------------------------------------- */
  .theme-dropdown {
    position: absolute;
    top: calc(100% + var(--space-2, 0.5rem));
    right: 0;
    min-width: 280px;
    background-color: var(--bg-primary, #ffffff);
    border: 1px solid var(--border, #e2e8f0);
    border-radius: var(--radius-xl, 1rem);
    box-shadow: var(--depth, 0 4px 12px rgba(0, 0, 0, 0.15));
    z-index: 1000;
    animation: dropdown-enter 200ms ease forwards;
    overflow: hidden;
  }
  @keyframes dropdown-enter {
    from {
      opacity: 0;
      transform: translateY(-8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
  .dropdown-header {
    padding: var(--space-3, 0.75rem) var(--space-4, 1rem);
    font-weight: 600;
    font-size: var(--text-sm, 0.875rem);
    color: var(--text-primary, #1e293b);
    border-bottom: 1px solid var(--border, #e2e8f0);
    background-color: var(--bg-secondary, #f8fafc);
  }

  /* -----------------------------------------------------------------
     LISTE DES THÈMES
     ----------------------------------------------------------------- */
  .themes-list {
    padding: var(--space-2, 0.5rem);
    max-height: 300px;
    overflow-y: auto;
  }
  .theme-option {
    display: flex;
    align-items: center;
    gap: var(--space-3, 0.75rem);
    width: 100%;
    padding: var(--space-3, 0.75rem);
    background: none;
    border: 1px solid transparent;
    border-radius: var(--radius-lg, 0.75rem);
    cursor: pointer;
    text-align: left;
    transition: all 150ms ease;
    color: var(--text-primary, #1e293b);
  }
  .theme-option:hover {
    background-color: var(--bg-secondary, #f1f5f9);
  }
  .theme-option.active,
  .theme-option:focus {
    background-color: var(--bg-tertiary, #e2e8f0);
    border-color: var(--accent, #4ade80);
    outline: none;
    box-shadow: 0 0 0 2px var(--accent, #4ade80);
  }

  /* -----------------------------------------------------------------
     APERÇU DU THÈME
     ----------------------------------------------------------------- */
  .theme-preview {
    display: flex;
    gap: 4px;
    flex-shrink: 0;
  }
  .preview-circle {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    border: 2px solid white;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
  }
  /* Couleurs spécifiques à chaque thème */
  .theme-preview[data-theme="jardin-secret"] .preview-circle.primary   { background-color: #f0fdf4; }
  .theme-preview[data-theme="jardin-secret"] .preview-circle.secondary { background-color: #e0f2fe; }
  .theme-preview[data-theme="jardin-secret"] .preview-circle.accent    { background-color: #4ade80; }
  .theme-preview[data-theme="space-hub"] .preview-circle.primary   { background-color: #0f172a; }
  .theme-preview[data-theme="space-hub"] .preview-circle.secondary { background-color: #1e293b; }
  .theme-preview[data-theme="space-hub"] .preview-circle.accent    { background-color: #8b5cf6; }
  .theme-preview[data-theme="maison-chaleureuse"] .preview-circle.primary   { background-color: #fdf2e9; }
  .theme-preview[data-theme="maison-chaleureuse"] .preview-circle.secondary { background-color: #fef3c7; }
  .theme-preview[data-theme="maison-chaleureuse"] .preview-circle.accent    { background-color: #ea580c; }

  /* -----------------------------------------------------------------
     INFORMATIONS DU THÈME
     ----------------------------------------------------------------- */
  .theme-info {
    flex: 1;
    min-width: 0;
  }
  .theme-option-name {
    display: block;
    font-weight: 500;
    font-size: var(--text-sm, 0.875rem);
    color: var(--text-primary, #1e293b);
  }
  .theme-option-description {
    display: block;
    font-size: var(--text-xs, 0.75rem);
    color: var(--text-secondary, #64748b);
    margin-top: 2px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* -----------------------------------------------------------------
     ICÔNE DE VALIDATION
     ----------------------------------------------------------------- */
  .check-icon {
    flex-shrink: 0;
    color: var(--accent, #4ade80);
  }

  /* -----------------------------------------------------------------
     FOOTER
     ----------------------------------------------------------------- */
  .dropdown-footer {
    padding: var(--space-3, 0.75rem) var(--space-4, 1rem);
    border-top: 1px solid var(--border, #e2e8f0);
    background-color: var(--bg-secondary, #f8fafc);
  }
  .footer-text {
    font-size: var(--text-xs, 0.75rem);
    color: var(--text-secondary, #64748b);
  }

  /* -----------------------------------------------------------------
     RESPONSIVE
     ----------------------------------------------------------------- */
  @media (max-width: 480px) {
    .theme-dropdown {
      position: fixed;
      top: auto;
      bottom: 0;
      left: 0;
      right: 0;
      min-width: 100%;
      border-radius: var(--radius-xl, 1rem) var(--radius-xl, 1rem) 0 0;
      animation: slide-up-mobile 300ms ease forwards;
    }
    @keyframes slide-up-mobile {
      from {
        opacity: 0;
        transform: translateY(100%);
      }
      to {
        opacity: 1;
        transform: translateY(0);
      }
    }
    .theme-switcher-trigger {
      min-width: auto;
      padding: var(--space-2, 0.5rem);
    }
    .theme-name {
      display: none;
    }
  }
</style>
