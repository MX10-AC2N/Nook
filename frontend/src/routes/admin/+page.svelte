<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { isAdmin, authLoading, initAuth } from '$lib/authStore';

  let pendingUsers = $state<any[]>([]);
  let allUsers = $state<any[]>([]);
  let invites = $state<any[]>([]);
  let loading = $state(true);
  let activeTab = $state<'pending' | 'all' | 'invites'>('pending');
  let generatingInvite = $state(false);
  let inviteLink = $state<string | null>(null);

  // Guard admin
  onMount(async () => {
    await initAuth();
    if ($authLoading) return;

    if (!$isAdmin) {
      goto('/chat');
      return;
    }

    await Promise.all([loadUsers(), loadInvites()]);
    loading = false;
  });

  async function loadUsers() {
    try {
      const [pendingRes, allRes] = await Promise.all([
        fetch('/api/pending-users-json', { credentials: 'include' }),
        fetch('/api/all-users-json', { credentials: 'include' })
      ]);

      if (pendingRes.ok) {
        const data = await pendingRes.json();
        pendingUsers = data.users || [];
      }
      if (allRes.ok) {
        const data = await allRes.json();
        allUsers = data.users || [];
      }
    } catch (err) {
      console.error('Erreur chargement utilisateurs:', err);
    }
  }

  async function loadInvites() {
    try {
      const res = await fetch('/api/list-invites', { credentials: 'include' });
      if (res.ok) {
        const data = await res.json();
        invites = data.invites || [];
      }
    } catch (err) {
      console.error('Erreur chargement invitations:', err);
    }
  }

  async function approveUser(userId: string) {
    try {
      const response = await fetch('/api/approve', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ user_id: userId })
      });

      if (response.ok) {
        await loadUsers();
      } else {
        alert('Erreur lors de l\'approbation');
      }
    } catch (err) {
      console.error('Erreur approbation:', err);
      alert('Erreur réseau');
    }
  }

  async function generateInvite() {
    generatingInvite = true;
    inviteLink = null;
    try {
      const response = await fetch('/api/generate-invite', {
        method: 'POST',
        credentials: 'include'
      });

      if (response.ok) {
        const data = await response.json();
        inviteLink = data.invite_link;
        await navigator.clipboard.writeText(inviteLink);
        alert('Lien copié dans le presse-papiers !');
        await loadInvites(); // Recharger la liste
      } else {
        alert('Erreur lors de la génération');
      }
    } catch (err) {
      console.error('Erreur génération invite:', err);
      alert('Erreur réseau');
    } finally {
      generatingInvite = false;
    }
  }

  async function deleteInvite(id: string) {
    if (!confirm('Supprimer cette invitation ?')) return;

    try {
      await fetch('/api/delete-invite', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ id })
      });
      await loadInvites();
    } catch (err) {
      alert('Erreur suppression');
    }
  }

  function formatDate(ts: number | string): string {
    const date = typeof ts === 'number' ? new Date(ts * 1000) : new Date(ts);
    return date.toLocaleDateString('fr-FR') { ... });
  }

  function getStatus(invite: any): string {
    if (invite.used) return 'Utilisée';
    if (invite.expired) return 'Expirée';
    return 'Valide';
  }
</script>

<svelte:head>
  <title>Administration - Nook</title>
</svelte:head>

