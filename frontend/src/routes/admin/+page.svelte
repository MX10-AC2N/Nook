<script lang="ts">
  let inviteLink = '';

  const invite = async () => {
    const res = await fetch('/api/invite', { method: 'POST' });
    const data = await res.json();
    inviteLink = data.message;
  };
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
</div>