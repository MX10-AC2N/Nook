<script>
  import '$lib/ui/themes/jardin.css';
  import '$lib/ui/themes/space.css';
  import '$lib/ui/themes/maison.css';
  import { onMount } from 'svelte';
  import { currentTheme } from '$lib/ui/ThemeStore';
  import Particles from '@tsparticles/svelte';
  import { loadSlim } from '@tsparticles/slim';

  let particlesInit = async () => {
    await loadSlim(); // Charge la version slim (légère)
  };

  // Réactif : options mises à jour quand le thème change
  $: particlesOptions = $currentTheme === 'jardin-secret'
    ? {
        background: { color: { value: 'transparent' } },
        fpsLimit: 60,
        particles: {
          number: { value: 50, density: { enable: true, area: 800 } },
          color: { value: ['#a8e6cf', '#dcedc1', '#ffd3b6'] }, // Verts doux pastel
          shape: { type: 'circle' },
          opacity: { value: { min: 0.3, max: 0.7 }, animation: { enable: true, speed: 1 } },
          size: { value: { min: 2, max: 6 } },
          move: {
            enable: true,
            speed: 1,
            direction: 'bottom',
            random: true,
            straight: false,
            outModes: { default: 'out' }
          }
        },
        interactivity: { events: { onHover: { enable: true, mode: 'repulse' } } },
        detectRetina: true
      }
    : $currentTheme === 'space-hub'
    ? {
        background: { color: { value: 'transparent' } },
        fpsLimit: 60,
        particles: {
          number: { value: 80, density: { enable: true, area: 800 } },
          color: { value: ['#00ffff', '#ffffff', '#a0e7ff'] }, // Cyan néon + blanc
          shape: { type: 'star' },
          opacity: { value: 0.8 },
          size: { value: { min: 1, max: 4 } },
          links: { enable: true, distance: 150, color: '#00ffff', opacity: 0.2, width: 1 },
          move: {
            enable: true,
            speed: 2,
            direction: 'none',
            random: true,
            straight: false
          }
        },
        interactivity: { events: { onHover: { enable: true, mode: 'bubble' } } },
        detectRetina: true
      }
    : { // maison-chaleureuse
        background: { color: { value: 'transparent' } },
        fpsLimit: 60,
        particles: {
          number: { value: 40, density: { enable: true, area: 800 } },
          color: { value: ['#ff6b35', '#f7931e', '#ffd700'] }, // Orange/jaune chaud
          shape: { type: 'circle' },
          opacity: { value: { min: 0.4, max: 1 }, animation: { enable: true, speed: 2 } },
          size: { value: { min: 3, max: 8 } },
          move: {
            enable: true,
            speed: 1.5,
            direction: 'top',
            random: true,
            straight: false,
            outModes: { default: 'out' }
          }
        },
        interactivity: { events: { onHover: { enable: true, mode: 'attract' } } },
        detectRetina: true
      };

  // Applique la classe thème sur body (comme avant)
  onMount(() => {
    const unsubscribe = currentTheme.subscribe(theme => {
      document.body.className = `theme-${theme}`;
    });
    return unsubscribe;
  });
</script>

<!-- Particules en fond global -->
<Particles
  id="tsparticles"
  init={particlesInit}
  options={particlesOptions}
  class="absolute inset-0 -z-10 pointer-events-none"
/>

<!-- Contenu des pages -->
<slot />