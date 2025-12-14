<script>
  let polls = [];
  let newPoll = { question: '', option1: '', option2: '' };

  const createPoll = () => {
    if (newPoll.question && newPoll.option1 && newPoll.option2) {
      polls = [...polls, {
        question: newPoll.question,
        options: [newPoll.option1, newPoll.option2],
        votes: [0, 0]
      }];
      newPoll = { question: '', option1: '', option2: '' };
    }
  };

  const vote = (pollIndex, optionIndex) => {
    polls[pollIndex].votes[optionIndex]++;
  };
</script>

<div class="p-4">
  <h1 class="text-2xl font-bold mb-4">Sondages</h1>
  <div class="mb-6">
    <input bind:value={newPoll.question} placeholder="Question" class="w-full p-2 mb-2 border rounded" />
    <input bind:value={newPoll.option1} placeholder="Option 1" class="w-full p-2 mb-2 border rounded" />
    <input bind:value={newPoll.option2} placeholder="Option 2" class="w-full p-2 mb-2 border rounded" />
    <button onclick={createPoll} class="bg-purple-500 text-white p-2 rounded">Créer un sondage</button>
  </div>
  {#each polls as poll, i}
    <div class="mb-4 p-4 border rounded">
      <div class="font-bold mb-2">{poll.question}</div>
      {#each poll.options as option, j}
        <div class="flex items-center mb-1">
          <button onclick={() => vote(i, j)} class="mr-2 bg-gray-200 px-2 py-1 rounded">🗳️</button>
          <span>{option} ({poll.votes[j]} votes)</span>
        </div>
      {/each}
    </div>
  {/each}
</div>