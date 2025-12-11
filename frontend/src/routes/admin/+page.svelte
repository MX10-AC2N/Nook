<script lang="ts">
  import { onMount } from 'svelte';

  let inviteLink = $state('');
  let members = $state<{ id: string; name: string; approved: boolean }[]>([]);
  let isLoading = $state(false);
  let error = $state<string | null>(null);

  const invite = async () => {
    try {
      isLoading = true;
      error = null;
      const res = await fetch('/api/invite', { method: 'POST' });
      
      if (!res.ok) {
        throw new Error(`Erreur HTTP: ${res.status}`);
      }
      
      const data = await res.json();
      inviteLink = data.message;
    } catch (err) {
      error = `Erreur lors de la création du lien d'invitation: ${err instanceof Error ? err.message : String(err)}`;
      console.error('Erreur invite:', err);
    } finally {
      isLoading = false;
    }
  };

  const approve = async (id: string) => {
    try {
      error = null;
      const res = await fetch(`/api/members/${id}/approve`, { method: 'PATCH' });
      
      if (!res.ok) {
        throw new Error(`Erreur HTTP: ${res.status}`);
      }
      
      loadMembers();
    } catch (err) {
      error = `Erreur lors de l'approbation: ${err instanceof Error ? err.message : String(err)}`;
      console.error('Erreur approve:', err);
    }
  };

  const loadMembers = async () => {
    try {
      isLoading = true;
      error = null;
      const res = await fetch('/api/members');
      
      if (!res.ok) {
        throw new Error(`Erreur HTTP: ${res.status}`);
      }
      
      const data = await res.json();
      members = data.members;
    } catch (err) {
      error = `Erreur lors du chargement des membres: ${err instanceof Error ? err.message : String(err)}`;
      console.error('Erreur loadMembers:', err);
    } finally {
      isLoading = false;
    }
  };

  onMount(loadMembers);
</script>

<div class="p-4">
  <h1 class="text-2xl font-bold mb-4">Admin — Nook</h1>
  
  {#if error}
    <div class="mb-4 p-3 bg-red-100 border border-red-400 text-red-700 rounded">
      {error}
      <button onclick={() => error = null} class="ml-2 text-red-800 font-bold">
        ×
      </button>
    </div>
  {/if}
  
  <button 
    onclick={invite} 
    disabled={isLoading}
    class="bg-blue-500 text-white p-2 rounded disabled:opacity-50 disabled:cursor-not-allowed"
  >
    {isLoading ? 'Création...' : 'Inviter un membre'}
  </button>
  
  {#if inviteLink}
    <div class="mt-4 p-4 bg-gray-100 rounded">
      <p class="font-medium">Lien d'invitation :</p>
      <input type="text" value={inviteLink} class="w-full mt-1 p-2 border rounded" readonly />
      <button 
        onclick={() => navigator.clipboard.writeText(inviteLink)} 
        class="mt-2 text-sm bg-gray-200 hover:bg-gray-300 p-1 rounded"
      >
        Copier le lien
      </button>
    </div>
  {/if}

  <h2 class="text-xl font-bold mt-8 mb-4">Membres</h2>
  
  {#if isLoading && members.length === 0}
    <div class="text-center p-4">Chargement...</div>
  {:else if members.length === 0}
    <div class="text-center p-4 text-gray-500">Aucun membre trouvé</div>
  {:else}
    <div class="mt-2 border rounded overflow-hidden">
      {#each members as member (member.id)}
        <div class="flex justify-between items-center p-3 border-b last:border-b-0 hover:bg-gray-50">
          <div class="flex-1">
            <span class="font-medium">{member.name}</span>
            <span class="ml-2 text-sm text-gray-500">(ID: {member.id})</span>
          </div>
          <span class={`px-2 py-1 rounded text-sm ${member.approved ? 'bg-green-100 text-green-800' : 'bg-yellow-100 text-yellow-800'}`}>
            {member.approved ? '✓ Approuvé' : '⏳ En attente'}
          </span>
          {#if !member.approved}
            <button 
              onclick={() => approve(member.id)} 
              class="ml-4 bg-green-500 hover:bg-green-600 text-white p-2 rounded text-sm transition-colors"
            >
              Approuver
            </button>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>