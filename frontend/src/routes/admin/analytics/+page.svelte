<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { authStore } from '$lib/authStore.svelte.js';
  import type { Chart as ChartType } from 'chart.js/auto';

  interface DayCount { day: string; count: number; }
  interface Analytics {
    user_count: number;
    message_count: number;
    conversation_count: number;
    poll_count: number;
    upload_count: number;
    active_users_7d: number;
    messages_7d: number;
    messages_per_day: DayCount[];
  }

  let analytics = $state<Analytics | null>(null);
  let loading    = $state(true);
  let error      = $state<string | null>(null);

  let doughnutCanvas = $state<HTMLCanvasElement | undefined>(undefined);
  let barCanvas      = $state<HTMLCanvasElement | undefined>(undefined);
  let doughnutChart: ChartType | undefined;
  let barChart: ChartType | undefined;

  async function loadAnalytics() {
    loading = true;
    error   = null;
    try {
      const res = await fetch('/api/analytics', { credentials: 'include' });
      if (res.status === 401) { goto('/login'); return; }
      if (res.status === 403) { goto('/chat');  return; }
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      analytics = await res.json();
      setTimeout(renderCharts, 0);
    } catch (e: any) {
      error = `Impossible de charger les statistiques : ${e.message}`;
    } finally {
      loading = false;
    }
  }

  async function renderCharts() {
    if (!analytics) return;
    const { default: Chart } = await import('chart.js/auto');
    (window as any).__Chart = Chart;
    renderDoughnut();
    renderBar();
  }

  function getCSSVar(name: string): string {
    if (typeof window === 'undefined') return '#64748b';
    return getComputedStyle(document.documentElement).getPropertyValue(name).trim() || '#64748b';
  }

  function renderDoughnut() {
    if (!doughnutCanvas || !analytics) return;
    doughnutChart?.destroy();
    const Chart = (window as any).__Chart;
    doughnutChart = new Chart(doughnutCanvas, {
      type: 'doughnut',
      data: {
        labels: ['Utilisateurs', 'Messages', 'Conversations', 'Sondages', 'Fichiers'],
        datasets: [{
          data: [analytics.user_count, analytics.message_count, analytics.conversation_count, analytics.poll_count, analytics.upload_count],
          backgroundColor: ['#4ade80', '#60a5fa', '#f59e0b', '#a78bfa', '#fb7185'],
          borderWidth: 2,
          borderColor: getCSSVar('--bg-primary') || '#ffffff',
        }],
      },
      options: {
        responsive: true,
        plugins: {
          legend: { position: 'bottom', labels: { color: getCSSVar('--text-secondary') || '#64748b', padding: 14, font: { size: 12 } } },
        },
        cutout: '60%',
      },
    });
  }

  function renderBar() {
    if (!barCanvas || !analytics) return;
    barChart?.destroy();
    const Chart = (window as any).__Chart;
    const last7: DayCount[] = [];
    for (let i = 6; i >= 0; i--) {
      const d = new Date();
      d.setDate(d.getDate() - i);
      const key = d.toISOString().split('T')[0];
      const found = analytics.messages_per_day.find(r => r.day === key);
      last7.push({ day: key.slice(5), count: found?.count ?? 0 });
    }
    barChart = new Chart(barCanvas, {
      type: 'bar',
      data: {
        labels: last7.map(r => r.day),
        datasets: [{
          label: 'Messages',
          data: last7.map(r => r.count),
          backgroundColor: 'rgba(96,165,250,0.6)',
          borderColor: '#60a5fa',
          borderWidth: 1,
          borderRadius: 6,
        }],
      },
      options: {
        responsive: true,
        scales: {
          y: { beginAtZero: true, ticks: { color: getCSSVar('--text-secondary') || '#64748b', stepSize: 1 }, grid: { color: 'rgba(0,0,0,0.06)' } },
          x: { ticks: { color: getCSSVar('--text-secondary') || '#64748b' }, grid: { display: false } },
        },
        plugins: { legend: { display: false } },
      },
    });
  }

  onMount(loadAnalytics);
</script>

<svelte:head>
  <title>Analytics — Nook</title>
</svelte:head>

