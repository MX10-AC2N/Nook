<script lang="ts">
  let { value = $bindable(''), placeholder = '', id = '', autocomplete = '', disabled = false, required = false }: {
    value: string;
    placeholder?: string;
    id?: string;
    autocomplete?: string;
    disabled?: boolean;
    required?: boolean;
  } = $props();

  let showPassword = $state(false);
</script>

<div class="password-wrapper">
  <input
    type={showPassword ? 'text' : 'password'}
    {id}
    bind:value
    {placeholder}
    {autocomplete}
    {disabled}
    {required}
  />
  <button
    type="button"
    class="toggle-password"
    onclick={() => showPassword = !showPassword}
    aria-label={showPassword ? 'Masquer le mot de passe' : 'Afficher le mot de passe'}
  >
    {showPassword ? '🙈' : '👁️'}
  </button>
</div>

<style>
  .password-wrapper {
    position: relative;
    display: flex;
    align-items: center;
    width: 100%;
  }
  .password-wrapper input {
    width: 100%;
    padding-right: 2.5rem;
  }
  .toggle-password {
    position: absolute;
    right: 0.5rem;
    background: none;
    border: none;
    cursor: pointer;
    font-size: 1.1rem;
    padding: 0.25rem;
    line-height: 1;
  }
  .toggle-password:hover {
    opacity: 0.7;
  }
</style>
