<script>
  import { onMount } from 'svelte';
  import { currentTheme } from '$lib/ui/ThemeStore';
  import { page } from '$app/stores';
  import { isAuthenticated, isLoading, checkAuth } from '$lib/authStore';
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';

  let canvas;
  let ctx;
  let particles = [];
  let animationId;
  let mouse = { x: 0, y: 0 };

  // Configs thèmes
  const themeConfigs = {
    'jardin-secret': {
      count: 60,
      colors: ['#a8e6cf', '#dcedc1', '#ffd3b6', '#ffaaa5'],
      size: { min: 2, max: 7 },
      speed: 0.8,
      direction: 'bottom',
      opacity: { min: 0.3, max: 0.7 },
      glow: false
    },
    'space-hub': {
      count: 80,
      colors: ['#00ffff', '#ffffff', '#a0e7ff', '#80d8ff'],
      size: { min: 1, max: 4 },
      speed: 1.5,
      direction: 'random',
      opacity: { min: 0.6, max: 1 },
      glow: true
    },
    'maison-chaleureuse': {
      count: 45,
      colors: ['#ff6b35', '#f7931e', '#ffd700', '#ff8c42'],
      size: { min: 3, max: 9 },
      opacity: { min: 0.4, max: 0.9 },
      speed: 1.2,
      direction: 'top',
      glow: true
    }
  };

  const createParticles = () => {
    const config = themeConfigs[$currentTheme] || themeConfigs['jardin-secret'];
    const isMobile = window.innerWidth < 768;
    const count = isMobile ? Math.floor(config.count / 2) : config.count;

    particles = [];
    for (let i = 0; i < count; i++) {
      particles.push({
        x: Math.random() * canvas.width,
        y: Math.random() * canvas.height,
        size: Math.random() * (config.size.max - config.size.min) + config.size.min,
        speedX: (Math.random() - 0.5) * config.speed,
        speedY: config.direction === 'top' ? -Math.random() * config.speed - 0.5 :
                config.direction === 'bottom' ? Math.random() * config.speed + 0.5 :
                (Math.random() - 0.5) * config.speed * 2,
        color: config.colors[Math.floor(Math.random() * config.colors.length)],
        opacity: Math.random() * (config.opacity.max - config.opacity.min) + config.opacity.min,
        glow: config.glow
      });
    }
  };

  const animate = () => {
    if (!ctx) return;
    
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    particles.forEach(p => {
      if (p.glow) {
        ctx.shadowBlur = 15;
        ctx.shadowColor = p.color;
      } else {
        ctx.shadowBlur = 0;
      }

      ctx.globalAlpha = p.opacity;
      ctx.fillStyle = p.color;
      ctx.beginPath();
      ctx.arc(p.x, p.y, p.size, 0, Math.PI * 2);
      ctx.fill();

      const dx = mouse.x - p.x;
      const dy = mouse.y - p.y;
      const distance = Math.sqrt(dx * dx + dy * dy);
      if (distance < 150) {
        p.speedX += dx / 10000;
        p.speedY += dy / 10000;
      }

      p.x += p.speedX;
      p.y += p.speedY;

      if (p.x < 0 || p.x > canvas.width) p.speedX *= -1;
      if (p.y < 0 || p.y > canvas.height) p.speedY *= -1;
    });

    animationId = requestAnimationFrame(animate);
  };

  onMount(() => {
    if (!browser) return;
    
    // Initialiser le canvas
    if (canvas) {
      ctx = canvas.getContext('2d');
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;
      createParticles();
      animate();
    }

    // Vérifier l'authentification en arrière-plan sans bloquer
    const initAuth = async () => {
      try {
        await checkAuth();
      } catch (error) {
        console.warn('Auth check failed (might be normal if not logged in):', error);
      }
    };
    
    // Lancer la vérification d'authentification mais ne pas attendre
    initAuth();

    window.addEventListener('mousemove', (e) => {
      mouse.x = e.clientX;
      mouse.y = e.clientY;
    });

    const unsubscribe = currentTheme.subscribe(() => {
      if (canvas) {
        createParticles();
      }
    });

    const resize = () => {
      if (canvas) {
        canvas.width = window.innerWidth;
        canvas.height = window.innerHeight;
        createParticles();
      }
    };
    window.addEventListener('resize', resize);

    return () => {
      if (animationId) {
        cancelAnimationFrame(animationId);
      }
      unsubscribe();
      window.removeEventListener('resize', resize);
      window.removeEventListener('mousemove', () => {});
    };
  });
</script>

<!-- Canvas en ARRIÈRE-PLAN (z-index très bas) -->
<canvas
  bind:this={canvas}
  class="fixed inset-0 pointer-events-none"
  style="z-index: -100;"
></canvas>

<!-- Contenu pages - SANS overlay -->
<slot />

