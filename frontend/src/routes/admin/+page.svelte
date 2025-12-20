<script>
  import { onMount } from 'svelte';
  
  let members = [];
  let invitations = [];
  let loading = $state(true);
  
  // Nouveau membre
  let newMember = $state({
    name: '',
    username: '',
    temporaryPassword: 'changeme123'
  });
  let createMemberResult = $state({ success: false, message: '' });
  
  onMount(async () => {
    await loadMembers();
  });
  
  async function loadMembers() {
    try {
      const res = await fetch('/api/admin/members');
      if (res.ok) {
        const data = await res.json();
        members = data.members;
      }
    } catch (error) {
      console.error('Erreur chargement membres:', error);
    }
    loading = false;
  }
  
  async function createInvite() {
    const res = await fetch('/api/admin/invite', { method: 'POST' });
    if (res.ok) {
      const data = await res.json();
      invitations.push(data.message);
    }
  }
  
  async function approveMember(id) {
    const res = await fetch(`/api/admin/members/${id}/approve`, {
      method: 'PATCH'
    });
    if (res.ok) {
      await loadMembers();
    }
  }
  
  async function createNewMember() {
    if (!newMember.name.trim() || !newMember.username.trim()) {
      createMemberResult = { success: false, message: 'Veuillez remplir tous les champs' };
      return;
    }
    
    if (newMember.username.length < 3) {
      createMemberResult = { success: false, message: 'Le nom d\'utilisateur doit avoir au moins 3 caractères' };
      return;
    }
    
    try {
      const res = await fetch('/api/admin/create-member', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(newMember),
        credentials: 'include'
      });
      
      const data = await res.json();
      createMemberResult = data;
      
      if (data.success) {
        // Réinitialiser le formulaire
        newMember = { name: '', username: '', temporaryPassword: 'changeme123' };
        // Recharger la liste
        await loadMembers();
      }
    } catch (error) {
      createMemberResult = { success: false, message: 'Erreur réseau' };
    }
  }
</script>

<svelte:head>
  <title>Administration — Nook</title>
</svelte:head>

<div class="min-h-screen p-6">
  <div class="max-w-6xl mx-auto">
    <h1 class="text-3xl font-bold mb-8 text-[var(--text-primary)]">Administration Nook</h1>
    
    <!-- Section création de membre -->
    <div class="bg-white/10 dark:bg-black/10 backdrop-blur-xl rounded-2xl p-6 mb-8">
      <h2 class="text-2xl font-bold mb-4 text-[var(--text-primary)]">Créer un nouveau membre</h2>
      
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
        <div>
          <label for="member-name" class="block text-sm font-medium mb-2 text-[var(--text-primary)]">Nom complet</label>
          <input
            id="member-name"
            type="text"
            bind:value={newMember.name}
            placeholder="Ex: Jean Dupont"
            class="w-full p-3 rounded-xl border border-white/40 bg-white/30 dark:bg-black/30 text-[var(--text-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--accent)]"
          />
        </div>
        
        <div>
          <label for="member-username" class="block text-sm font-medium mb-2 text-[var(--text-primary)]">Nom d'utilisateur</label>
          <input
            id="member-username"
            type="text"
            bind:value={newMember.username}
            placeholder="Ex: jean.dupont"
            class="w-full p-3 rounded-xl border border-white/40 bg-white/30 dark:bg-black/30 text-[var(--text-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--accent)]"
          />
        </div>
        
        <div class="md:col-span-2">
          <label for="temporary-password" class="block text-sm font-medium mb-2 text-[var(--text-primary)]">Mot de passe temporaire</label>
          <input
            id="temporary-password"
            type="text"
            bind:value={newMember.temporaryPassword}
            class="w-full p-3 rounded-xl border border-white/40 bg-white/30 dark:bg-black/30 text-[var(--text-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--accent)]"
          />
          <p class="text-xs text-[var(--text-secondary)] mt-1">
            Le membre devra changer ce mot de passe lors de sa première connexion
          </p>
        </div>
      </div>
      
      <button
        onclick={createNewMember}
        class="px-6 py-3 bg-[var(--accent)] text-white font-semibold rounded-xl hover:opacity-90 transition"
      >
        Créer le membre
      </button>
      
      {#if createMemberResult.message}
        <div class={`mt-4 p-3 rounded-xl ${createMemberResult.success ? 'bg-green-500/20 text-green-400' : 'bg-red-500/20 text-red-400'}`}>
          {createMemberResult.message}
        </div>
      {/if}
    </div>
    
    <!-- Liste des membres existants -->
    <div class="bg-white/10 dark:bg-black/10 backdrop-blur-xl rounded-2xl p-6">
      <div class="flex justify-between items-center mb-4">
        <h2 class="text-2xl font-bold text-[var(--text-primary)]">Membres ({members.length})</h2>
        <button
          onclick={createInvite}
          class="px-4 py-2 bg-blue-500/80 text-white rounded-xl hover:opacity-90 transition"
        >
          + Créer un lien d'invitation
        </button>
      </div>
      
      {#if invitations.length > 0}
        <div class="mb-6 p-4 bg-blue-500/10 rounded-xl">
          <h3 class="font-bold mb-2 text-blue-400">Liens d'invitation générés :</h3>
          {#each invitations as invite}
            <div class="mb-2 p-2 bg-white/5 rounded">
              <code class="text-sm break-all">{invite}</code>
            </div>
          {/each}
        </div>
      {/if}
      
      {#if loading}
        <div class="text-center py-8">
          <div class="text-4xl animate-spin mb-2">🌀</div>
          <p class="text-[var(--text-secondary)]">Chargement...</p>
        </div>
      {:else if members.length === 0}
        <div class="text-center py-8 text-[var(--text-secondary)]">
          Aucun membre pour l'instant
        </div>
      {:else}
        <div class="overflow-x-auto">
          <table class="w-full">
            <thead>
              <tr class="border-b border-white/20">
                <th class="text-left p-3 text-[var(--text-primary)]">Nom</th>
                <th class="text-left p-3 text-[var(--text-primary)]">Username</th>
                <th class="text-left p-3 text-[var(--text-primary)]">ID</th>
                <th class="text-left p-3 text-[var(--text-primary)]">Statut</th>
                <th class="text-left p-3 text-[var(--text-primary)]">Actions</th>
              </tr>
            </thead>
            <tbody>
              {#each members as member (member.id)}
                <tr class="border-b border-white/10 hover:bg-white/5">
                  <td class="p-3 text-[var(--text-primary)]">{member.name}</td>
                  <td class="p-3 text-[var(--text-primary)]">{member.username || 'N/A'}</td>
                  <td class="p-3 text-[var(--text-secondary)] text-sm font-mono">{member.id.substring(0, 8)}...</td>
                  <td class="p-3">
                    {#if member.approved}
                      <span class="px-3 py-1 bg-green-500/20 text-green-400 rounded-full text-sm">Approuvé</span>
                    {:else}
                      <span class="px-3 py-1 bg-yellow-500/20 text-yellow-400 rounded-full text-sm">En attente</span>
                    {/if}
                  </td>
                  <td class="p-3">
                    {#if !member.approved}
                      <button
                        onclick={() => approveMember(member.id)}
                        class="px-4 py-2 bg-[var(--accent)] text-white rounded-lg hover:opacity-90 transition text-sm"
                      >
                        Approuver
                      </button>
                    {:else}
                      <span class="text-[var(--text-secondary)] text-sm">✓ Approuvé</span>
                    {/if}
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </div>
  </div>
</div>