<div class="admin-container">
  <div class="admin-header">
    <h1>👑 Administration</h1>
    <p>Gérez les membres et les invitations de votre espace familial</p>
  </div>

  <div class="admin-actions">
    <button class="invite-btn" on:click={generateInvite} disabled={generatingInvite}>
      {generatingInvite ? 'Génération...' : '➕ Générer un lien d\'invitation'}
    </button>
    {#if inviteLink}
      <p class="invite-link">Dernier lien généré : <code>{inviteLink}</code></p>
    {/if}
  </div>

  <div class="admin-tabs">
    <button class="tab" class:active={activeTab === 'pending'} on:click={() => activeTab = 'pending'}>
      En attente ({pendingUsers.length})
    </button>
    <button class="tab" class:active={activeTab === 'all'} on:click={() => activeTab = 'all'}>
      Membres ({allUsers.length})
    </button>
    <button class="tab" class:active={activeTab === 'invites'} on:click={() => activeTab = 'invites'}>
      Invitations ({invites.length})
    </button>
  </div>

  <div class="admin-content">
    {#if loading}
      <div class="loading">Chargement...</div>
    {:else if activeTab === 'pending'}
      {#if pendingUsers.length === 0}
        <div class="empty-state">Aucun utilisateur en attente</div>
      {:else}
        <div class="user-list">
          {#each pendingUsers as user}
            <div class="user-card pending">
              <div class="user-info">
                <span class="user-name">{user.name || 'Sans nom'}</span>
                <span class="user-username">@{user.username}</span>
                <span class="user-date">Inscrit le {formatDate(user.created_at)}</span>
              </div>
              <button class="approve-btn" on:click={() => approveUser(user.id)}>
                ✅ Approuver
              </button>
            </div>
          {/each}
        </div>
      {/if}
    {:else if activeTab === 'all'}
      <div class="user-list">
        {#each allUsers as user}
          <div class="user-card" class:admin={user.role === 'admin'}>
            <div class="user-info">
              <span class="user-name">
                {user.name || 'Sans nom'}
                {#if user.role === 'admin'} <span class="admin-badge">Admin</span>{/if}
              </span>
              <span class="user-username">@{user.username}</span>
              <span class="user-status" class:approved={user.approved}>
                {user.approved ? '✅ Approuvé' : '⏳ En attente'}
              </span>
            </div>
          </div>
        {/each}
      </div>
    {:else if activeTab === 'invites'}
      {#if invites.length === 0}
        <div class="empty-state">Aucune invitation créée</div>
      {:else}
        <table class="invites-table">
          <thead>
            <tr>
              <th>Créée le</th>
              <th>Expire le</th>
              <th>Statut</th>
              <th>Lien</th>
              <th>Action</th>
            </tr>
          </thead>
          <tbody>
            {#each invites as invite}
              <tr class:expired={invite.expired} class:used={invite.used}>
                <td>{formatDate(invite.created_at)}</td>
                <td>{formatDate(invite.expires_at)}</td>
                <td class="status">{getStatus(invite)}</td>
                <td class="link">
                  <code>{invite.token.slice(0, 12)}...</code>
                  <button on:click={() => navigator.clipboard.writeText(`https://ton-domaine.com/join?token=${invite.token}`)}>
                    Copier
                  </button>
                </td>
                <td>
                  <button class="delete-btn" on:click={() => deleteInvite(invite.id)} disabled={invite.used || invite.expired}>
                    Supprimer
                  </button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    {/if}
  </div>
</div>

<style>
  .admin-container { max-width: 900px; margin: 0 auto; padding: 1rem; }
  .admin-header { text-align: center; margin-bottom: 2rem; }
  .admin-header h1 { font-size: 1.75rem; color: #2d5a27; }
  .admin-header p { color: #666; }

  .admin-actions { text-align: center; margin-bottom: 1.5rem; }
  .invite-btn { padding: 0.75rem 1.5rem; background: #2d5a27; color: white; border: none; border-radius: 8px; cursor: pointer; }
  .invite-btn:hover:not(:disabled) { background: #3d7a37; }

  .invite-link { margin-top: 0.8rem; word-break: break-all; }
  .invite-link code { background: #f0f0f0; padding: 0.3rem 0.6rem; border-radius: 4px; }

  .admin-tabs { display: flex; gap: 0.5rem; margin-bottom: 1.5rem; justify-content: center; flex-wrap: wrap; }
  .tab { padding: 0.75rem 1.25rem; background: none; border: none; cursor: pointer; color: #666; border-radius: 8px 8px 0 0; }
  .tab.active { background: #2d5a27; color: white; }

  .admin-content { background: white; border-radius: 12px; box-shadow: 0 2px 8px rgba(0,0,0,0.1); overflow: hidden; }

  .loading, .empty-state { padding: 3rem; text-align: center; color: #888; }

  .user-list, .invites-table { width: 100%; }
  .user-card { display: flex; justify-content: space-between; align-items: center; padding: 1rem; border-bottom: 1px solid #eee; }
  .user-card.pending { background: #fff8e1; }
  .user-card.admin { background: #e3f2fd; }

  .user-info { display: flex; flex-direction: column; gap: 0.25rem; }
  .user-name { font-weight: 500; display: flex; align-items: center; gap: 0.5rem; }
  .admin-badge { font-size: 0.7rem; padding: 0.2rem 0.5rem; background: #2196f3; color: white; border-radius: 4px; }
  .user-username, .user-date, .user-status { font-size: 0.85rem; color: #666; }
  .user-status.approved { color: #4caf50; }

  .approve-btn { padding: 0.5rem 1rem; background: #4caf50; color: white; border: none; border-radius: 6px; cursor: pointer; }
  .approve-btn:hover { background: #43a047; }

  .invites-table { border-collapse: collapse; }
  .invites-table th, .invites-table td { padding: 0.75rem; text-align: left; border-bottom: 1px solid #eee; }
  .invites-table th { background: #f8f9fa; font-weight: 600; }
  .status { font-weight: 500; }
  .expired { opacity: 0.6; }
  .used { opacity: 0.6; background: #f0f0f0; }
  .link code { font-size: 0.8rem; background: #f0f0f0; padding: 0.2rem 0.4rem; border-radius: 4px; }
  .delete-btn { background: #dc2626; color: white; border: none; padding: 0.4rem 0.8rem; border-radius: 4px; cursor: pointer; }
  .delete-btn:disabled { opacity: 0.5; cursor: not-allowed; }
</style>