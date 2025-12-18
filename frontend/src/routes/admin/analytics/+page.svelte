<script>
  import { onMount } from 'svelte';
  import { currentTheme } from '$lib/ui/ThemeStore';
  import Chart from 'chart.js/auto';

  let analytics = $state({ user_count: 0, message_count: 0, active_sessions: 0 });
  let error = $state(null);
  let chart;

  const loadAnalytics = async () => {
    try {
      error = null;
      const res = await fetch('/api/analytics');
      
      if (!res.ok) throw new Error(`Erreur HTTP: ${res.status}`);
      
      analytics = await res.json();
      renderChart();
    } catch (err) {
      error = `Erreur chargement analytics : ${err.message}`;
    }
  };

  const renderChart = () => {
    const ctx = document.getElementById('analyticsChart');
    if (chart) chart.destroy();

    chart = new Chart(ctx, {
      type: 'doughnut',
      data: {
        labels: ['Utilisateurs', 'Messages', 'Sessions actives'],
        datasets: [{
          data: [analytics.user_count, analytics.message_count, analytics.active_sessions],
          backgroundColor: ['#4a90e2', '#ff6b35', '#a8e6cf'],
          borderWidth: 0
        }]
      },
      options: {
        responsive: true,
        plugins: {
          legend: { position: 'bottom' },
          tooltip: { enabled: true }
        }
      }
    });
  };

  onMount(loadAnalytics);
</script>

<svelte:head>
  <title>Analytics Admin — Nook</title>
</svelte:head>

<div class="min-h-screen flex items-center justify-center p-6 relative">
  <div class="max-w-lg w-full backdrop-blur-2xl bg-white/20 dark:bg-black/20 border border-white/30 dark:border-white/20 rounded-3xl shadow-2xl p-10 text-center transition-all duration-1000 animate-fade-in">
    
    <div class="text-7xl mb-6 animate-bounce-slow">
      {#if $currentTheme === 'space-hub'}
        📊
      {:else}
        📈
      {/if}
    </div>

    <h1 class="text-5xl font-extrabold mb-3 bg-gradient-to-r from-[var(--accent)] to-[var(--accent-dark, var(--accent))] bg-clip-text text-transparent">
      Analytics Admin
    </h1>
    
    <p class="text-lg text-[var(--text-secondary)] mb-10">Statistiques de votre espace familial</p>

    {#if error}
      <div class="mb-6 p-4 bg-red-500/20 text-red-600 dark:text-red-400 rounded-2xl border border-red-500/30">
        {error}
      </div>
    {/if}

    <canvas id="analyticsChart" class="w-full h-64"></canvas>

    <div class="mt-8 text-left">
      <p class="mb-2">Utilisateurs : {analytics.user_count}</p>
      <p class="mb-2">Messages : {analytics.message_count}</p>
      <p>Sessions actives : {analytics.active_sessions}</p>
    </div>

  </div>
</div>

<style>
  @keyframes bounce-slow {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(-10px); }
  }
  .animate-bounce-slow { animation: bounce-slow 4s infinite ease-in-out; }

  /* Responsive optimisations */
  @media (max-width: 767px) {
    h1 { font-size: 3rem; }
    p { font-size: 1rem; }
    .p-10 { padding: 1.5rem; }
    .h-64 { height: 12rem; } /* Chart plus petit sur mobile */
  }
</style>