<div class="analytics-page">
  <div class="analytics-header">
    <h1>Analytics</h1>
    <div class="header-actions">
      <button class="btn-back" onclick={() => goto('/admin')}>← Administration</button>
      <button class="btn-refresh" onclick={loadAnalytics}>🔄</button>
    </div>
  </div>
  <p class="subtitle">Tableau de bord — {new Date().toLocaleDateString('fr-FR', { dateStyle: 'long' })}</p>

  {#if loading}
    <div class="loading">Chargement…</div>
  {:else if error}
    <div class="error-box">{error}</div>
  {:else if analytics}
    <!-- Stats cards -->
    <section class="stats-grid">
      <div class="stat-card">
        <span class="stat-icon">👥</span>
        <span class="stat-value">{analytics.user_count}</span>
        <span class="stat-label">Utilisateurs</span>
      </div>
      <div class="stat-card">
        <span class="stat-icon">💬</span>
        <span class="stat-value">{analytics.message_count}</span>
        <span class="stat-label">Messages</span>
      </div>
      <div class="stat-card">
        <span class="stat-icon">🗂️</span>
        <span class="stat-value">{analytics.conversation_count}</span>
        <span class="stat-label">Conversations</span>
      </div>
      <div class="stat-card">
        <span class="stat-icon">📊</span>
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
  {/if}
</div>

<style>
  .analytics-page {
    max-width: 900px;
    margin: 0 auto;
    padding: 1rem 1.25rem 2rem;
  }
  .analytics-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
  }
  h1 {
    font-size: 1.5rem;
    font-weight: 700;
    margin: 0;
    color: var(--text-primary, #1e293b);
  }
  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  .btn-back {
    padding: 0.4rem 0.85rem;
    border-radius: 8px;
    border: 1px solid var(--border, #e2e8f0);
    background: var(--bg-secondary, #f8fafc);
    color: var(--text-secondary, #64748b);
    font-size: 0.8rem;
    cursor: pointer;
    transition: all 0.15s;
  }
  .btn-back:hover { background: var(--bg-tertiary, #e2e8f0); color: var(--text-primary); }
  .btn-refresh {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    border: 1px solid var(--border, #e2e8f0);
    background: var(--bg-secondary, #f8fafc);
    cursor: pointer;
    font-size: 1rem;
    transition: all 0.2s;
  }
  .btn-refresh:hover { background: var(--bg-tertiary, #e2e8f0); }
  .subtitle {
    color: var(--text-secondary, #64748b);
    margin: 0.25rem 0 1.5rem;
    font-size: 0.85rem;
  }
  .loading {
    text-align: center;
    padding: 3rem;
    color: var(--text-secondary, #64748b);
  }
  .error-box {
    background: #fef2f2;
    border: 1px solid #fecaca;
    color: #dc2626;
    padding: 1rem 1.25rem;
    border-radius: 0.75rem;
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 0.75rem;
    margin-bottom: 1.5rem;
  }
  .stat-card {
    background: var(--bg-primary, #fff);
    border: 1px solid var(--border, #e2e8f0);
    border-radius: 12px;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.3rem;
    transition: transform 0.15s;
  }
  .stat-card:hover { transform: translateY(-2px); }
  .stat-card.highlight {
    background: var(--accent-light, #f0fdf4);
    border-color: var(--accent, #4ade80);
  }
  .stat-icon { font-size: 1.3rem; }
  .stat-value {
    font-size: 1.75rem;
    font-weight: 800;
    color: var(--text-primary, #1e293b);
    line-height: 1;
  }
  .stat-label {
    font-size: 0.7rem;
    color: var(--text-secondary, #64748b);
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  .charts-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
  }
  .chart-card {
    background: var(--bg-primary, #fff);
    border: 1px solid var(--border, #e2e8f0);
    border-radius: 12px;
    padding: 1.25rem;
  }
  .chart-card h2 {
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--text-primary, #475569);
    margin: 0 0 1rem;
  }

  @media (max-width: 640px) {
    .charts-grid { grid-template-columns: 1fr; }
    .stats-grid  { grid-template-columns: repeat(2, 1fr); }
    .stat-value  { font-size: 1.5rem; }
  }
</style>
