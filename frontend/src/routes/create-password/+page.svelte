<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';

  let redirecting = $state(true);
  let redirectMessage = $state('Redirection vers le nouveau système...');

  onMount(async () => {
    try {
      const res = await fetch('/api/member/check-password', {
        credentials: 'include'
      });
      
      if (res.ok) {
        const data = await res.json();
        if (data.has_password) {
          redirectMessage = 'Mot de passe détecté, redirection vers le chat...';
          await new Promise(r => setTimeout(r, 500));
          goto('/chat');
        } else {
          const changeCheck = await fetch('/api/member/check-password-change', {
            credentials: 'include'
          });
          
          if (changeCheck.ok) {
            const changeData = await changeCheck.json();
            if (changeData.needs_password_change) {
              redirectMessage = 'Changement de mot de passe requis...';
              await new Promise(r => setTimeout(r, 500));
              goto('/change-password');
            } else {
              redirectMessage = 'Redirection vers le chat...';
              await new Promise(r => setTimeout(r, 500));
              goto('/chat');
            }
          } else {
            redirectMessage = 'Chargement de la page de création de mot de passe...';
          }
        }
      } else {
        redirectMessage = 'Non connecté, redirection vers la connexion...';
        await new Promise(r => setTimeout(r, 500));
        goto('/login');
      }
    } catch (err) {
      console.error('Erreur vérification:', err);
      redirectMessage = 'Erreur, redirection vers la connexion...';
      await new Promise(r => setTimeout(r, 500));
      goto('/login');
    }
  });
</script>

<svelte:head>
  <title>Créer un mot de passe — Nook</title>
</svelte:head>

<div class="redirect-container">
  <div class="redirect-content">
    <div class="spinner">🌀</div>
    <p class="redirect-message">{redirectMessage}</p>
  </div>
</div>

<style>
  .redirect-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    padding: 1rem;
    background-color: var(--bg-primary, #f0fdf4);
  }

  .redirect-content {
    text-align: center;
  }

  .spinner {
    font-size: 4rem;
    margin-bottom: 1.5rem;
    animation: spin 1.5s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .redirect-message {
    color: var(--text-secondary, #64748b);
    font-size: 1rem;
  }
</style>
