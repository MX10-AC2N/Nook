<script lang="ts">
  import { goto } from '$app/navigation';

  let name = $state('');
  let username = $state('');
  let email = $state('');
  let password = $state('');
  let confirmPassword = $state('');
  let error = $state('');
  let success = $state(false);
  let loading = $state(false);

  async function handleRegister() {
    if (!name || !username || !email || !password || !confirmPassword) {
      error = 'Veuillez remplir tous les champs';
      return;
    }

    if (password !== confirmPassword) {
      error = 'Les mots de passe ne correspondent pas';
      return;
    }

    if (password.length < 8) {
      error = 'Le mot de passe doit contenir au moins 8 caractères';
      return;
    }

    loading = true;
    error = '';

    try {
      const response = await fetch('/api/auth/register', {  // ← Endpoint correct
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ 
          name, 
          username, 
          email, 
          password 
        })
      });

      const data = await response.json();

      if (response.ok) {
        success = true;
      } else {
        error = data.message || 'Erreur lors de l\'inscription (ex: username/email déjà pris)';
      }
    } catch (err) {
      error = 'Erreur de connexion au serveur';
    } finally {
      loading = false;
    }
  }
</script>

<svelte:head>
  <title>Inscription libre - Nook</title>
</svelte:head>

<div class="register-container">
  {#if success}
    <div class="success-card">
      <h1>✅ Inscription envoyée</h1>
      <p>Votre demande a été soumise à l'administrateur pour approbation.</p>
      <p>Vous pourrez vous connecter une fois approuvé.</p>
      <a href="/login" class="back-btn">Retour à la connexion</a>
    </div>
  {:else}
    <div class="register-card">
      <h1>🌱 Inscription Nook</h1>
      <p class="subtitle">Créez un compte (en attente d'approbation admin)</p>

      {#if error}
        <div class="error-message">{error}</div>
      {/if}

      <form on:submit|preventDefault={handleRegister}>
        <div class="form-group">
          <label for="name">Prénom/Nom</label>
          <input type="text" id="name" bind:value={name} placeholder="Jean Dupont" disabled={loading} required />
        </div>

        <div class="form-group">
          <label for="username">Identifiant</label>
          <input type="text" id="username" bind:value={username} placeholder="jean" disabled={loading} required />
        </div>

        <div class="form-group">
          <label for="email">Email</label>
          <input type="email" id="email" bind:value={email} placeholder="jean@example.com" disabled={loading} required />
        </div>

        <div class="form-group">
          <label for="password">Mot de passe</label>
          <input type="password" id="password" bind:value={password} placeholder="Au moins 8 caractères" disabled={loading} required />
        </div>

        <div class="form-group">
          <label for="confirmPassword">Confirmer mot de passe</label>
          <input type="password" id="confirmPassword" bind:value={confirmPassword} placeholder="Répétez" disabled={loading} required />
        </div>

        <button type="submit" class="register-btn" disabled={loading}>
          {loading ? 'Inscription...' : 'S\'inscrire'}
        </button>
      </form>

      <div class="links">
        <a href="/login">Déjà un compte ? Se connecter</a>
      </div>
    </div>
  {/if}
</div>

<style>
	.register-container { display: flex; align-items: center; justify-content: center; min-height: calc(100vh - 100px); padding: 1rem; }
	.register-card, .success-card { background: white; padding: 2rem; border-radius: 16px; box-shadow: 0 4px 20px rgba(0,0,0,0.1); width: 100%; max-width: 400px; text-align: center; }
	.success-card h1 { color: #2d5a27; margin-bottom: 1rem; }
	h1 { font-size: 2rem; margin-bottom: 0.5rem; }
	.subtitle { color: #666; margin-bottom: 2rem; }
	.form-group { margin-bottom: 1.25rem; text-align: left; }
	label { display: block; margin-bottom: 0.5rem; font-weight: 500; color: #333; }
	input { width: 100%; padding: 0.75rem; border: 2px solid #e0e0e0; border-radius: 8px; font-size: 1rem; }
	input:focus { outline: none; border-color: #2d5a27; }
	.error-message { background: #ffebee; color: #c62828; padding: 0.75rem; border-radius: 8px; margin-bottom: 1rem; font-size: 0.9rem; }
	.register-btn { width: 100%; padding: 0.875rem; background: #2d5a27; color: white; border: none; border-radius: 8px; font-size: 1rem; font-weight: 500; cursor: pointer; }
	.register-btn:hover:not(:disabled) { background: #3d7a37; }
	.register-btn:disabled { opacity: 0.6; cursor: not-allowed; }
	.back-btn { display: inline-block; margin-top: 1.5rem; padding: 0.75rem 1.5rem; background: #2d5a27; color: white; text-decoration: none; border-radius: 8px; }
	.links { margin-top: 1.5rem; }
	.links a { color: #2d5a27; text-decoration: none; font-size: 0.9rem; }
</style>
