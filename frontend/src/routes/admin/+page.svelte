<!-- frontend/src/routes/admin/+page.svelte — Session 34
     Ajouts :
       - Onglet "Analytics" → goto('/admin/analytics')
       - Badge dynamique : affiche authStore.user.name (pas "admin" hardcodé)
       - Couleurs via variables CSS thème (var(--accent) etc.)
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { authStore } from '$lib/authStore.svelte.js';

  let pendingUsers     = $state<any[]>([]);
  let allUsers         = $state<any[]>([]);
  import { notifyAdmin } from '$lib/notificationStore.svelte';
  let invites          = $state<any[]>([]);
  let loading          = $state(true);
  let activeTab        = $state<'pending' | 'all' | 'invites'>('pending');
  let generatingInvite = $state(false);
  let inviteLink       = $state<string | null>(null);
  let authChecked      = $state(false);

  async function deleteUser(userId: string) {
    if (!confirm('Supprimer définitivement ce membre ? Cette action est irréversible.')) return;
    try {
      const res = await fetch(`/api/users/${userId}`, {
        method: 'DELETE',
        credentials: 'include',
      });
      const data = await res.json();
      if (!res.ok) { alert(data.message ?? 'Erreur'); return; }
      allUsers = allUsers.filter(u => u.id !== userId);
    } catch (e) { alert('Erreur réseau'); }
  }

  onMount(async () => {
    const ok = await checkAuthAndRedirect();
    if (ok) {
      authChecked = true;
      await loadAdminData();
    }
  });

  async function checkAuthAndRedirect(): Promise<boolean> {
    try {
      const response = await fetch('/api/auth/me', { credentials: 'include' });
      if (!response.ok) throw new Error('Non-authentifié');
      const data = await response.json();
      if (data.authenticated && data.user) {
        authStore.login(data.user);
        if (data.user.role !== 'admin') { goto('/chat'); return false; }
        return true;
      }
      goto('/login'); return false;
    } catch (e) {
      console.error('Erreur auth admin :', e);
      goto('/login'); return false;
    }
  }

  async function loadAdminData() {
    try { await Promise.all([loadUsers(), loadInvites()]); }
    catch (e) { console.error('Erreur chargement admin :', e); }
    finally { loading = false; }
  }

  async function loadUsers() {
    try {
      const [pendingRes, allRes] = await Promise.all([
        fetch('/api/users/pending', { credentials: 'include' }),
        fetch('/api/users',         { credentials: 'include' }),
      ]);
      if (pendingRes.ok) { const d = await pendingRes.json(); pendingUsers = Array.isArray(d) ? d : (d.users || []); }
      if (allRes.ok)     { const d = await allRes.json();     allUsers     = Array.isArray(d) ? d : (d.users || []); }
    } catch (e) { console.error('Erreur chargement utilisateurs :', e); }
  }

  async function loadInvites() {
    try {
      const res = await fetch('/api/invites', { credentials: 'include' });
      if (res.ok) { const d = await res.json(); invites = Array.isArray(d) ? d : (d.invites || []); }
    } catch (e) { console.error('Erreur chargement invitations :', e); }
  }

  async function approveUser(userId: string) {
    try {
      const response = await fetch('/api/users/approve', {
        method: 'POST', headers: { 'Content-Type': 'application/json' },
        credentials: 'include', body: JSON.stringify({ user_id: userId }),
      });
      if (response.ok) { await loadUsers(); alert('✅ Utilisateur approuvé avec succès'); }
      else alert("Erreur lors de l'approbation");
    } catch (e) { alert('Erreur réseau'); }
  }

  async function generateInvite() {
    generatingInvite = true; inviteLink = null;
    try {
      const response = await fetch('/api/invites', { method: 'POST', credentials: 'include' });
      if (response.ok) {
        const data = await response.json();
        const inviteToken = data.invite_link?.split('token=')[1] ?? data.token;
        inviteLink = `${window.location.origin}/invite?token=${inviteToken}`;
    notifyAdmin('Invitation generee', 'Un nouveau lien d invitation a ete cree');
        try { await navigator.clipboard.writeText(inviteLink); alert('✅ Lien copié dans le presse-papiers !'); }
        catch { console.warn('Clipboard non disponible'); }
        await loadInvites();
      } else alert('Erreur lors de la génération');
    } catch (e) { alert('Erreur réseau'); }
    finally { generatingInvite = false; }
  }

  async function deleteInvite(id: string) {
    if (!confirm('Supprimer cette invitation ?')) return;
    try {
      const res = await fetch('/api/invites/delete', {
        method: 'POST', headers: { 'Content-Type': 'application/json' },
        credentials: 'include', body: JSON.stringify({ id }),
      });
      if (res.ok) await loadInvites(); else alert('Erreur lors de la suppression');
    } catch (e) { alert('Erreur réseau'); }
  }

  function formatDate(ts: number | string): string {
    const date = typeof ts === 'number' ? new Date(ts * 1000) : new Date(ts);
    return date.toLocaleDateString('fr-FR', { day: '2-digit', month: '2-digit', year: 'numeric', hour: '2-digit', minute: '2-digit' });
  }
  function isExpired(invite: any): boolean { return Date.now() / 1000 > (invite.expires_at || 0); }
  function getStatus(invite: any): string {
    if (invite.used) return 'Utilisée';
    if (isExpired(invite)) return 'Expirée';
    return 'Valide';
  }
