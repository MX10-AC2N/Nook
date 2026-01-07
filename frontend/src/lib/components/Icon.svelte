<script lang="ts">
  export let name: string;
  export let size = '1em';
  export let color = 'currentColor';
  export let class: string = '';
  
  const icons = {
    'logo': '/logo.svg',
    'warning': '/warning.svg',
    'success': '/success.svg',
    'error': '/error.svg',
    'home': '/home.svg',
    'chat': '/chat.svg',
    'calendar': '/calendar.svg',
    'settings': '/settings.svg',
    'user': '/user.svg',
    'plus': '/plus.svg',
    'help': '/help.svg'
  };
</script>

<img 
  src={icons[name] || `/icons/${name}.svg`} 
  alt={name}
  style="width: {size}; height: {size}; color: {color};"
  class={class}
/>