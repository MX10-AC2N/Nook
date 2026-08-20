<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import StatCard from '$lib/components/StatCard.svelte';
  import StorageCard from '$lib/components/StorageCard.svelte';
  import { onMount, onDestroy } from 'svelte';
  import { goto } from '$app/navigation';
  import { authStore } from '$lib/authStore.svelte.js';
  import type { Chart as ChartType } from 'chart.js/auto';

  // ===== Types =====
  interface DayCount { day: string; count: number; }
  interface ContentTypeStat { content_type: string; count: number; total_size_bytes: number; }

  interface Overview {
    user_count: number;
    message_count: number;
    conversation_count: number;
    poll_count: number;
    upload_count: number;
    missed_call_count: number;
    active_users_7d: number;
    messages_7d: number;
    calls_7d: number;
    messages_per_day: DayCount[];
    calls_per_day: DayCount[];
  }

  interface Activity {
    range: string;
    points: Array<{
      day: string;
      messages: number;
      calls: number;
      uploads: number;
      active_users: number;
    }>;
  }

  interface Storage {
    db_size_bytes: number;
    uploads_size_bytes: number;
    uploads_count: number;
    gifs_size_bytes: number;
    gifs_count: number;
    total_size_bytes: number;
    uploads_by_type: ContentTypeStat[];
  }

  // ===== State =====
  let overview = $state<Overview | null>(null);
  let activity = $state<Activity | null>(null);
  let storage = $state<Storage | null>(null);
  let activeRange = $state<'7d' | '30d'>('7d');
  let loading = $state(true);
  let error = $state<string | null>(null);

  // Charts
  let doughnutCanvas = $state<HTMLCanvasElement | undefined>(undefined);
  let barCanvas = $state<HTMLCanvasElement | undefined>(undefined);
  let lineCanvas = $state<HTMLCanvasElement | undefined>(undefined);
  let doughnutChart: ChartType | undefined;
  let barChart: ChartType | undefined;
  let lineChart: ChartType | undefined;

  // WebSocket for real-time updates
  let analyticsWs: WebSocket | null = null;
  let reconnectTimeout: ReturnType<typeof setTimeout> | null = null;
  let wsConnected = $state(false);
  let lastUpdate = $state<Date | null>(null);

  // ===== Helpers =====
  function getCSSVar(name: string): string {
    if (typeof window === 'undefined') return '#64748b';
    return getComputedStyle(document.documentElement).getPropertyValue(name).trim() || '#64748b';
  }

  function fmtBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
  }

  function fmtNumber(n: number): string {
    return n.toLocaleString('fr-FR');
  }

  function dayLabel(day: string): string {
    return day.slice(5); // MM-DD
  }

  // ===== Data Loading =====
  async function loadOverview() {
    try {
      const res = await fetch('/api/analytics/overview', { credentials: 'include' });
      if (res.status === 401) { goto('/login'); return; }
      if (res.status === 403) { goto('/chat'); return; }
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      overview = await res.json();
    } catch (e: any) {
      error = `Overview: ${e.message}`;
    }
  }

  async function loadActivity() {
    try {
      const res = await fetch(`/api/analytics/activity?range=${activeRange}`, { credentials: 'include' });
      if (res.status === 401) { goto('/login'); return; }
      if (res.status === 403) { goto('/chat'); return; }
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      activity = await res.json();
    } catch (e: any) {
      error = `Activity: ${e.message}`;
    }
  }

  async function loadStorage() {
    try {
      const res = await fetch('/api/analytics/storage', { credentials: 'include' });
      if (res.status === 401) { goto('/login'); return; }
      if (res.status === 403) { goto('/chat'); return; }
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      storage = await res.json();
    } catch (e: any) {
      error = `Storage: ${e.message}`;
    }
  }

  async function loadAll() {
    loading = true;
    error = null;
    await Promise.all([loadOverview(), loadActivity(), loadStorage()]);
    setTimeout(renderCharts, 0);
    loading = false;
  }

  // ===== Charts =====
  async function renderCharts() {
    if (!overview || !activity) return;
    const { default: Chart } = await import('chart.js/auto');
    (window as any).__Chart = Chart;
    renderDoughnut();
    renderBar();
    renderLine();
  }

  function renderDoughnut() {
    if (!doughnutCanvas || !overview) return;
    doughnutChart?.destroy();
    const Chart = (window as any).__Chart;
    doughnutChart = new Chart(doughnutCanvas, {
      type: 'doughnut',
      data: {
        labels: ['Utilisateurs', 'Messages', 'Conversations', 'Sondages', 'Fichiers', 'Appels manqu\u00e9s'],
        datasets: [{
          data: [overview.user_count, overview.message_count, overview.conversation_count, overview.poll_count, overview.upload_count, overview.missed_call_count],
          backgroundColor: ['#4ade80', '#60a5fa', '#f59e0b', '#a78bfa', '#fb7185', '#f87171'],
          borderWidth: 2,
          borderColor: getCSSVar('--bg-primary') || '#ffffff',
        }],
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        plugins: {
          legend: { position: 'bottom', labels: { color: getCSSVar('--text-secondary') || '#64748b', padding: 14, font: { size: 11 } } },
        },
        cutout: '60%',
      },
    });
  }

  function renderBar() {
    if (!barCanvas || !overview) return;
    barChart?.destroy();
    const Chart = (window as any).__Chart;
    const last7: DayCount[] = [];
    for (let i = 6; i >= 0; i--) {
      const d = new Date();
      d.setDate(d.getDate() - i);
      const key = d.toISOString().split('T')[0];
      const found = overview.messages_per_day.find(r => r.day === key);
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
        maintainAspectRatio: false,
        scales: {
          y: { beginAtZero: true, ticks: { color: getCSSVar('--text-secondary') || '#64748b', stepSize: 1 }, grid: { color: 'rgba(0,0,0,0.06)' } },
          x: { ticks: { color: getCSSVar('--text-secondary') || '#64748b' }, grid: { display: false } },
        },
        plugins: { legend: { display: false } },
      },
    });
  }

  function renderLine() {
    if (!lineCanvas || !activity) return;
    lineChart?.destroy();
    const Chart = (window as any).__Chart;
    const points = activity.points;
    lineChart = new Chart(lineCanvas, {
      type: 'line',
      data: {
        labels: points.map(p => dayLabel(p.day)),
        datasets: [
          { label: 'Messages', data: points.map(p => p.messages), borderColor: '#60a5fa', backgroundColor: 'rgba(96,165,250,0.1)' },
          { label: 'Appels', data: points.map(p => p.calls), borderColor: '#f472b6', backgroundColor: 'rgba(244,114,182,0.1)' },
          { label: 'Uploads', data: points.map(p => p.uploads), borderColor: '#f59e0b', backgroundColor: 'rgba(245,158,11,0.1)' },
          { label: 'Utilisateurs actifs', data: points.map(p => p.active_users), borderColor: '#22d3ee', backgroundColor: 'rgba(34,211,238,0.1)' },
        ],
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        scales: {
          y: { beginAtZero: true, ticks: { color: getCSSVar('--text-secondary') || '#64748b' }, grid: { color: 'rgba(0,0,0,0.06)' } },
          x: { ticks: { color: getCSSVar('--text-secondary') || '#64748b' }, grid: { display: false } },
        },
        plugins: { legend: { labels: { color: getCSSVar('--text-secondary') || '#64748b', font: { size: 11 } } } },
      },
    });
  }

  // ===== WebSocket for Real-time Updates =====
  function connectWebSocket() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/api/analytics/ws`;
    
    analyticsWs = new WebSocket(wsUrl);
    
    analyticsWs.onopen = () => {
      console.log('[Analytics] WebSocket connected');
      wsConnected = true;
      analyticsWs?.send('ping');
    };
    
    analyticsWs.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data);
        handleWsMessage(msg);
      } catch (e) {
        console.error('[Analytics] Failed to parse WS message:', e);
      }
    };
    
    analyticsWs.onclose = () => {
      console.log('[Analytics] WebSocket disconnected, reconnecting in 5s...');
      wsConnected = false;
      analyticsWs = null;
      reconnectTimeout = setTimeout(connectWebSocket, 5000);
    };
    
    analyticsWs.onerror = (error) => {
      console.error('[Analytics] WebSocket error:', error);
    };
  }

  function handleWsMessage(msg: any) {
    lastUpdate = new Date();
    
    switch (msg.type) {
      case 'overview_update':
        if (msg.data) {
          overview = msg.data;
          updateDoughnutChart();
          updateBarChart();
        }
        break;
      case 'activity_update':
        if (msg.data) {
          activity = msg.data;
          updateLineChart();
        }
        break;
      case 'storage_update':
        if (msg.data) {
          storage = msg.data;
        }
        break;
      case 'pong':
        break;
    }
  }

  function updateDoughnutChart() {
    if (!doughnutChart || !overview) return;
    doughnutChart.data.datasets[0].data = [
      overview.user_count,
      overview.message_count,
      overview.conversation_count,
      overview.poll_count,
      overview.upload_count,
      overview.missed_call_count
    ];
    doughnutChart.update('none');
  }

  function updateBarChart() {
    if (!barChart || !overview) return;
    const last7: DayCount[] = [];
    for (let i = 6; i >= 0; i--) {
      const d = new Date();
      d.setDate(d.getDate() - i);
      const key = d.toISOString().split('T')[0];
      const found = overview.messages_per_day.find(r => r.day === key);
      last7.push({ day: key.slice(5), count: found?.count ?? 0 });
    }
    barChart.data.labels = last7.map(r => r.day);
    barChart.data.datasets[0].data = last7.map(r => r.count);
    barChart.update('none');
  }

  function updateLineChart() {
    if (!lineChart || !activity) return;
    const points = activity.points;
    lineChart.data.labels = points.map(p => dayLabel(p.day));
    lineChart.data.datasets[0].data = points.map(p => p.messages);
    lineChart.data.datasets[1].data = points.map(p => p.calls);
    lineChart.data.datasets[2].data = points.map(p => p.uploads);
    lineChart.data.datasets[3].data = points.map(p => p.active_users);
    lineChart.update('none');
  }

  // ===== Lifecycle =====
  onMount(() => {
    loadAll();
    connectWebSocket();
  });

  onDestroy(() => {
    if (reconnectTimeout) clearTimeout(reconnectTimeout);
    analyticsWs?.close();
    doughnutChart?.destroy();
    barChart?.destroy();
    lineChart?.destroy();
  });

  // ===== Range Change =====
  $effect(() => {
    if (activeRange) {
      loadActivity();
      setTimeout(renderCharts, 0);
    }
  });
</script>

<div class="admin-analytics">
  <!-- Header -->
  <header class="page-header">
    <h1><Icon name="chart-bar" class="inline" /> Analytics Dashboard</h1>
    <div class="header-actions">
      <div class="ws-status" title={wsConnected ? 'Connecté (temps réel)' : 'Déconnecté'}>
        <span class={`status-dot ${wsConnected ? 'connected' : 'disconnected'}`}></span>
        <span class="status-text">{wsConnected ? 'Temps réel' : 'Hors ligne'}</span>
        {#if lastUpdate}<span class="last-update">MAJ: {lastUpdate.toLocaleTimeString('fr-FR')}</span>{/if}
      </div>
    </div>
  </header>

  {#if error}
    <div class="error-banner">
      <Icon name="alert-circle" />
      <span>{error}</span>
      <button onclick={loadAll}><Icon name="refresh-cw" class="spin" /></button>
    </div>
  {/if}

  {#if loading}
    <div class="loading-state">
      <div class="spinner"></div>
      <p>Chargement des données...</p>
    </div>
  {:else}
    <!-- Overview Cards -->
    <section class="overview-section">
      <h2><Icon name="grid" class="inline" /> Vue d'ensemble</h2>
      <div class="stats-grid">
        <StatCard label="Utilisateurs" value={fmtNumber(overview?.user_count ?? 0)} icon="users" color="#4ade80" />
        <StatCard label="Messages" value={fmtNumber(overview?.message_count ?? 0)} icon="message-square" color="#60a5fa" />
        <StatCard label="Conversations" value={fmtNumber(overview?.conversation_count ?? 0)} icon="message-circle" color="#f59e0b" />
        <StatCard label="Sondages" value={fmtNumber(overview?.poll_count ?? 0)} icon="pie-chart" color="#a78bfa" />
        <StatCard label="Fichiers" value={fmtNumber(overview?.upload_count ?? 0)} icon="file" color="#fb7185" />
        <StatCard label="Appels manqués" value={fmtNumber(overview?.missed_call_count ?? 0)} icon="phone-off" color="#f87171" />
      </div>
      <div class="stats-grid secondary">
        <StatCard label="Actifs 7j" value={fmtNumber(overview?.active_users_7d ?? 0)} icon="user-check" color="#22d3ee" />
        <StatCard label="Messages 7j" value={fmtNumber(overview?.messages_7d ?? 0)} icon="message-square" color="#22d3ee" />
        <StatCard label="Appels 7j" value={fmtNumber(overview?.calls_7d ?? 0)} icon="phone" color="#f472b6" />
      </div>
    </section>

    <!-- Charts -->
    <section class="charts-section">
      <div class="charts-row">
        <div class="chart-card">
          <h3><Icon name="pie-chart" class="inline" /> Répartition globale</h3>
          <canvas bind:this={doughnutCanvas} aria-label="Répartition globale"></canvas>
        </div>
        <div class="chart-card">
          <h3><Icon name="bar-chart-2" class="inline" /> Messages (7 derniers jours)</h3>
          <canvas bind:this={barCanvas} aria-label="Messages par jour"></canvas>
        </div>
      </div>
      
      <div class="chart-card full-width">
        <div class="chart-header">
          <h3><Icon name="activity" class="inline" /> Activité ({activeRange === '7d' ? '7 jours' : '30 jours'})</h3>
          <div class="range-selector">
            <button 
              class={activeRange === '7d' ? 'active' : ''} 
              onclick={() => activeRange = '7d'}
            >7j</button>
            <button 
              class={activeRange === '30d' ? 'active' : ''} 
              onclick={() => activeRange = '30d'}
            >30j</button>
          </div>
        </div>
        <canvas bind:this={lineCanvas} aria-label="Activité au fil du temps"></canvas>
      </div>
    </section>

    <!-- Storage -->
    <section class="storage-section">
      <h2><Icon name="database" class="inline" /> Stockage</h2>
      <div class="storage-grid">
        <StorageCard label="Base de données" value={fmtBytes(storage?.db_size_bytes ?? 0)} icon="database" />
        <StorageCard label="Uploads" value={fmtBytes(storage?.uploads_size_bytes ?? 0)} icon="upload" sub={fmtNumber(storage?.uploads_count ?? 0) + " fichiers"} />
        <StorageCard label="GIFs" value={fmtBytes(storage?.gifs_size_bytes ?? 0)} icon="image" sub={fmtNumber(storage?.gifs_count ?? 0) + " fichiers"} />
        <StorageCard label="Total" value={fmtBytes(storage?.total_size_bytes ?? 0)} icon="hard-drive" />
      </div>
      {#if storage?.uploads_by_type && storage.uploads_by_type.length > 0}
        <div class="storage-types">
          <h3>Par type de contenu</h3>
          <table>
            <thead>
              <tr><th>Type</th><th>Fichiers</th><th>Taille</th></tr>
            </thead>
            <tbody>
              {#each storage.uploads_by_type as type}
                <tr>
                  <td>{type.content_type}</td>
                  <td>{fmtNumber(type.count)}</td>
                  <td>{fmtBytes(type.total_size_bytes)}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </section>
  {/if}
</div>

<style>
  .admin-analytics {
    padding: 1.5rem;
    max-width: 1400px;
    margin: 0 auto;
  }

  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 2rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid var(--border-color, #e2e8f0);
  }

  .page-header h1 {
    font-size: 1.5rem;
    font-weight: 600;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .ws-status {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    background: var(--bg-secondary, #f8fafc);
    border-radius: 9999px;
    font-size: 0.875rem;
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }

  .status-dot.connected { background: #22c55e; }
  .status-dot.disconnected { background: #ef4444; }

  .last-update {
    color: var(--text-muted, #94a3b8);
    font-size: 0.75rem;
  }

  .error-banner {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 1rem;
    background: #fef2f2;
    border: 1px solid #fecaca;
    border-radius: 0.5rem;
    color: #dc2626;
    margin-bottom: 1.5rem;
  }

  .error-banner button {
    margin-left: auto;
    background: none;
    border: none;
    color: inherit;
    cursor: pointer;
  }

  .spin { animation: spin 1s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }

  .loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 3rem;
    gap: 1rem;
    color: var(--text-muted, #94a3b8);
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 3px solid var(--border-color, #e2e8f0);
    border-top-color: var(--primary, #3b82f6);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  /* Overview Cards */
  .overview-section {
    margin-bottom: 2rem;
  }

  .overview-section h2 {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 1.125rem;
    font-weight: 600;
    margin-bottom: 1rem;
    color: var(--text-primary, #1e293b);
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 1rem;
    margin-bottom: 1rem;
  }

  .stats-grid.secondary {
    grid-template-columns: repeat(3, 1fr);
  }

  .stat-card {
    background: var(--bg-primary, #fff);
    border: 1px solid var(--border-color, #e2e8f0);
    border-radius: 0.75rem;
    padding: 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .stat-card .label {
    font-size: 0.875rem;
    color: var(--text-secondary, #64748b);
    display: flex;
    align-items: center;
    gap: 0.375rem;
  }

  .stat-card .value {
    font-size: 2rem;
    font-weight: 700;
    color: var(--text-primary, #1e293b);
  }

  /* Charts */
  .charts-section {
    margin-bottom: 2rem;
  }

  .charts-row {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
    gap: 1.5rem;
    margin-bottom: 1.5rem;
  }

  .chart-card {
    background: var(--bg-primary, #fff);
    border: 1px solid var(--border-color, #e2e8f0);
    border-radius: 0.75rem;
    padding: 1.5rem;
  }

  .chart-card.full-width {
    grid-column: 1 / -1;
  }

  .chart-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  .chart-card h3 {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary, #1e293b);
  }

  .chart-card canvas {
    max-height: 300px;
  }

  .range-selector {
    display: flex;
    gap: 0.25rem;
    background: var(--bg-secondary, #f1f5f9);
    padding: 0.25rem;
    border-radius: 0.375rem;
  }

  .range-selector button {
    padding: 0.375rem 0.75rem;
    border: none;
    background: transparent;
    border-radius: 0.25rem;
    font-size: 0.875rem;
    color: var(--text-secondary, #64748b);
    cursor: pointer;
    transition: all 0.2s;
  }

  .range-selector button.active {
    background: var(--bg-primary, #fff);
    color: var(--primary, #3b82f6);
    font-weight: 500;
    box-shadow: 0 1
2px rgba(0,0,0,0.05);
  }

  .range-selector button:hover:not(.active) {
    color: var(--text-primary, #1e293b);
  }

  /* Storage */
  .storage-section {
    margin-bottom: 2rem;
  }

  .storage-section h2 {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 1.125rem;
    font-weight: 600;
    margin-bottom: 1rem;
    color: var(--text-primary, #1e293b);
  }

  .storage-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
    margin-bottom: 1.5rem;
  }

  .storage-card {
    background: var(--bg-primary, #fff);
    border: 1px solid var(--border-color, #e2e8f0);
    border-radius: 0.75rem;
    padding: 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .storage-card .label {
    font-size: 0.875rem;
    color: var(--text-secondary, #64748b);
    display: flex;
    align-items: center;
    gap: 0.375rem;
  }

  .storage-card .value {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--text-primary, #1e293b);
  }

  .storage-types h3 {
    font-size: 1rem;
    font-weight: 600;
    margin-bottom: 0.75rem;
    color: var(--text-primary, #1e293b);
  }

  .type-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .type-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem;
    background: var(--bg-primary, #fff);
    border: 1px solid var(--border-color, #e2e8f0);
    border-radius: 0.5rem;
  }

  .type-info {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .type-name {
    font-weight: 500;
    color: var(--text-primary, #1e293b);
  }

  .type-count {
    font-size: 0.875rem;
    color: var(--text-secondary, #64748b);
  }

  .type-size {
    font-weight: 600;
    color: var(--text-primary, #1e293b);
  }
</style>

<!-- Header -->
<div class="admin-analytics">
  <header class="page-header">
    <h1><Icon name="chart-bar" class="inline" /> Analytics Dashboard</h1>
    <div class="header-actions">
      <div class="ws-status" title={wsConnected ? 'Connecté (temps réel)' : 'Déconnecté'}>
        <span class={`status-dot ${wsConnected ? 'connected' : 'disconnected'}`}></span>
        <span class="status-text">{wsConnected ? 'Temps réel' : 'Hors ligne'}</span>
        {#if lastUpdate}<span class="last-update">MAJ: {lastUpdate.toLocaleTimeString('fr-FR')}</span>{/if}
      </div>
    </div>
  </header>

  {#if error}
    <div class="error-banner">
      <Icon name="alert-circle" />
      <span>{error}</span>
      <button onclick={loadAll}><Icon name="refresh-cw" class="spin" /></button>
    </div>
  {/if}

  {#if loading}
    <div class="loading-state">
      <div class="spinner"></div>
      <p>Chargement des analytics...</p>
    </div>
  {:else}
    <!-- Overview Section -->
    <section class="overview-section">
      <h2><Icon name="grid" class="inline" /> Vue d'ensemble</h2>
      <div class="stats-grid">
        <StatCard label="Utilisateurs" value={fmtNumber(overview?.user_count ?? 0)} icon="users" color="#4ade80" />
        <StatCard label="Messages" value={fmtNumber(overview?.message_count ?? 0)} icon="message-square" color="#60a5fa" />
        <StatCard label="Conversations" value={fmtNumber(overview?.conversation_count ?? 0)} icon="message-circle" color="#f59e0b" />
        <StatCard label="Sondages" value={fmtNumber(overview?.poll_count ?? 0)} icon="pie-chart" color="#a78bfa" />
        <StatCard label="Fichiers" value={fmtNumber(overview?.upload_count ?? 0)} icon="file" color="#fb7185" />
        <StatCard label="Appels manqués" value={fmtNumber(overview?.missed_call_count ?? 0)} icon="phone-off" color="#f87171" />
      </div>
      <div class="stats-grid secondary">
        <StatCard label="Actifs 7j" value={fmtNumber(overview?.active_users_7d ?? 0)} icon="user-check" color="#22d3ee" />
        <StatCard label="Messages 7j" value={fmtNumber(overview?.messages_7d ?? 0)} icon="message-square" color="#22d3ee" />
        <StatCard label="Appels 7j" value={fmtNumber(overview?.calls_7d ?? 0)} icon="phone" color="#f472b6" />
      </div>
    </section>

    <!-- Charts Section -->
    <section class="charts-section">
      <h2><Icon name="bar-chart-2" class="inline" /> Visualisations</h2>
      
      <div class="charts-row">
        <div class="chart-card">
          <div class="chart-header">
            <h3><Icon name="pie-chart" class="inline" /> Répartition globale</h3>
          </div>
          <canvas bind:this={doughnutCanvas} aria-label="Répartition globale"></canvas>
        </div>
        <div class="chart-card">
          <div class="chart-header">
            <h3><Icon name="bar-chart-2" class="inline" /> Messages (7 derniers jours)</h3>
          </div>
          <canvas bind:this={barCanvas} aria-label="Messages par jour"></canvas>
        </div>
      </div>

      <div class="chart-card full-width">
        <div class="chart-header">
          <h3><Icon name="trending-up" class="inline" /> Activité sur {activeRange === '7d' ? '7' : '30'} jours</h3>
          <div class="range-selector">
            <button 
              class={activeRange === '7d' ? 'active' : ''} 
              onclick={() => { activeRange = '7d'; loadActivity().then(() => setTimeout(renderCharts, 0)); }}
            >7j</button>
            <button 
              class={activeRange === '30d' ? 'active' : ''} 
              onclick={() => { activeRange = '30d'; loadActivity().then(() => setTimeout(renderCharts, 0)); }}
            >30j</button>
          </div>
        </div>
        <canvas bind:this={lineCanvas} aria-label="Activité au fil du temps"></canvas>
      </div>
    </section>

    <!-- Storage Section -->
    <section class="storage-section">
      <h2><Icon name="database" class="inline" /> Stockage</h2>
      <div class="storage-grid">
        <StorageCard label="Base de données" value={fmtBytes(storage?.db_size_bytes ?? 0)} icon="database" />
        <StorageCard label="Fichiers uploadés" value={fmtBytes(storage?.uploads_size_bytes ?? 0)} icon="upload" sub={fmtNumber(storage?.uploads_count ?? 0) + " fichiers"} />
        <StorageCard label="GIFs" value={fmtBytes(storage?.gifs_size_bytes ?? 0)} icon="image" sub={fmtNumber(storage?.gifs_count ?? 0) + " fichiers"} />
        <StorageCard label="Total" value={fmtBytes(storage?.total_size_bytes ?? 0)} icon="hard-drive" />
      </div>
      {#if storage?.uploads_by_type && storage.uploads_by_type.length > 0}
        <div class="storage-types">
          <h3>Par type de contenu</h3>
          <table>
            <thead>
              <tr><th>Type</th><th>Fichiers</th><th>Taille</th></tr>
            </thead>
            <tbody>
              {#each storage.uploads_by_type as type}
                <tr>
                  <td>{type.content_type}</td>
                  <td>{fmtNumber(type.count)}</td>
                  <td>{fmtBytes(type.total_size_bytes)}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </section>
  {/if}
</div>