</script>

<svelte:head><title>Administration - Nook</title></svelte:head>

<div class="admin-container">
  {#if !authChecked}
    <div class="loading-fullpage">
      <div class="spinner-large"></div>
      <p>Vérification des permissions…</p>
    </div>

  {:else if !authStore.isAdmin}
    <div class="not-authorized">
      <h2>Accès non autorisé</h2>
      <p>Vous n'avez pas les permissions nécessaires.</p>
      <button onclick={() => goto('/chat')}>Aller au chat</button>
    </div>

  {:else}
    <div class="admin-header">
      <h1>👑 Administration</h1>
      <div class="auth-status">
        <span class="admin-badge">
          Connecté en tant que {authStore.user?.name || authStore.user?.username || 'admin'}
        </span>
      </div>
      <p>Gérez les membres et les invitations de votre espace familial</p>
    </div>

    {#if loading}
      <div class="loading-message">Chargement des données d'administration…</div>
    {:else}
      <div class="admin-actions">
        <button class="invite-btn" onclick={generateInvite} disabled={generatingInvite}>
          {generatingInvite ? 'Génération…' : "➕ Générer un lien d'invitation"}
        </button>
        {#if inviteLink}
          <p class="invite-link">Dernier lien généré : <code>{inviteLink}</code></p>
        {/if}
      </div>

      <div class="admin-tabs">
        <button class="tab" class:active={activeTab === 'pending'} onclick={() => (activeTab = 'pending')}>
          ⏳ En attente ({pendingUsers.length})
        </button>
        <button class="tab" class:active={activeTab === 'all'} onclick={() => (activeTab = 'all')}>
          👥 Membres ({allUsers.length})
        </button>
        <button class="tab" class:active={activeTab === 'invites'} onclick={() => (activeTab = 'invites')}>
          🔗 Invitations ({invites.length})
        </button>
        <button class="tab tab-analytics" onclick={() => goto('/admin/analytics')}>
          📊 Analytics ↗
        </button>
      </div>

      <div class="admin-content">
        {#if activeTab === 'pending'}
          {#if pendingUsers.length === 0}
            <div class="empty-state">✅ Aucun utilisateur en attente</div>
          {:else}
            <div class="user-list">
              {#each pendingUsers as user}
                <div class="user-card pending">
                  <div class="user-info">
                    <span class="user-name">{user.name || 'Sans nom'}</span>
                    <span class="user-username">@{user.username}</span>
                    <span class="user-date">Inscrit le {formatDate(user.created_at)}</span>
                  </div>
                  <button class="approve-btn" onclick={() => approveUser(user.id)}>✅ Approuver</button>
                </div>
              {/each}
            </div>
          {/if}

        {:else if activeTab === 'all'}
          <div class="user-list">
            {#each allUsers as user}
              <div class="user-card" class:admin-card={user.role === 'admin'}>
                <div class="user-info">
                  <span class="user-name">
                    {user.name || 'Sans nom'}
                    {#if user.role === 'admin'}<span class="role-badge">Admin</span>{/if}
                  </span>
                  <span class="user-username">@{user.username}</span>
                  <span class="user-status" class:approved={user.approved}>
                    {user.approved ? '✅ Approuvé' : '⏳ En attente'}
                  </span>
                </div>
                {#if user.role !== 'admin'}
                  <button class="delete-user-btn" onclick={() => deleteUser(user.id)}
                    title="Supprimer ce membre">🗑</button>
                {/if}
              </div>
            {/each}
          </div>

        {:else if activeTab === 'invites'}
          {#if invites.length === 0}
            <div class="empty-state">Aucune invitation créée</div>
          {:else}
            <table class="invites-table">
              <thead>
                <tr><th>Créée le</th><th>Expire le</th><th>Statut</th><th>Lien</th><th>Action</th></tr>
              </thead>
              <tbody>
                {#each invites as invite}
                  <tr class:expired={isExpired(invite)} class:used={invite.used}>
                    <td>{formatDate(invite.created_at)}</td>
                    <td>{formatDate(invite.expires_at)}</td>
                    <td class="status">{getStatus(invite)}</td>
                    <td class="link">
                      <code>{invite.token.slice(0, 12)}…</code>
                      <button onclick={async () => {
                        const link = `${window.location.origin}/invite?token=${invite.token}`;
                        try { await navigator.clipboard.writeText(link); alert('✅ Lien copié !'); }
                        catch { prompt('Copiez ce lien :', link); }
                      }}>Copier</button>
                    </td>
                    <td>
                      <button class="delete-btn" onclick={() => deleteInvite(invite.id)}
                        disabled={invite.used || isExpired(invite)}>Supprimer</button>
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          {/if}
        {/if}
      </div>
    {/if}
  {/if}
</div>

<style>
  .loading-fullpage { display: flex; flex-direction: column; align-items: center; justify-content: center; height: 60vh; color: var(--text-secondary); }
  .spinner-large { width: 40px; height: 40px; border: 4px solid var(--border); border-top-color: var(--accent); border-radius: 50%; animation: spin 1s linear infinite; margin-bottom: 1rem; }
  @keyframes spin { to { transform: rotate(360deg); } }

  .admin-container { max-width: 900px; margin: 0 auto; padding: 1rem; color: var(--text-primary); }

  .admin-header { text-align: center; margin-bottom: 2rem; }
  .admin-header h1 { font-size: 1.75rem; color: var(--accent-dark, var(--accent)); margin-bottom: 0.5rem; }
  .admin-header p  { color: var(--text-secondary); }
  .auth-status { margin: 0.5rem 0; }

  .admin-badge {
    display: inline-block; padding: 0.25rem 0.75rem; border-radius: var(--radius-full);
    font-size: 0.8rem; font-weight: 600;
    background: color-mix(in srgb, var(--accent) 20%, transparent);
    color: var(--accent-dark, var(--accent));
    border: 1px solid var(--border);
  }

  .admin-actions { text-align: center; margin-bottom: 1.5rem; }
  .invite-btn {
    padding: 0.75rem 1.5rem; background: var(--accent); color: #fff;
    border: none; border-radius: var(--radius-lg); cursor: pointer;
    font-weight: 600; transition: all 0.2s;
  }
  .invite-btn:hover:not(:disabled) { background: var(--button-hover); transform: translateY(-1px); }
  .invite-btn:disabled { opacity: 0.6; cursor: not-allowed; }
  .invite-link { margin-top: 0.8rem; word-break: break-all; color: var(--text-secondary); font-size: 0.9rem; }
  .invite-link code { background: var(--bg-secondary); padding: 0.3rem 0.6rem; border-radius: var(--radius-md); font-size: 0.85rem; }

  .admin-tabs { display: flex; gap: 0.5rem; margin-bottom: 1.5rem; justify-content: center; flex-wrap: wrap; }
  .tab {
    padding: 0.65rem 1.1rem; background: var(--bg-secondary); border: 1px solid var(--border);
    cursor: pointer; color: var(--text-secondary); border-radius: var(--radius-lg);
    font-size: 0.9rem; font-weight: 500; transition: all 0.2s;
  }
  .tab:hover  { background: var(--border); color: var(--text-primary); }
  .tab.active { background: var(--accent); color: #fff; border-color: transparent; }
  /* Onglet Analytics : distinct, pas "actif" car redirige vers une autre page */
  .tab-analytics { border-color: var(--accent); color: var(--accent); background: transparent; }
  .tab-analytics:hover { background: var(--accent); color: #fff; }

  .admin-content { background: var(--bg-primary); border-radius: var(--radius-xl); border: 1px solid var(--border); box-shadow: var(--depth); overflow: hidden; }
  .loading-message, .empty-state { text-align: center; padding: 3rem; color: var(--text-secondary); }

  .not-authorized { text-align: center; padding: 3rem; background: var(--bg-primary); border-radius: var(--radius-xl); box-shadow: var(--depth); }
  .not-authorized h2 { color: #dc2626; margin-bottom: 1rem; }
  .not-authorized button { margin-top: 1rem; padding: 0.75rem 1.5rem; background: var(--accent); color: white; border: none; border-radius: var(--radius-lg); cursor: pointer; }

  .user-list { display: flex; flex-direction: column; gap: 0.75rem; padding: 1rem; }
  .user-card {
    display: flex; justify-content: space-between; align-items: center;
    padding: 0.75rem 1rem; border-radius: var(--radius-lg);
    background: var(--bg-secondary); border: 1px solid var(--border);
  }
  .user-card.pending    { background: color-mix(in srgb, #fbbf24 12%, var(--bg-secondary)); border-color: #fbbf24; }
  .user-card.admin-card { background: color-mix(in srgb, var(--accent) 10%, var(--bg-secondary)); border-color: var(--accent); }
  .user-info { display: flex; flex-direction: column; gap: 0.2rem; }
  .user-name { font-weight: 600; display: flex; align-items: center; gap: 0.5rem; color: var(--text-primary); }
  .role-badge { font-size: 0.7rem; padding: 0.15rem 0.45rem; background: var(--accent); color: #fff; border-radius: var(--radius-md); }
  .user-username, .user-date { font-size: 0.85rem; color: var(--text-secondary); }
  .user-status { font-size: 0.85rem; color: var(--text-muted); }
  .user-status.approved { color: var(--status-online); }

  .approve-btn { padding: 0.45rem 1rem; background: #4caf50; color: white; border: none; border-radius: var(--radius-lg); cursor: pointer; font-weight: 600; transition: background 0.2s; }
  .approve-btn:hover { background: #43a047; }

  .invites-table { width: 100%; border-collapse: collapse; }
  .invites-table th, .invites-table td { padding: 0.75rem 1rem; text-align: left; border-bottom: 1px solid var(--border); font-size: 0.9rem; color: var(--text-primary); }
  .invites-table th { background: var(--bg-secondary); font-weight: 600; color: var(--text-secondary); }
  .invites-table tr:hover td { background: var(--bg-tertiary); }
  .invites-table tr.expired td { opacity: 0.6; }
  .invites-table tr.used td { color: var(--text-muted); }
  .link code { font-size: 0.85rem; background: var(--bg-secondary); padding: 0.2rem 0.4rem; border-radius: var(--radius-sm); }
  .link button { margin-left: 0.5rem; padding: 0.25rem 0.5rem; background: var(--accent); color: white; border: none; border-radius: var(--radius-sm); cursor: pointer; font-size: 0.8rem; }
  .delete-btn { padding: 0.25rem 0.6rem; background: #ef4444; color: white; border: none; border-radius: var(--radius-sm); cursor: pointer; font-size: 0.8rem; }
  .delete-btn:disabled { opacity: 0.5; cursor: not-allowed; }

  @media (max-width: 768px) {
    .admin-container { padding: 0.75rem; }
    .admin-tabs { flex-direction: column; }
    .tab { width: 100%; text-align: center; }
    .invites-table { font-size: 0.8rem; }
    .invites-table th, .invites-table td { padding: 0.5rem; }
  }
  .delete-user-btn {
    padding: .35rem .6rem; background: none; border: 1px solid #fecaca;
    border-radius: .4rem; color: #dc2626; cursor: pointer; font-size: .9rem;
    transition: background .15s; flex-shrink: 0;
  }
  .delete-user-btn:hover { background: #fee2e2; }

</style>
