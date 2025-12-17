<script>
  import { onMount } from 'svelte';
  import { currentTheme } from '$lib/ui/ThemeStore';

  let canvas;
  let ctx;
  let particles = [];
  let animationId;
  let mouse = { x: 0, y: 0 };

  // Configurations précises par thème
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
      speed: 1.2,
      direction: 'top',
      opacity: { min: 0.4, max: 0.9 },
      glow: true
    }
  };

  const createParticles = () => {
    const config = themeConfigs[$currentTheme] || themeConfigs['jardin-secret'];
    const count = window.innerWidth < 768 ? Math.floor(config.count / 2) : config.count;

    particles = [];
    for (let i = 0; i < count; i++) {
      particles.push({
        x: Math.random() * canvas.width,
        y: Math.random() * canvas.height,
        size: Math.random() * (config.size.max - config.size.min) + config.size.min,
        speedX: (Math.random() - 0.5) * config.speed * 2,
        speedY: config.direction === 'top' ? -config.speed * 1.5 :
                config.direction === 'bottom' ? config.speed * 1.5 :
                (Math.random() - 0.5) * config.speed * 2,
        color: config.colors[Math.floor(Math.random() * config.colors.length)],
        opacity: Math.random() * (config.opacity.max - config.opacity.min) + config.opacity.min,
        glow: config.glow
      });
    }
  };

  const animate = () => {
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    particles.forEach(p => {
      // Glow subtil
      if (p.glow) {
        ctx.shadowBlur = 20;
        ctx.shadowColor = p.color;
      } else {
        ctx.shadowBlur = 0;
      }

      ctx.globalAlpha = p.opacity;
      ctx.fillStyle = p.color;
      ctx.beginPath();
      ctx.arc(p.x, p.y, p.size, 0, Math.PI * 2);
      ctx.fill();

      // Interaction souris douce
      const dx = mouse.x - p.x;
      const dy = mouse.y - p.y;
      const distance = Math.hypot(dx, dy);
      if (distance < 180) {
        p.speedX += dx / 20000;
        p.speedY += dy / 20000;
      }

      p.x += p.speedX;
      p.y += p.speedY;

      // Boucle infinie aux bords
      if (p.x < -p.size) p.x = canvas.width + p.size;
      if (p.x > canvas.width + p.size) p.x = -p.size;
      if (p.y < -p.size) p.y = canvas.height + p.size;
      if (p.y > canvas.height + p.size) p.y = -p.size;
    });

    animationId = requestAnimationFrame(animate);
  };

  onMount(() => {
    ctx = canvas.getContext('2d');

    const resize = () => {
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;
      createParticles();
    };

    resize();
    animate();

    // Suivi souris
    window.addEventListener('mousemove', (e) => {
      mouse.x = e.clientX;
      mouse.y = e.clientY;
    });

    // Recréer les particules quand le thème change
    const unsubscribe = currentTheme.subscribe(() => {
      createParticles();
    });

    window.addEventListener('resize', resize);

    // Applique la classe thème sur body (comme ton ancien layout)
    const unsubscribeBody = currentTheme.subscribe((theme) => {
      document.body.className = `theme-${theme}`;
    });

    return () => {
      cancelAnimationFrame(animationId);
      unsubscribe();
      unsubscribeBody();
      window.removeEventListener('resize', resize);
      window.removeEventListener('mousemove', () => {});
    };
  });
</script>

<!-- Canvas fond global -->
<canvas
  bind:this={canvas}
  class="fixed inset-0 -z-10 pointer-events-none"
/>

<!-- Contenu des pages -->
<slot />