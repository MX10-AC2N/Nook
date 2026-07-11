<script lang="ts">
  import Icon from './Icon.svelte';

  let { label, value, icon, sub }: {
    label: string;
    value: string | number;
    icon: string;
    sub?: string;
  } = $props();
</script>

<div class="storage-card">
  <div class="label">
    <Icon name={icon} size={16} />
    <span>{label}</span>
  </div>
  <div class="value">{value}</div>
  {#if sub}
    <div class="sub">{sub}</div>
  {/if}
</div>

<style>
  .storage-card {
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
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--text-primary, #1e293b);
  }

  .sub {
    font-size: 0.75rem;
    color: var(--text-muted, #94a3b8);
  }
</style>