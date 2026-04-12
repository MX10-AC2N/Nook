<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { authStore } from '$lib/authStore.svelte.js';
  // Chart.js loaded dynamically (lazy)
  import type { Chart as ChartType } from 'chart.js/auto';

  // ─── Types ────────────────────────────────────────────────────
  interface DayCount { day: string; count: number; }
  interface Analytics {
    user_count:        number;
    message_count:     number;
    conversation_count: number;
    poll_count:        number;
    upload_count:      number;
    active_users_7d:   number;
    messages_7d:       number;
    messages_per_day:  DayCount[];
  }

  // ─── État ─────────────────────────────────────────────────────
  let analytics = $state<Analytics | null>(null);
  let loading    = $state(true);
  let error      = $state<string | null>(null);

  let doughnutCanvas = $state<HTMLCanvasElement | undefined>(undefined);
  let barCanvas      = $state<HTMLCanvasElement | undefined>(undefined);
  let doughnutChart: ChartType | undefined;
  let barChart: ChartType | undefined;

  // ─── Chargement ───────────────────────────────────────────────
  async function loadAnalytics() {
    loading = true;
    error   = null;
    try {
      const res = await fetch('/api/analytics', { credentials: 'include' });
      if (res.status === 401) { goto('/login'); return; }
      if (res.status === 403) { goto('/chat');  return; }
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      analytics = await res.json();
      // Charts rendus au prochain tick (après mise à jour DOM)
      setTimeout(renderCharts, 0);
    } catch (e: any) {
      error = `Impossible de charger les statistiques : ${e.message}`;
    } finally {
      loading = false;
    }
  }

  // ─── Charts ───────────────────────────────────────────────────
  async function renderCharts() {
    if (!analytics) return;
    // Lazy load Chart.js only when needed
    const { default: Chart } = await import('chart.js/auto');
    // Store for use in render functions
    (window as any).__Chart = Chart;
    renderDoughnut();
    renderBar();
  }

  function renderDoughnut() {
    if (!doughnutCanvas || !analytics) return;
    doughnutChart?.destroy();
    doughnutChart = new ((window as any).__Chart)(doughnutCanvas, {
      type: 'doughnut',
      data: {
        labels: ['Utilisateurs', 'Messages', 'Conversations', 'Sondages', 'Fichiers'],
        datasets: [{
          data: [
            analytics.user_count,
            analytics.message_count,
            analytics.conversation_count,
            analytics.poll_count,
            analytics.upload_count,
          ],
          backgroundColor: ['#4ade80', '#60a5fa', '#f59e0b', '#a78bfa', '#fb7185'],
          borderWidth: 2,
          borderColor: '#ffffff22',
        }],
      },
      options: {
        responsive: true,
        plugins: {
          legend: { position: 'bottom', labels: { color: '#64748b', padding: 16 } },
          tooltip: { enabled: true },
        },
        cutout: '60%',
      },
    });
  }

  function renderBar() {
    if (!barCanvas || !analytics) return;
    barChart?.destroy();

    // Remplir les jours manquants avec 0 pour un affichage continu
    const last7: DayCount[] = [];
    for (let i = 6; i >= 0; i--) {
      const d = new Date();
      d.setDate(d.getDate() - i);
      const key = d.toISOString().split('T')[0];
      const found = analytics.messages_per_day.find(r => r.day === key);
      last7.push({ day: key.slice(5), count: found?.count ?? 0 }); // MM-DD
    }

    barChart = new ((window as any).__Chart)(barCanvas, {
      type: 'bar',
      data: {
        labels: last7.map(r => r.day),
        datasets: [{
          label: 'Messages',
          data: last7.map(r => r.count),
          backgroundColor: '#60a5fa99',
          borderColor: '#60a5fa',
          borderWidth: 1,
          borderRadius: 6,
        }],
      },
      options: {
        responsive: true,
        scales: {
          y: {
            beginAtZero: true,
            ticks: { color: '#64748b', stepSize: 1 },
            grid: { color: '#e2e8f033' },
          },
          x: { ticks: { color: '#64748b' }, grid: { display: false } },
        },
        plugins: { legend: { display: false } },
      },
    });
  }

  onMount(loadAnalytics);
</script>

<svelte:head>
  <title>Analytics Admin — Nook</title>
</svelte:head>

