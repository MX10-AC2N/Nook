<script lang="ts">
  import { onMount } from 'svelte';

  let inviteLink = $state('');
  let members = $state<{ id: string; name: string; approved: boolean }[]>([]);

  const invite = async () => {
    const res = await fetch('/api/invite', { method: 'POST' });
    const data = await res.json();
    inviteLink = data.message;
  };

  const approve = async (id: string) => {
    await fetch(`/api/members/${id}/approve`, { method: 'PATCH' });
    loadMembers();
  };

  const loadMembers = async () => {
    const res = await fetch('/api/members');
    const data = await res.json();
    members = data.members;
  };

  const handleApprove = (id: string) => {
    approve(id);
  };

  // Fonction intermédiaire qui encapsule l'id
  const createApproveHandler = (id: string) => () => handleApprove(id);

  onMount(loadMembers);
</script>

<div class="p-4">
  <h1 class="text-2xl font-bold mb-4">Admin — Nook</h1>
  <button on:click={invite} class="bg-blue-500 text-white p-2 rounded">
    Inviter un membre
  </button>
  {#if inviteLink}
    <div class="mt-4 p-2 bg-gray-100 rounded">
      <p>Lien d’invitation :</p>
      <input type="text" value={inviteLink} class="w-full mt-1 p-1" readonly />
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
        {!member.approved && (
          <button on:click={createApproveHandler(member.id)} class="bg-green-500 text-white p-1 rounded text-sm">
            Approuver
          </button>
        )}
      </div>
    {/each}
  </div>
</div>