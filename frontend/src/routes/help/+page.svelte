<!-- frontend/src/routes/help/+page.svelte — Session 39
     Nook v0.4.0-beta.2 :
     - Picker emoji natif + GIFs locaux (plus de proxy Tenor)
     - E2EE actif (clés X25519 générées au login, chiffrement transparent)
     - Push notifications VAPID (Paramètres → Sécurité)
     - Bouton d'urgence connecté (push à tous les membres)
     - Chess IA temps réel (WS broadcast)
-->
<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  let searchQuery = $state('');
  let openFaq     = $state<number | null>(null);

  const faqs = [
    {
      question: 'Quel est mon identifiant de connexion ?',
      answer:
        "Votre identifiant de connexion (ex: \"admin\", \"julien\") est permanent et ne peut pas être modifié. " +
        "En revanche, votre prénom affiché peut être changé depuis Paramètres → Profil. " +
        "Même si vous avez changé votre nom affiché, vous devez toujours utiliser l'identifiant original pour vous connecter.",
    },
    {
      question: 'Comment créer un compte sur Nook ?',
      answer:
        "Deux façons :\n" +
        "• Inscription libre → \"Créer un compte\" depuis la page de connexion. Votre compte sera en attente d'approbation par l'administrateur.\n" +
        "• Via lien d'invitation → L'administrateur génère un lien (valable 48h, usage unique). Votre compte est approuvé automatiquement.",
    },
    {
      question: 'Pourquoi les images envoyées dans le chat ne s\'affichent pas immédiatement ?',
      answer:
        "Tous les fichiers envoyés dans Nook sont chiffrés automatiquement (XChaCha20-Poly1305) " +
        "avant d'être sauvegardés sur le serveur. L'image est déchiffrée et affichée directement dans la bulle de message. " +
        "Si une image apparaît comme un nom de fichier brisé, c'est un ancien message envoyé avant la mise à jour — cliquez dessus pour le télécharger.",
    },
    {
      question: 'Comment envoyer un fichier ou une image ?',
      answer:
        "Dans le chat, cliquez sur l'icône 📎 en bas de l'écran. " +
        "Les images sont affichées directement dans la conversation. " +
        "Les autres fichiers (PDF, documents...) apparaissent comme un lien cliquable à télécharger. " +
        "Limite : 50 Mo par fichier. Les fichiers sont supprimés automatiquement après 48h.",
    },
    {
      question: 'Comment utiliser les emojis et les GIFs ?',
      answer:
        "Cliquez sur le bouton 😊 dans la barre de saisie du chat.\n" +
        "• Onglet Emoji : 292 emojis répartis en 8 catégories (Visages, Cœurs, Fête, Animaux…). " +
        "Cliquer sur un emoji l'insère dans votre texte en cours, ou l'envoie directement si l'input est vide.\n" +
        "• Onglet GIF 🎬 : collection de GIFs curatés stockés directement sur votre serveur. " +
        "Aucune donnée ne sort de chez vous — tout est local.",
    },
    {
      question: 'Comment changer l\'apparence de l\'application ?',
      answer:
        "Allez dans Paramètres → Apparence. Trois thèmes sont disponibles :\n" +
        "• 🌿 Jardin Secret — tons verts apaisants (défaut)\n" +
        "• 🌌 Space Hub — fond sombre, accents violets\n" +
        "• 🏠 Maison Chaleureuse — tons ambrés et chaleureux\n" +
        "Activez aussi le Mode Sombre (bascule) pour assombrir n'importe quel thème. " +
        "Le choix est sauvegardé automatiquement.",
    },
    {
      question: 'Mes messages sont-ils vraiment chiffrés ?',
      answer:
        "Nook utilise deux niveaux de chiffrement :\n" +
        "• Fichiers : toujours chiffrés (XChaCha20-Poly1305) côté serveur.\n" +
        "• Messages texte : chiffrement de bout en bout (E2EE) avec libsodium/Curve25519 quand les deux parties ont activé leurs clés dans Paramètres → Sécurité. " +
        "Les messages E2EE ne sont lisibles que par les destinataires — ni l'admin ni le serveur ne peuvent les lire.",
    },
    {
      question: 'Comment créer un sondage ?',
      answer:
        "Dans la page Sondages, cliquez \"＋ Nouveau sondage\". " +
        "Donnez une question et au moins 2 options (10 maximum). " +
        "Chaque membre peut voter et modifier son vote tant que le sondage est ouvert. " +
        "Le créateur ou l'administrateur peut fermer le sondage à tout moment — les résultats restent visibles.",
    },
    {
      question: 'Comment jouer aux échecs contre l\'IA ?',
      answer:
        "Dans la page Échecs, créez une nouvelle partie et choisissez votre couleur + la difficulté IA (Facile / Moyen / Difficile). " +
        "L'IA utilise l'algorithme Minimax avec élagage alpha-bêta. " +
        "Pour jouer contre un autre membre, laissez la difficulté sur \"Humain vs Humain\".",
    },
    {
      question: 'Comment accéder aux statistiques (admin) ?',
      answer:
        "Si vous êtes administrateur, allez dans /admin et cliquez sur l'onglet \"📊 Analytics\". " +
        "Vous y trouverez : nombre d'utilisateurs, messages, conversations, sondages, fichiers, " +
        "utilisateurs actifs sur 7 jours, et un graphique des messages par jour.",
    },
    {
      question: 'Comment activer les notifications push ?',
      answer:
        "Allez dans Paramètres → Sécurité → \"🔔 Notifications push\". " +
        "Cliquez \"Activer les notifications\" et acceptez la demande de permission du navigateur. " +
        "Vous recevrez une notification sur cet appareil pour chaque nouveau message, " +
        "même quand l'application est fermée ou en arrière-plan. " +
        "Désactivable à tout moment depuis le même écran.",
    },
    {
      question: 'Comment envoyer une alerte d\'urgence ?',
      answer:
        "La page /help dispose d'un bouton d'urgence accessible à tous les membres. " +
        "En cas de situation critique, envoyez un message d'urgence : " +
        "tous les membres connectés sont notifiés immédiatement via l'application " +
        "et reçoivent une notification push sur leurs appareils. " +
        "Le message est également enregistré dans les logs du serveur.",
    },
    {
      question: 'Comment faire un appel vidéo ?',
      answer:
        "Dans une conversation, cliquez sur l'icône 📞. Nook utilise WebRTC en pair-à-pair (P2P) : " +
        "les flux vidéo/audio passent directement entre les participants sans transiter par le serveur. " +
        "⚠️ Les appels sont stables en réseau local (LAN). Depuis internet (WAN) un serveur TURN est nécessaire — " +
        "cette fonctionnalité est en cours d'amélioration.",
    },
    {
      question: 'Puis-je utiliser Nook comme application mobile ?',
      answer:
        "Oui, Nook est une PWA (Progressive Web App). Sur Android/Chrome : menu → \"Ajouter à l'écran d'accueil\". " +
        "Sur iOS/Safari : bouton Partager → \"Sur l'écran d'accueil\". " +
        "Sur ordinateur (Chrome/Edge) : icône d'installation dans la barre d'adresse.",
    },
    {
      question: "Que faire si j'ai oublié mon mot de passe ?",
      answer:
        "Contactez l'administrateur familial. Il peut générer un nouveau lien d'invitation pour vous permettre de réinitialiser votre compte. " +
        "Vos anciens messages chiffrés resteront accessibles si vos clés E2EE sont toujours stockées dans votre navigateur.",
    },
    {
      question: 'Où sont stockées mes données ?',
      answer:
        "Toutes vos données sont sur votre serveur auto-hébergé (Docker volume) : " +
        "messages, fichiers, photos, événements, parties d'échecs, sondages. " +
        "Aucune donnée n'est envoyée vers des serveurs externes — " +
        "les emojis et GIFs sont servis directement depuis votre serveur. " +
        "Configurez des sauvegardes régulières du volume Docker pour ne rien perdre.",
    },
  ];

  const filteredFaqs = $derived(
    searchQuery.trim()
      ? faqs.filter(
          faq =>
            faq.question.toLowerCase().includes(searchQuery.toLowerCase()) ||
            faq.answer.toLowerCase().includes(searchQuery.toLowerCase())
        )
      : faqs
  );

  function toggle(i: number) { openFaq = openFaq === i ? null : i; }
