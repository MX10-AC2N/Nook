<script lang="ts">
  let events = [
    { title: 'Anniversaire de Léa', date: '2025-12-25', time: '18:00' },
    { title: 'Appel famille', date: '2025-12-20', time: '20:00' }
  ];
  let newEvent = { title: '', date: '', time: '' };

  const addEvent = () => {
    if (newEvent.title && newEvent.date && newEvent.time) {
      events = [...events, { ...newEvent }];
      newEvent = { title: '', date: '', time: '' };
    }
  };
</script>

<div class="p-4">
  <h1 class="text-2xl font-bold mb-4">Rendez-vous</h1>
  <div class="mb-4">
    <input bind:value={newEvent.title} placeholder="Titre" class="w-full p-2 mb-2 border rounded" />
    <input type="date" bind:value={newEvent.date} class="w-full p-2 mb-2 border rounded" />
    <input type="time" bind:value={newEvent.time} class="w-full p-2 mb-4 border rounded" />
    <button on:click={addEvent} class="bg-blue-500 text-white p-2 rounded">Ajouter</button>
  </div>
  <div>
    {#each events as event}
      <div class="p-2 border-b">{event.title} — {event.date} à {event.time}</div>
    {/each}
  </div>
</div>