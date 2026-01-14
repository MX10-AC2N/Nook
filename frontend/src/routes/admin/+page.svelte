<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { authStore, isAdmin } from '$lib/authStore';

  // -----------------------------------------------------------------
  // 1️⃣ États locaux (Svelte 5)
  // -----------------------------------------------------------------
  let pendingUsers = $state<any[]>([]);
  let allUsers = $state<any[]>([]);
  let invites = $state<any[]>([]);
  let loading = $state(true);
  let activeTab = $state<'pending' | 'all' | 'invites'>('pending');
  let generatingInvite = $state(false);
  let inviteLink = $state<string | null>(null);
  let authChecked = $state(false);

  // -----------------------------------------------------------------
  // 2️⃣ Vérification d'authentification + droits admin
  // -----------------------------------------------------------------
  onMount(async () => {
    const ok = await checkAuthAndRedirect();
    if (ok) {
      authChecked = true;
      await loadAdminData();
    }
  });

  /**
   * Vérifie que l'utilisateur est bien connecté **et** possède le rôle admin.
   * Met à jour le store `authStore` et effectue les redirections nécessaires.
   *
   * @returns {Promise<boolean>} `true` si l'utilisateur est admin, sinon redirige.
   */
  async function checkAuthAndRedirect(): Promise<boolean> {
    try {
      const response = await fetch('/api/auth/me', {
        credentials: 'include',
      });

      if (!response.ok) throw new Error('Non‑authentifié');

      const data = await response.json();

      if (data.authenticated && data.user) {
        // Met à jour le store (comme dans la page login)
        authStore.setAuthenticated(data.user, data.user.role === 'admin');

        if (data.user.role !== 'admin') {
          // Pas admin → redirection vers le chat
          goto('/chat');
          return false;
        }

        // Admin confirmé
        return true;
      }

      // Pas authentifié
      goto('/login');
      return false;
    } catch (e) {
      console.error('Erreur auth admin :', e);
      goto('/login');
      return false;
    }
  }

  // -----------------------------------------------------------------
  // 3️⃣ Chargement des données d'administration
  // -----------------------------------------------------------------
  async function loadAdminData() {
    try {
      await Promise.all([loadUsers(), loadInvites()]);
    } catch (e) {
      console.error('Erreur chargement admin :', e);
    } finally {
      loading = false;
    }
  }

  async function loadUsers() {
    try {
      const [pendingRes, allRes] = await Promise.all([
        fetch('/api/pending-users-json', { credentials: 'include' }),
        fetch('/api/all-users-json', { credentials: 'include' }),
      ]);

      if (pendingRes.ok) {
        const data = await pendingRes.json();
        pendingUsers = data.users || [];
      }

      if (allRes.ok) {
        const data = await allRes.json();
        allUsers = data.users || [];
      }
    } catch (e) {
      console.error('Erreur chargement utilisateurs :', e);
    }
  }

  async function loadInvites() {
    try {
      const res = await fetch('/api/list-invites', { credentials: 'include' });
      if (res.ok) {
        const data = await res.json();
        invites = data.invites || [];
      }
    } catch (e) {
      console.error('Erreur chargement invitations :', e);
    }
  }

  // -----------------------------------------------------------------
  // 4️⃣ Actions (approbation, génération d'invitation, suppression)
  // -----------------------------------------------------------------
  async function approveUser(userId: string) {
    try {
      const response = await fetch('/api/approve', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ user_id: userId }),
      });

      if (response.ok) {
        await loadUsers();
      } else {
        alert('Erreur lors de l'approbation');
      }
    } catch (e) {
      console.error('Erreur approbation :', e);
      alert('Erreur réseau');
    }
  }

  async function generateInvite() {
    generatingInvite = true;
    inviteLink = null;
    try {
      const response = await fetch('/api/generate-invite', {
        method: 'POST',
        credentials: 'include',
      });

      if (response.ok) {
        const data = await response.json();
        inviteLink = data.invite_link;

        // Copie dans le presse‑papier (protégé côté SSR)
        if (typeof window !== 'undefined') {
          await navigator.clipboard.writeText(inviteLink);
          alert('Lien copié dans le presse‑papiers !');
        }

        await loadInvites();
      } else {
        alert('Erreur lors de la génération');
      }
    } catch (e) {
      console.error('Erreur génération invite :', e);
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
        body: JSON.stringify({ id }),
      });
      await loadInvites();
    } catch (e) {
      alert('Erreur suppression');
    }
  }

  // -----------------------------------------------------------------
  // 5️⃣ Helpers d'affichage
  // -----------------------------------------------------------------
  function formatDate(ts: number | string): string {
    const date = typeof ts === 'number' ? new Date(ts * 1000) : new Date(ts);
    return date.toLocaleDateString('fr-FR', {
      day: '2-digit',
      month: '2-digit',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  }

  function isExpired(invite: any): boolean {
    return Date.now() / 1000 > invite.expires_at;
  }

  function getStatus(invite: any): string {
    if (invite.used) return 'Utilisée';
    if (isExpired(invite)) return 'Expirée';
    return 'Valide';
  }
</script>

<svelte:head>
  <title>Administration - Nook</title>
</svelte:head>

<div class="admin-container">
  {#if !authChecked}
    <!-- Vérification en cours -->
    <div class="loading-fullpage">
      <div class="spinner-large"></div>
      <p>Vérification des permissions administrateur…</p>
    </div>
  {:else if !$isAdmin}
    <!-- Utilisateur authentifié mais pas admin -->
    <div class="not-authorized">
      <h2>Accès non autorisé</h2>
      <p>Vous n'avez pas les permissions nécessaires pour accéder à cette page.</p>
      <button onclick={() => goto('/chat')}>Aller au chat</button>
    </div>
  {:else}
    <!-- Interface admin -->
    <div class="admin-header">
      <h1>👑 Administration</h1>
      <div class="auth-status">
        <span class="admin-badge">Connecté en tant qu'admin</span>
      </div>
      <p>Gérez les membres et les invitations de votre espace familial</p>
    </div>

    {#if loading}
      <div class="loading-message">Chargement des données d'administration…</div>
    {:else}
      <!-- Actions globales -->
      <div class="admin-actions">
        <button class="invite-btn" onclick={generateInvite} disabled={generatingInvite}>
          {generatingInvite ? 'Génération…' : '➕ Générer un lien d'invitation'}
        </button>

        {#if inviteLink}
          <p class="invite-link">
            Dernier lien généré : <code>{inviteLink}</code>
          </p>
        {/if}
      </div>

      <!-- Onglets -->
      <div class="admin-tabs">
        <button class="tab" class:active={activeTab === 'pending'} onclick={() => (activeTab = 'pending')}>
          En attente ({pendingUsers.length})
        </button>
        <button class="tab" class:active={activeTab === 'all'} onclick={() => (activeTab = 'all')}>
          Membres ({allUsers.length})
        </button>
        <button class="tab" class:active={activeTab === 'invites'} onclick={() => (activeTab = 'invites')}>
          Invitations ({invites.length})
        </button>
      </div>

      <!-- Contenu des onglets -->
      <div class="admin-content">
        {#if activeTab === 'pending'}
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
                  <button class="approve-btn" onclick={() => approveUser(user.id)}>✅ Approuver</button>
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
                    {#if user.role === 'admin'}
                      <span class="admin-badge">Admin</span>
                    {/if}
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
                  <tr class:expired={isExpired(invite)} class:used={invite.used}>
                    <td>{formatDate(invite.created_at)}</td>
                    <td>{formatDate(invite.expires_at)}</td>
                    <td class="status">{getStatus(invite)}</td>
                    <td class="link">
                      <code>{invite.token.slice(0, 12)}…</code>
                      <button
                        onclick={() => {
                          if (typeof window !== 'undefined')
                            navigator.clipboard.writeText(
                              `${window.location.origin}/join?token=${invite.token}`
                            );
                        }}
                      >
                        Copier
                      </button>
                    </td>
                    <td>
                      <button
                        class="delete-btn"
                        onclick={() => deleteInvite(invite.id)}
                        disabled={invite.used || isExpired(invite)}
                      >
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
    {/if}
  {/if}
</div>

<style>
  /* -----------------------------------------------------------------
     GLOBAL LAYOUT
     ----------------------------------------------------------------- */
  .loading-fullpage {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 60vh;
    color: #666;
  }

  .spinner-large {
    width: 40px;
    height: 40px;
    border: 4px solid rgba(45, 90, 39, 0.1);
    border-top-color: #2d5a27;
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin-bottom: 1rem;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .admin-container {
    max-width: 900px;
    margin: 0 auto;
    padding: 1rem;
  }

  .admin-header {
    text-align: center;
    margin-bottom: 2rem;
  }

  .admin-header h1 {
    font-size: 1.75rem;
    color: #2d5a27;
  }

  .admin-header p {
    color: #666;
  }

  .auth-status {
    margin: 0.5rem 0;
    font-size: 0.9rem;
  }

  .admin-badge {
    padding: 0.25rem 0.75rem;
    border-radius: 1rem;
    font-size: 0.8rem;
    font-weight: 600;
    background: #d1fae5;
    color: #065f46;
  }

  .admin-actions {
    text-align: center;
    margin-bottom: 1.5rem;
  }

  .invite-btn {
    padding: 0.75rem 1.5rem;
    background: #2d5a27;
    color: white;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    transition: background 0.2s;
  }

  .invite-btn:hover:not(:disabled) {
    background: #3d7a37;
  }

  .invite-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .invite-link {
    margin-top: 0.8rem;
    word-break: break-all;
  }

  .invite-link code {
    background: #f0f0f0;
    padding: 0.3rem 0.6rem;
    border-radius: 4px;
  }

  .admin-tabs {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1.5rem;
    justify-content: center;
    flex-wrap: wrap;
  }

  .tab {
    padding: 0.75rem 1.25rem;
    background: #f1f5f9;
    border: none;
    cursor: pointer;
    color: #666;
    border-radius: 8px 8px 0 0;
    transition: all 0.2s;
  }

  .tab:hover {
    background: #e2e8f0;
  }

  .tab.active {
    background: #2d5a27;
    color: white;
  }

  .admin-content {
    background: white;
    border-radius: 12px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    overflow: hidden;
  }

  .loading-message,
  .empty-state {
    text-align: center;
    padding: 3rem;
    color: #888;
  }

  .not-authorized {
    text-align: center;
    padding: 3rem;
    background: #fff;
    border-radius: 1rem;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  }

  .not-authorized h2 {
    color: #dc2626;
    margin-bottom: 1rem;
  }

  .not-authorized button {
    margin-top: 1rem;
    padding: 0.75rem 1.5rem;
    background: #2d5a27;
    color: white;
    border: none;
    border-radius: 0.5rem;
    cursor: pointer;
    transition: background 0.2s;
  }

  .not-authorized button:hover {
    background: #3d7a37;
  }

  /* -----------------------------------------------------------------
     USER LISTS (pending / all)
     ----------------------------------------------------------------- */
  .user-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    padding: 1rem;
  }

  .user-card {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem;
    border-radius: 0.75rem;
    background: #f8fafc;
    border: 1px solid #e2e8f0;
  }

  .user-card.pending {
    background: #fff8e1;
    border-color: #fde68a;
  }

  .user-card.admin {
    background: #e3f2fd;
    border-color: #90caf9;
  }

  .user-info {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .user-name {
    font-weight: 500;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .user-name .admin-badge {
    font-size: 0.7rem;
    padding: 0.2rem 0.5rem;
    background: #2196f3;
    color: white;
    border-radius: 4px;
  }

  .user-username,
  .user-date,
  .user-status {
    font-size: 0.85rem;
    color: #64748b;
  }

  .user-status.approved {
    color: #4caf50;
  }

  .approve-btn {
    padding: 0.5rem 1rem;
    background: #4caf50;
    color: white;
    border: none;
    border-radius: 0.5rem;
    cursor: pointer;
    transition: background 0.2s;
  }

  .approve-btn:hover {
    background: #43a047;
  }

  /* -----------------------------------------------------------------
     INVITES TABLE
     ----------------------------------------------------------------- */
  .invites-table {
    width: 100%;
    border-collapse: collapse;
    margin: 0;
  }

  .invites-table th,
  .invites-table td {
    padding: 0.75rem 1rem;
    text-align: left;
    border-bottom: 1px solid #e2e8f0;
    font-size: 0.9rem;
  }

  .invites-table th {
    background: #f1f5f9;
    font-weight: 600;
    color: #374151;
  }

  .invites-table tr:hover td {
    background: #f9fafb;
  }

  .invites-table tr.expired td {
    opacity: 0.6;
  }

  .invites-table tr.used td {
    color: #9ca3af;
  }

  .status {
    font-weight: 500;
  }

  .link code {
    font-size: 0.85rem;
    background: #f0f0f0;
    padding: 0.2rem 0.4rem;
    border-radius: 4px;
  }

  .link button {
    margin-left: 0.5rem;
    padding: 0.25rem 0.5rem;
    background: #2d5a27;
    color: white;
    border: none;
    border-radius: 0.25rem;
    cursor: pointer;
    font-size: 0.8rem;
    transition: background 0.2s;
  }

  .link button:hover {
    background: #3d7a37;
  }

  .delete-btn {
    padding: 0.25rem 0.6rem;
    background: #ef4444;
    color: white;
    border: none;
    border-radius: 0.25rem;
    cursor: pointer;
    font-size: 0.8rem;
    transition: background 0.2s;
  }

  .delete-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .delete-btn:hover:not(:disabled) {
    background: #dc2626;
  }

  /* -----------------------------------------------------------------
     RESPONSIVE
     ----------------------------------------------------------------- */
  @media (max-width: 768px) {
    .admin-container {
      padding: 0.75rem;
    }

    .admin-tabs {
      flex-direction: column;
    }

    .tab {
      width: 100%;
      text-align: center;
    }

    .invites-table {
      font-size: 0.8rem;
    }

    .invites-table th,
    .invites-table td {
      padding: 0.5rem;
    }
  }
</style>