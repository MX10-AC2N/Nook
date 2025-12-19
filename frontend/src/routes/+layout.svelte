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

<canvas
  bind:this={canvas}
  class="fixed inset-0 -z-10 pointer-events-none"
></canvas>

<!-- Contenu pages - TOUJOURS AFFICHER SANS BLOQUER -->
<slot />

<!-- Bottom navigation mobile - UNIQUEMENT SI AUTHENTIFIÉ -->
{#if $isAuthenticated && !$page.url.pathname.startsWith('/admin') && $page.url.pathname !== '/login' && $page.url.pathname !== '/join' && $page.url.pathname !== '/create-password'}
<nav class="fixed bottom-0 left-0 right-0 bg-white/10 dark:bg-black/10 backdrop-blur-xl border-t border-white/20 flex justify-around py-2 rounded-t-3xl shadow-2xl md:hidden z-50">
  <a href="/chat" class="flex flex-col items-center text-[var(--text-primary)] hover:text-[var(--accent)] transition { $page.url.pathname === '/chat' ? 'text-[var(--accent)]' : '' }">
    <span class="text-2xl">💬</span>
    <span class="text-xs">Chat</span>
  </a>
  <a href="/call" class="flex flex-col items-center text-[var(--text-primary)] hover:text-[var(--accent)] transition { $page.url.pathname === '/call' ? 'text-[var(--accent)]' : '' }">
    <span class="text-2xl">📞</span>
    <span class="text-xs">Appels</span>
  </a>
  <a href="/calendar" class="flex flex-col items-center text-[var(--text-primary)] hover:text-[var(--accent)] transition { $page.url.pathname === '/calendar' ? 'text-[var(--accent)]' : '' }">
    <span class="text-2xl">📅</span>
    <span class="text-xs">Calendrier</span>
  </a>
  <a href="/events" class="flex flex-col items-center text-[var(--text-primary)] hover:text-[var(--accent)] transition { $page.url.pathname === '/events' ? 'text-[var(--accent)]' : '' }">
    <span class="text-2xl">🗓️</span>
    <span class="text-xs">Événements</span>
  </a>
  <a href="/settings" class="flex flex-col items-center text-[var(--text-primary)] hover:text-[var(--accent)] transition { $page.url.pathname === '/settings' ? 'text-[var(--accent)]' : '' }">
    <span class="text-2xl">⚙️</span>
    <span class="text-xs">Réglages</span>
  </a>
</nav>
{/if}

<!-- Sidebar desktop/tablette - UNIQUEMENT SI AUTHENTIFIÉ -->
{#if $isAuthenticated && !$page.url.pathname.startsWith('/admin') && $page.url.pathname !== '/login' && $page.url.pathname !== '/join' && $page.url.pathname !== '/create-password'}
<aside class="hidden md:block fixed left-0 top-0 h-screen w-24 bg-white/10 dark:bg-black/10 backdrop-blur-xl border-r border-white/20 p-4 flex flex-col items-center gap-8 shadow-2xl z-50">
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
    nav { padding-bottom: env(safe-area-inset-bottom); }
  }

  @media (min-width: 768px) and (max-width: 1024px) {
    aside { width: 6rem; gap: 2rem; }
    aside a { font-size: 1.5rem; }
  }

  @media (min-width: 1025px) {
    aside { width: 7rem; gap: 3rem; }
    aside a { font-size: 2rem; }
  }
</style>