<div class="analytics-page">
  <h1><Icon name="check-circle" size="24" /> Analytics</h1>
  <p class="subtitle">Tableau de bord — {new Date().toLocaleDateString('fr-FR', { dateStyle: 'long' })}</p>

  {#if loading}
    <div class="loading">Chargement des statistiques…</div>

  {:else if error}
    <div class="error-box">{error}</div>

  {:else if analytics}
    <!-- Compteurs globaux -->
    <section class="stats-grid">
      <div class="stat-card">
        <span class="stat-icon">👥</span>
        <span class="stat-value">{analytics.user_count}</span>
        <span class="stat-label">Utilisateurs</span>
      </div>
      <div class="stat-card">
        <span class="stat-icon"><Icon name="chat" size="24" /></span>
        <span class="stat-value">{analytics.message_count}</span>
        <span class="stat-label">Messages</span>
      </div>
      <div class="stat-card">
        <span class="stat-icon">🗂️</span>
        <span class="stat-value">{analytics.conversation_count}</span>
        <span class="stat-label">Conversations</span>
      </div>
      <div class="stat-card">
        <span class="stat-icon"><Icon name="check-circle" size="24" /></span>
        <span class="stat-value">{analytics.poll_count}</span>
        <span class="stat-label">Sondages</span>
      </div>
      <div class="stat-card highlight">
        <span class="stat-icon">🟢</span>
        <span class="stat-value">{analytics.active_users_7d}</span>
        <span class="stat-label">Actifs (7j)</span>
      </div>
      <div class="stat-card highlight">
        <span class="stat-icon">📨</span>
        <span class="stat-value">{analytics.messages_7d}</span>
        <span class="stat-label">Messages (7j)</span>
      </div>
    </section>

    <!-- Charts -->
    <section class="charts-grid">
      <div class="chart-card">
        <h2>Répartition globale</h2>
        <canvas bind:this={doughnutCanvas}></canvas>
      </div>
      <div class="chart-card">
        <h2>Messages — 7 derniers jours</h2>
        <canvas bind:this={barCanvas}></canvas>
      </div>
    </section>

    <div class="refresh-row">
      <button onclick={loadAnalytics} class="refresh-btn">🔄 Actualiser</button>
    </div>
  {/if}
</div>

<style>
  .analytics-page {
    max-width: 900px;
    margin: 0 auto;
    padding: 1.5rem;
  }

  h1 {
    font-size: 1.75rem;
    font-weight: 700;
    margin: 0 0 0.25rem 0;
    color: #1e293b;
  }

  .subtitle {
    color: #64748b;
    margin: 0 0 2rem 0;
    font-size: 0.9rem;
  }

  .loading {
    text-align: center;
    padding: 3rem;
    color: #64748b;
  }

  .error-box {
    background: #fef2f2;
    border: 1px solid #fecaca;
    color: #dc2626;
    padding: 1rem 1.25rem;
    border-radius: 0.75rem;
    margin-bottom: 1rem;
  }

  /* Grille de compteurs */
  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
    gap: 1rem;
    margin-bottom: 2rem;
  }

  .stat-card {
    background: white;
    border-radius: 1rem;
    padding: 1.25rem 1rem;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.4rem;
    box-shadow: 0 1px 4px rgba(0,0,0,0.07);
    border: 1px solid #e2e8f0;
    transition: transform 0.15s;
  }

  .stat-card:hover { transform: translateY(-2px); }

  .stat-card.highlight {
    background: linear-gradient(135deg, #f0fdf4, #dcfce7);
    border-color: #bbf7d0;
  }

  .stat-icon { font-size: 1.5rem; }

  .stat-value {
    font-size: 2rem;
    font-weight: 800;
    color: #1e293b;
    line-height: 1;
  }

  .stat-label {
    font-size: 0.75rem;
    color: #64748b;
    text-align: center;
  }

  /* Grille de charts */
  .charts-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1.5rem;
    margin-bottom: 1.5rem;
  }

  .chart-card {
    background: white;
    border-radius: 1rem;
    padding: 1.5rem;
    box-shadow: 0 1px 4px rgba(0,0,0,0.07);
    border: 1px solid #e2e8f0;
  }

  .chart-card h2 {
    font-size: 0.95rem;
    font-weight: 600;
    color: #475569;
    margin: 0 0 1rem 0;
  }

  .refresh-row {
    display: flex;
    justify-content: flex-end;
  }

  .refresh-btn {
    padding: 0.6rem 1.25rem;
    background: #f1f5f9;
    border: 1px solid #e2e8f0;
    border-radius: 0.5rem;
    color: #475569;
    font-size: 0.9rem;
    cursor: pointer;
    transition: all 0.15s;
  }

  .refresh-btn:hover {
    background: #e2e8f0;
    color: #1e293b;
  }

  @media (max-width: 640px) {
    .charts-grid { grid-template-columns: 1fr; }
    .stats-grid  { grid-template-columns: repeat(3, 1fr); }
    .stat-value  { font-size: 1.5rem; }
  }
</style>