<!-- Bottom navigation mobile - AVEC z-index élevé -->
{#if $isAuthenticated && !$page.url.pathname.startsWith('/admin') && $page.url.pathname !== '/login' && $page.url.pathname !== '/join' && $page.url.pathname !== '/create-password' && $page.url.pathname !== '/change-password'}
<nav class="fixed bottom-0 left-0 right-0 bg-white/20 dark:bg-black/20 backdrop-blur-xl border-t border-white/40 flex justify-around py-3 rounded-t-3xl shadow-2xl md:hidden"
     style="z-index: 1000;">
  <a href="/chat" class="flex flex-col items-center text-[var(--text-primary)] hover:text-[var(--accent)] transition { $page.url.pathname === '/chat' ? 'text-[var(--accent)]' : '' }">
    <span class="text-2xl">💬</span>
    <span class="text-xs mt-1">Chat</span>
  </a>
  <a href="/call" class="flex flex-col items-center text-[var(--text-primary)] hover:text-[var(--accent)] transition { $page.url.pathname === '/call' ? 'text-[var(--accent)]' : '' }">
    <span class="text-2xl">📞</span>
    <span class="text-xs mt-1">Appels</span>
  </a>
  <a href="/calendar" class="flex flex-col items-center text-[var(--text-primary)] hover:text-[var(--accent)] transition { $page.url.pathname === '/calendar' ? 'text-[var(--accent)]' : '' }">
    <span class="text-2xl">📅</span>
    <span class="text-xs mt-1">Calendrier</span>
  </a>
  <a href="/events" class="flex flex-col items-center text-[var(--text-primary)] hover:text-[var(--accent)] transition { $page.url.pathname === '/events' ? 'text-[var(--accent)]' : '' }">
    <span class="text-2xl">🗓️</span>
    <span class="text-xs mt-1">Événements</span>
  </a>
  <a href="/settings" class="flex flex-col items-center text-[var(--text-primary)] hover:text-[var(--accent)] transition { $page.url.pathname === '/settings' ? 'text-[var(--accent)]' : '' }">
    <span class="text-2xl">⚙️</span>
    <span class="text-xs mt-1">Réglages</span>
  </a>
</nav>
{/if}

<!-- Sidebar desktop/tablette - AVEC z-index élevé -->
{#if $isAuthenticated && !$page.url.pathname.startsWith('/admin') && $page.url.pathname !== '/login' && $page.url.pathname !== '/join' && $page.url.pathname !== '/create-password' && $page.url.pathname !== '/change-password'}
<aside class="hidden md:block fixed left-0 top-0 h-screen w-20 bg-white/20 dark:bg-black/20 backdrop-blur-xl border-r border-white/40 p-4 flex flex-col items-center gap-6 shadow-2xl"
       style="z-index: 1000;">
  <a href="/chat" class="text-3xl hover:scale-110 transition { $page.url.pathname === '/chat' ? 'text-[var(--accent)]' : 'text-[var(--text-primary)]' }">💬</a>
  <a href="/call" class="text-3xl hover:scale-110 transition { $page.url.pathname === '/call' ? 'text-[var(--accent)]' : 'text-[var(--text-primary)]' }">📞</a>
  <a href="/calendar" class="text-3xl hover:scale-110 transition { $page.url.pathname === '/calendar' ? 'text-[var(--accent)]' : 'text-[var(--text-primary)]' }">📅</a>
  <a href="/events" class="text-3xl hover:scale-110 transition { $page.url.pathname === '/events' ? 'text-[var(--accent)]' : 'text-[var(--text-primary)]' }">🗓️</a>
  <a href="/settings" class="text-3xl hover:scale-110 transition { $page.url.pathname === '/settings' ? 'text-[var(--accent)]' : 'text-[var(--text-primary)]' }">⚙️</a>
  {#if $isAuthenticated}
    <a href="/admin" class="text-3xl hover:scale-110 transition { $page.url.pathname === '/admin' ? 'text-[var(--accent)]' : 'text-[var(--text-primary)]' }">👑</a>
  {/if}
</aside>
{/if}

<style>
  /* Responsive optimisations */
  @media (max-width: 767px) {
    aside { display: none; }
    nav { 
      padding-bottom: env(safe-area-inset-bottom);
      box-shadow: 0 -4px 20px rgba(0, 0, 0, 0.3);
    }
  }

  @media (min-width: 768px) and (max-width: 1024px) {
    aside { 
      width: 5rem; 
      gap: 1.5rem; 
      padding: 1rem;
    }
    aside a { font-size: 1.8rem; }
  }

  @media (min-width: 1025px) {
    aside { 
      width: 6rem; 
      gap: 2rem; 
      padding: 1.5rem;
    }
    aside a { font-size: 2.2rem; }
  }

  /* Améliorer la visibilité du contenu */
  canvas {
    opacity: 0.3; /* Réduire l'opacité pour mieux voir le contenu */
  }
</style>