<script lang="ts">
  import Avatar from '$lib/components/Avatar.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { authStore } from '$lib/authStore.svelte.js';
  import { notifyAdmin } from '$lib/notificationStore.svelte';

  let pendingUsers     = $state<any[]>([]);
  let allUsers         = $state<any[]>([]);
  let invites          = $state<any[]>([]);
  let loading          = $state(true);
  let activeTab        = $state<'pending' | 'all' | 'invites'>('pending');
  let generatingInvite = $state(false);
  let inviteLink       = $state<string | null>(null);
  let authChecked      = $state(false);

  // ─── Stats derived ─────────────────────────────────────────
  let totalUsers   = $derived(allUsers.length);
  let pendingCount = $derived(pendingUsers.length);
  let adminCount   = $derived(allUsers.filter(u => u.role === 'admin').length);
  let activeInvites = $derived(invites.filter(i => !i.used && !isExpired(i)).length);

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
      if (response.ok) { await loadUsers(); }
      else alert("Erreur lors de l'approbation");
    } catch (e) { alert('Erreur réseau'); }
  }

  async function rejectUser(userId: string) {
    if (!confirm('Refuser cet utilisateur ? Il sera supprimé définitivement.')) return;
    try {
      const response = await fetch('/api/users/reject', {
        method: 'POST', headers: { 'Content-Type': 'application/json' },
        credentials: 'include', body: JSON.stringify({ user_id: userId }),
      });
      if (response.ok) { await loadUsers(); }
      else alert('Erreur lors du refus');
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
        try { await navigator.clipboard.writeText(inviteLink); }
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
  function getStatusClass(invite: any): string {
    if (invite.used) return 'status-used';
    if (isExpired(invite)) return 'status-expired';
    return 'status-active';
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
      <div class="admin-header-top">
        <h1><Icon name="user" size="24" /> Administration</h1>
        <div class="admin-actions-header">
          <button class="btn-icon" onclick={() => goto('/admin/analytics')} title="Analytics">
            <Icon name="check-circle" size="20" />
          </button>
          <span class="admin-badge">
            {authStore.user?.name || authStore.user?.username}
          </span>
        </div>
      </div>
      <p class="admin-subtitle">Gérez les membres et les invitations de votre espace familial</p>
    </div>

    {#if loading}
      <div class="loading-message">Chargement…</div>
    {:else}
      <!-- Quick stats -->
      <div class="quick-stats">
        <div class="quick-stat" class:alert={pendingCount > 0}>
          <span class="qs-value">{pendingCount}</span>
          <span class="qs-label">En attente</span>
        </div>
        <div class="quick-stat">
          <span class="qs-value">{totalUsers}</span>
          <span class="qs-label">Membres</span>
        </div>
        <div class="quick-stat">
          <span class="qs-value">{adminCount}</span>
          <span class="qs-label">Admins</span>
        </div>
        <div class="quick-stat">
          <span class="qs-value">{activeInvites}</span>
          <span class="qs-label">Invitations</span>
        </div>
      </div>

      <!-- Generate invite -->
      <div class="invite-bar">
        <button class="btn-primary" onclick={generateInvite} disabled={generatingInvite}>
          {generatingInvite ? 'Génération…' : '➕ Nouvelle invitation'}
        </button>
        {#if inviteLink}
          <div class="invite-link-box">
            <code class="invite-link-code">{inviteLink}</code>
            <button class="btn-copy" onclick={async () => {
              try { await navigator.clipboard.writeText(inviteLink!); }
              catch { prompt('Copiez :', inviteLink); }
            }}>Copier</button>
          </div>
        {/if}
      </div>

      <!-- Tabs -->
      <div class="admin-tabs">
        <button class="tab" class:active={activeTab === 'pending'} onclick={() => (activeTab = 'pending')}>
          En attente {#if pendingCount > 0}<span class="tab-badge">{pendingCount}</span>{/if}
        </button>
        <button class="tab" class:active={activeTab === 'all'} onclick={() => (activeTab = 'all')}>
          Membres <span class="tab-count">{totalUsers}</span>
        </button>
        <button class="tab" class:active={activeTab === 'invites'} onclick={() => (activeTab = 'invites')}>
          Invitations <span class="tab-count">{invites.length}</span>
        </button>
      </div>

      <div class="admin-content">
        {#if activeTab === 'pending'}
          {#if pendingUsers.length === 0}
            <div class="empty-state">
              <span class="empty-icon">✓</span>
              <span>Tous les membres sont approuvés</span>
            </div>
          {:else}
            <div class="user-list">
              {#each pendingUsers as user}
                <div class="user-card pending">
                  <Avatar username={user.username} name={user.name} size={40} userId={user.id} style={user.avatar_style} seed={user.avatar_seed} />
                  <div class="user-info">
                    <span class="user-name">{user.name || 'Sans nom'}</span>
                    <span class="user-meta">@{user.username} · Inscrit le {formatDate(user.created_at)}</span>
                  </div>
                  <div class="user-actions">
                    <button class="btn-approve" onclick={() => approveUser(user.id)}>Approuver</button>
                    <button class="btn-reject" onclick={() => rejectUser(user.id)}>Refuser</button>
                  </div>
                </div>
              {/each}
            </div>
          {/if}

        {:else if activeTab === 'all'}
          <div class="user-list">
            {#each allUsers as user}
              <div class="user-card" class:admin-card={user.role === 'admin'}>
                <Avatar username={user.username} name={user.name} size={40} userId={user.id} style={user.avatar_style} seed={user.avatar_seed} />
                <div class="user-info">
                  <span class="user-name">
                    {user.name || 'Sans nom'}
                    {#if user.role === 'admin'}<span class="role-badge">Admin</span>{/if}
                  </span>
                  <span class="user-meta">
                    @{user.username}
                    · {user.approved ? 'Approuvé' : 'En attente'}
                  </span>
                </div>
                {#if user.role !== 'admin'}
                  <button class="btn-delete" onclick={() => deleteUser(user.id)} title="Supprimer">✕</button>
                {/if}
              </div>
            {/each}
          </div>

        {:else if activeTab === 'invites'}
          {#if invites.length === 0}
            <div class="empty-state">
              <span class="empty-icon">🔗</span>
              <span>Aucune invitation créée</span>
            </div>
          {:else}
            <div class="invites-list">
              {#each invites as invite}
                <div class="invite-card" class:expired={isExpired(invite)} class:used={invite.used}>
                  <div class="invite-info">
                    <span class="invite-token"><code>{invite.token.slice(0, 10)}…</code></span>
                    <span class="invite-date">Créée le {formatDate(invite.created_at)}</span>
                    <span class="invite-expires">Expire le {formatDate(invite.expires_at)}</span>
                  </div>
                  <div class="invite-actions">
                    <span class="status-badge {getStatusClass(invite)}">{getStatus(invite)}</span>
                    <button class="btn-copy-sm" onclick={async () => {
                      const link = `${window.location.origin}/invite?token=${invite.token}`;
                      try { await navigator.clipboard.writeText(link); }
                      catch { prompt('Copiez :', link); }
                    }}>Copier</button>
                    {#if !invite.used && !isExpired(invite)}
                      <button class="btn-delete-sm" onclick={() => deleteInvite(invite.id)}>✕</button>
                    {/if}
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        {/if}
      </div>
    {/if}
  {/if}
</div>

<style>
  .admin-container {
    max-width: 800px;
    margin: 0 auto;
    padding: 1rem 1.25rem 2rem;
  }

  .admin-header {
    margin-bottom: 1.5rem;
  }
  .admin-header-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
  }
  .admin-header h1 {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--text-primary, #1e293b);
    margin: 0;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  .admin-subtitle {
    color: var(--text-secondary, #64748b);
    font-size: 0.85rem;
    margin: 0.25rem 0 0;
  }
  .admin-actions-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }
  .btn-icon {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    border: 1px solid var(--border, #e2e8f0);
    background: var(--bg-secondary, #f8fafc);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s;
    color: var(--text-secondary, #64748b);
  }
  .btn-icon:hover {
    background: var(--bg-tertiary, #e2e8f0);
    color: var(--text-primary, #1e293b);
  }
  .admin-badge {
    font-size: 0.8rem;
    font-weight: 600;
    padding: 0.35rem 0.75rem;
    border-radius: 20px;
    background: var(--accent-light, #dcfce7);
    color: var(--accent-dark, #16a34a);
  }

  /* Quick stats */
  .quick-stats {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 0.75rem;
    margin-bottom: 1.25rem;
  }
  .quick-stat {
    background: var(--bg-primary, #fff);
    border: 1px solid var(--border, #e2e8f0);
    border-radius: 12px;
    padding: 0.75rem;
    text-align: center;
    transition: all 0.2s;
  }
  .quick-stat.alert {
    border-color: var(--accent, #4ade80);
    background: var(--accent-light, #f0fdf4);
  }
  .qs-value {
    display: block;
    font-size: 1.5rem;
    font-weight: 800;
    color: var(--text-primary, #1e293b);
    line-height: 1.2;
  }
  .qs-label {
    font-size: 0.7rem;
    color: var(--text-secondary, #64748b);
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  /* Invite bar */
  .invite-bar {
    margin-bottom: 1.25rem;
  }
  .btn-primary {
    padding: 0.6rem 1.25rem;
    background: var(--accent, #4ade80);
    color: #fff;
    border: none;
    border-radius: 10px;
    font-weight: 600;
    font-size: 0.9rem;
    cursor: pointer;
    transition: all 0.2s;
  }
  .btn-primary:hover { opacity: 0.9; transform: translateY(-1px); }
  .btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
  .invite-link-box {
    margin-top: 0.5rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    background: var(--bg-secondary, #f8fafc);
    border-radius: 8px;
    border: 1px solid var(--border, #e2e8f0);
  }
  .invite-link-code {
    flex: 1;
    font-size: 0.75rem;
    color: var(--text-secondary, #64748b);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .btn-copy {
    padding: 0.3rem 0.75rem;
    border-radius: 6px;
    border: 1px solid var(--border, #e2e8f0);
    background: var(--bg-primary, #fff);
    font-size: 0.8rem;
    cursor: pointer;
    color: var(--text-secondary, #64748b);
    transition: all 0.15s;
  }
  .btn-copy:hover { background: var(--bg-tertiary, #e2e8f0); }

  /* Tabs */
  .admin-tabs {
    display: flex;
    gap: 0.25rem;
    margin-bottom: 1rem;
    background: var(--bg-secondary, #f1f5f9);
    border-radius: 10px;
    padding: 4px;
  }
  .tab {
    flex: 1;
    padding: 0.5rem 0.75rem;
    border: none;
    background: transparent;
    border-radius: 8px;
    font-size: 0.85rem;
    font-weight: 500;
    color: var(--text-secondary, #64748b);
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.35rem;
  }
  .tab:hover { color: var(--text-primary, #1e293b); }
  .tab.active {
    background: var(--bg-primary, #fff);
    color: var(--text-primary, #1e293b);
    box-shadow: 0 1px 3px rgba(0,0,0,0.08);
  }
  .tab-badge {
    background: var(--accent, #4ade80);
    color: #fff;
    font-size: 0.7rem;
    font-weight: 700;
    padding: 0.15rem 0.45rem;
    border-radius: 10px;
    min-width: 20px;
    text-align: center;
  }
  .tab-count {
    font-size: 0.75rem;
    color: var(--text-muted, #94a3b8);
  }

  /* User list */
  .user-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .user-card {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1rem;
    background: var(--bg-primary, #fff);
    border: 1px solid var(--border, #e2e8f0);
    border-radius: 12px;
    transition: all 0.15s;
  }
  .user-card:hover { border-color: var(--accent, #4ade80); }
  .user-card.pending {
    border-left: 3px solid var(--accent, #4ade80);
  }
  .user-card.admin-card {
    border-left: 3px solid #60a5fa;
  }
  .user-info {
    flex: 1;
    min-width: 0;
  }
  .user-name {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-weight: 600;
    font-size: 0.95rem;
    color: var(--text-primary, #1e293b);
  }
  .user-meta {
    font-size: 0.8rem;
    color: var(--text-secondary, #64748b);
  }
  .role-badge {
    font-size: 0.65rem;
    font-weight: 600;
    padding: 0.15rem 0.5rem;
    border-radius: 10px;
    background: #dbeafe;
    color: #2563eb;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .btn-approve {
    padding: 0.4rem 1rem;
    background: var(--accent, #4ade80);
    color: #fff;
    border: none;
    border-radius: 8px;
    font-weight: 600;
    font-size: 0.85rem;
    cursor: pointer;
    transition: all 0.2s;
    white-space: nowrap;
  }
  .btn-approve:hover { opacity: 0.9; }

  .user-actions {
    display: flex;
    gap: 0.5rem;
  }

  .btn-reject {
    padding: 0.4rem 1rem;
    background: transparent;
    color: #dc2626;
    border: 1px solid #fecaca;
    border-radius: 8px;
    font-weight: 600;
    font-size: 0.85rem;
    cursor: pointer;
    transition: all 0.2s;
    white-space: nowrap;
  }
  .btn-reject:hover { background: #fef2f2; color: #991b1b; border-color: #fca5a5; }

  .btn-delete {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    border: 1px solid var(--border, #e2e8f0);
    background: transparent;
    color: var(--text-muted, #94a3b8);
    cursor: pointer;
    font-size: 0.85rem;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .btn-delete:hover { background: #fef2f2; color: #dc2626; border-color: #fecaca; }

  /* Invites */
  .invites-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .invite-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    padding: 0.75rem 1rem;
    background: var(--bg-primary, #fff);
    border: 1px solid var(--border, #e2e8f0);
    border-radius: 12px;
    transition: all 0.15s;
  }
  .invite-card.expired { opacity: 0.6; }
  .invite-card.used { opacity: 0.5; }
  .invite-info { flex: 1; min-width: 0; }
  .invite-token {
    display: block;
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--text-primary, #1e293b);
  }
  .invite-date, .invite-expires {
    display: block;
    font-size: 0.75rem;
    color: var(--text-secondary, #64748b);
  }
  .invite-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  .status-badge {
    font-size: 0.7rem;
    font-weight: 600;
    padding: 0.2rem 0.6rem;
    border-radius: 10px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .status-active { background: #dcfce7; color: #16a34a; }
  .status-used { background: #f1f5f9; color: #64748b; }
  .status-expired { background: #fef2f2; color: #dc2626; }

  .btn-copy-sm, .btn-delete-sm {
    padding: 0.3rem 0.6rem;
    border-radius: 6px;
    border: 1px solid var(--border, #e2e8f0);
    background: var(--bg-primary, #fff);
    font-size: 0.75rem;
    cursor: pointer;
    color: var(--text-secondary, #64748b);
    transition: all 0.15s;
  }
  .btn-copy-sm:hover { background: var(--bg-tertiary, #e2e8f0); }
  .btn-delete-sm { color: #dc2626; border-color: #fecaca; }
  .btn-delete-sm:hover { background: #fef2f2; }

  /* Empty state */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    padding: 2.5rem 1rem;
    color: var(--text-secondary, #64748b);
    font-size: 0.95rem;
  }
  .empty-icon {
    font-size: 2rem;
    opacity: 0.5;
  }

  .loading-message {
    text-align: center;
    padding: 2rem;
    color: var(--text-secondary, #64748b);
  }

  .not-authorized {
    text-align: center;
    padding: 3rem 1rem;
  }
  .not-authorized h2 { color: var(--text-primary, #1e293b); }
  .not-authorized button {
    margin-top: 1rem;
    padding: 0.5rem 1.5rem;
    background: var(--accent, #4ade80);
    color: #fff;
    border: none;
    border-radius: 8px;
    cursor: pointer;
  }

  .loading-fullpage {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 50vh;
    color: var(--text-secondary, #64748b);
  }

  @media (max-width: 640px) {
    .quick-stats { grid-template-columns: repeat(2, 1fr); }
    .admin-tabs { flex-direction: column; }
    .invite-card { flex-direction: column; align-items: flex-start; }
  }
</style>
