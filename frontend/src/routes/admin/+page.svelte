<script lang="ts">
  import { onMount } from 'svelte';

  // Stores réactifs (Svelte 5)
  let inviteLink = $state('');
  let members = $state<Array<{ id: string; name: string; approved: boolean }>>([]);

  /** Crée un lien d’invitation */
  const invite = async () => {
    const res = await fetch('/api/invite', { method: 'POST' });
    const data = await res.json();
    inviteLink = data.message;
  };

  /** Approuve un membre puis recharge la liste */
  const approve = async (id: string) => {
    await fetch(`/api/members/${id}/approve`, { method: 'PATCH' });
    await loadMembers();   // on attend le rafraîchissement
  };

  /** Charge la liste des membres */
  const loadMembers = async () => {
    const res = await fetch('/api/members');
    const data = await res.json();
    members = data.members;
  };

  onMount(loadMembers);
</script>

<div class="p-4">
  <h1 class="text-2xl font-bold mb-4">Admin — Nook</h1>

  <!-- Bouton d’invitation -->
  <button on:click={invite} class="bg-blue-500 text-white p-2 rounded">
    Inviter un membre
  </button>

  {#if inviteLink}
    <div class="mt-4 p-2 bg-gray-100 rounded">
      <p>Lien d’invitation :</p>
      <input
        type="text"
        bind:value={inviteLink}
        class="w-full mt-1 p-1"
        readonly
      />
    </div>
  {/if}

  <h2 class="text-xl font-bold mt-6">Membres</h2>

  <div class="mt-2">
    {#each members as member}
      <div class="flex justify-between items-center p-2 border-b">
        <span>{member.name}</span>

        <span class={member.approved ? 'text-green-500' : 'text-yellow-500'}>
          {member.approved ? 'Approuvé' : 'En attente'}
        </span>

        {#if !member.approved}
          <button
            on:click={() => approve(member.id)}
            class="bg-green-500 text-white p-1 rounded text-sm"
          >
            Approuver
          </button>
        {/if}
      </div>
    {/each}
  </div>
</div>