</script>

<svelte:head><title>Aide — Nook</title></svelte:head>

<div class="help-page">

  <header class="help-header">
    <h1><Icon name="help" size="24" /> Aide</h1>
    <p class="subtitle">Questions fréquentes sur Nook</p>
  </header>

  <!-- Recherche -->
  <div class="search-wrapper">
    <span class="search-icon">🔍</span>
    <input
      type="search"
      class="search-input"
      placeholder="Rechercher une question…"
      bind:value={searchQuery}
    />
    {#if searchQuery}
      <button class="search-clear" onclick={() => searchQuery = ''} aria-label="Effacer">✕</button>
    {/if}
  </div>

  <!-- Résultats -->
  {#if filteredFaqs.length === 0}
    <div class="no-results">
      <span>😕</span>
      <p>Aucune question ne correspond à « {searchQuery} »</p>
    </div>
  {:else}
    <div class="faq-list">
      {#each filteredFaqs as faq, i (faq.question)}
        <div class="faq-item" class:open={openFaq === i}>
          <button class="faq-question" onclick={() => toggle(i)} aria-expanded={openFaq === i}>
            <span class="faq-q-text">{faq.question}</span>
            <span class="faq-chevron" class:rotated={openFaq === i}>▸</span>
          </button>
          {#if openFaq === i}
            <div class="faq-answer">
              {#each faq.answer.split('\n') as line}
                {#if line.startsWith('•')}
                  <p class="bullet">{line}</p>
                {:else if line.trim()}
                  <p>{line}</p>
                {/if}
              {/each}
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}

  <!-- Contact admin -->
  <div class="contact-card">
    <h2><Icon name="chat" size="24" /> Besoin d'aide supplémentaire ?</h2>
    <p>Envoyez un message à votre administrateur familial directement dans le chat Nook (conversation Groupe Global), ou demandez-lui de régénérer votre accès.</p>
  </div>

  <!-- Version -->
  <p class="version-note">Nook v0.4.0-beta.2 — Auto-hébergé, chiffré, familial 🌿</p>

</div>

<style>
  .help-page { max-width: 720px; margin: 0 auto; padding: 1.5rem 1rem; color: var(--text-primary); }

  .help-header { text-align: center; margin-bottom: 1.5rem; }
  .help-header h1 { font-size: 1.8rem; margin: 0 0 .4rem; color: var(--text-primary); }
  .subtitle { color: var(--text-secondary); font-size: .95rem; margin: 0; }

  /* ─── Recherche ─── */
  .search-wrapper {
    position: relative; display: flex; align-items: center;
    margin-bottom: 1.5rem;
    background: var(--bg-secondary); border: 1.5px solid var(--border);
    border-radius: var(--radius-full); padding: 0 1rem;
    transition: border-color .2s;
  }
  .search-wrapper:focus-within { border-color: var(--accent); box-shadow: var(--glow-sm); }
  .search-icon { font-size: 1rem; color: var(--text-muted); flex-shrink: 0; }
  .search-input {
    flex: 1; border: none; background: transparent;
    padding: .75rem .65rem; font-size: .95rem; color: var(--text-primary); outline: none;
  }
  .search-input::placeholder { color: var(--text-muted); }
  .search-clear { background: none; border: none; cursor: pointer; font-size: .9rem; color: var(--text-muted); padding: 0 .2rem; }
  .search-clear:hover { color: var(--text-primary); }

  /* ─── FAQ ─── */
  .faq-list { display: flex; flex-direction: column; gap: .5rem; margin-bottom: 2rem; }

  .faq-item {
    background: var(--bg-primary); border: 1.5px solid var(--border);
    border-radius: var(--radius-xl); overflow: hidden;
    transition: border-color .2s, box-shadow .2s;
  }
  .faq-item.open { border-color: var(--accent); box-shadow: var(--depth); }

  .faq-question {
    width: 100%; display: flex; align-items: center; justify-content: space-between;
    gap: 1rem; padding: 1rem 1.25rem; background: none; border: none;
    text-align: left; cursor: pointer; font-size: .95rem; font-weight: 600;
    color: var(--text-primary); transition: background .15s;
  }
  .faq-question:hover { background: var(--bg-secondary); }
  .faq-q-text { flex: 1; line-height: 1.4; }
  .faq-chevron { font-size: .85rem; color: var(--accent); transition: transform .2s; flex-shrink: 0; }
  .faq-chevron.rotated { transform: rotate(90deg); }

  .faq-answer {
    padding: 0 1.25rem 1rem;
    border-top: 1px solid var(--border);
    animation: slideDown .15s ease-out;
  }
  .faq-answer p { margin: .6rem 0 0; font-size: .9rem; color: var(--text-secondary); line-height: 1.6; }
  .faq-answer p.bullet { padding-left: .5rem; color: var(--text-primary); }

  @keyframes slideDown { from { opacity: 0; transform: translateY(-6px); } to { opacity: 1; transform: none; } }

  .no-results { text-align: center; padding: 3rem 1rem; color: var(--text-secondary); }
  .no-results span { font-size: 2.5rem; display: block; margin-bottom: .75rem; }

  /* ─── Contact card ─── */
  .contact-card {
    background: color-mix(in srgb, var(--accent) 10%, var(--bg-secondary));
    border: 1.5px solid color-mix(in srgb, var(--accent) 30%, transparent);
    border-radius: var(--radius-xl); padding: 1.5rem; margin-bottom: 1.5rem;
  }
  .contact-card h2 { margin: 0 0 .5rem; font-size: 1rem; color: var(--text-primary); }
  .contact-card p  { margin: 0; font-size: .9rem; color: var(--text-secondary); line-height: 1.5; }

  .version-note { text-align: center; font-size: .78rem; color: var(--text-muted); }

  @media (max-width: 480px) {
    .faq-question { padding: .85rem 1rem; font-size: .9rem; }
    .faq-answer   { padding: 0 1rem .85rem; }
  }
</style>
