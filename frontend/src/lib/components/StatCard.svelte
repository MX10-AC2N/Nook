<script lang="ts">
  import Icon from './Icon.svelte';

  let { label, value, icon, color = '#3b82f6' }: {
    label: string;
    value: string | number;
    icon: string;
    color?: string;
  } = $props();
</script>

<div class="stat-card" style="--stat-color: {color}">
  <div class="label">
    <Icon name={icon} size={16} />
    <span>{label}</span>
  </div>
  <div class="value">{value}</div>
</div>

<style>
  .stat-card {
    background: var(--bg-primary, #fff);
    border: 1px solid var(--border-color, #e2e8f0);
    border-radius: 0.75rem;
    padding: 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .label {
    font-size: 0.875rem;
    color: var(--text-secondary, #64748b);
    display: flex;
    align-items: center;
    gap: 0.375rem;
  }

  .value {
    font-size: 2rem;
    font-weight: 700;
    color: var(--text-primary, #1e293b);
  }
</style>