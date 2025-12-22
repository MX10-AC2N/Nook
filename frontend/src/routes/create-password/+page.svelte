<script module>
  // Mode Svelte 5 (runes)
  export const runes = true;
</script>

<script>
  import { onMount } from 'svelte';
  import { goto } from '@roxi/routify';  // ← Routify au lieu de $app/navigation (SvelteKit)

  onMount(async () => {
    // Rediriger vers la nouvelle page de changement de mot de passe
    // ou vérifier si l'utilisateur a déjà un mot de passe
    
    try {
      const res = await fetch('/api/member/check-password', {
        credentials: 'include'
      });
      
      if (res.ok) {
        const data = await res.json();
        if (data.has_password) {
          // L'utilisateur a déjà un mot de passe, rediriger vers le chat
          goto('/chat');
        } else {
          // Pas de mot de passe, mais utiliser le nouveau système
          // Vérifier si c'est un mot de passe temporaire
          const changeCheck = await fetch('/api/member/check-password-change', {
            credentials: 'include'
          });
          
          if (changeCheck.ok) {
            const changeData = await changeCheck.json();
            if (changeData.needs_password_change) {
              goto('/change-password');
            } else {
              goto('/chat');
            }
          } else {
            // Ancien système, rester sur cette page
            console.log('Utilisation ancien système de création de mot de passe');
          }
        }
      } else {
        // Non connecté, rediriger vers login
        goto('/login');
      }
    } catch (err) {
      console.error('Erreur vérification:', err);
      goto('/login');
    }
  });
</script>

<svelte:head>
  <title>Créer un mot de passe — Nook</title>
</svelte:head>

<div class="min-h-screen flex flex-col items-center justify-center p-4">
  <div class="text-center">
    <div class="text-6xl mb-4 animate-spin">🌀</div>
    <p class="text-[var(--text-secondary)]">Redirection vers le nouveau système...</p>
  </div>
</div>