<script lang="ts">
  export let name: string;
  export let size = '1em';
  export let color = 'currentColor';
  export let class: string = '';
  
  // Utilise maintenant le dossier icons
  const iconPath = `/icons/${name}.svg`;
</script>

<img 
  src={iconPath} 
  alt={name}
  style="width: {size}; height: {size}; color: {color}; fill: {color};"
  class={class}
/>