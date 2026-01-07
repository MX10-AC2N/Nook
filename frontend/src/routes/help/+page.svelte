<script lang="ts">
  let searchQuery = $state('');
  let openFaq = $state<number | null>(null);

  const faqs = [
    {
      question: 'Comment créer un compte sur Nook ?',
      answer: 'Il y a deux façons de créer un compte :\n' +
        '• Inscription libre : Cliquez sur "Créer un compte" depuis la page de connexion. Votre compte sera créé en attente d\'approbation par l\'administrateur.\n' +
        '• Via invitation : Utilisez un lien d\'invitation généré par l\'administrateur. Votre compte sera automatiquement approuvé et vous devrez définir un mot de passe à la première connexion.'
    },
    {
      question: 'Mes messages sont-ils vraiment chiffrés ?',
      answer: 'Oui ! Nook utilise le chiffrement de bout en bout avec libsodium. Les messages sont chiffrés sur votre appareil avant envoi. Seul le destinataire peut les déchiffrer. Même les administrateurs du serveur ne peuvent pas lire vos conversations.'
    },
    {
      question: 'Comment faire un appel vidéo ?',
      answer: 'Dans une conversation, cliquez sur l\'icône 📞 pour lancer un appel. Nook utilise WebRTC en pair-à-pair (P2P) : les flux vidéo/audio passent directement entre les participants sans transiter par le serveur (sauf fallback si nécessaire).'
    },
    {
      question: 'Puis-je envoyer des fichiers volumineux ?',
      answer: 'Oui, jusqu\'à 50 Mo par fichier via le serveur (chiffrés). Pour des fichiers plus gros, une fonctionnalité d\'envoi P2P direct est prévue dans une future mise à jour.'
    },
    {
      question: 'Comment fonctionnent les invitations ?',
      answer: 'L\'administrateur peut générer des liens d\'invitation uniques depuis le tableau de bord. Ces liens permettent une inscription immédiate (compte approuvé automatiquement) et sont à usage unique avec expiration de 48 heures.'
    },
    {
      question: 'Puis-je utiliser Nook hors ligne ?',
      answer: 'Nook nécessite une connexion pour la synchronisation et le chiffrement. Les messages déjà chargés restent lisibles hors ligne, mais les nouveaux messages et appels nécessitent une connexion internet.'
    },
    {
      question: 'Comment installer Nook comme application ?',
      answer: 'Nook est une PWA (Progressive Web App) :\n' +
        '• Mobile : Ouvrez dans Chrome/Safari → "Ajouter à l\'écran d\'accueil".\n' +
        '• Ordinateur : Dans Chrome/Edge → menu → "Installer Nook".\n' +
        'Vous aurez une app native-like avec notifications push (en développement).'
    },
    {
      question: 'Mes données sont-elles sauvegardées ?',
      answer: 'Vos données sont stockées sur votre serveur auto-hébergé (Docker volume). Configurez des sauvegardes régulières du volume Docker. Aucune donnée n\'est envoyée vers des serveurs externes.'
    },
    {
      question: 'Que faire si j\'ai oublié mon mot de passe ?',
      answer: 'Contactez votre administrateur familial : il peut générer un nouveau lien d\'invitation pour vous permettre de recréer un compte (les anciens messages restent accessibles via le serveur).'
    }
  ];

  let filteredFaqs = $derived(
    searchQuery.trim() 
      ? faqs.filter(faq => 
          faq.question.toLowerCase().includes(searchQuery.toLowerCase()) ||
          faq.answer.toLowerCase().includes(searchQuery.toLowerCase())
        )
      : faqs
  );

  function toggleFaq(index: number) {
    openFaq = openFaq === index ? null : index;
  }

  function contactSupport() {
    window.open('https://github.com/MX10-AC2N/Nook/issues', '_blank');
  }
</script>

<svelte:head>
  <title>Aide & FAQ - Nook</title>
</svelte:head>

<div class="help-page">
  <div class="help-card">
    <div class="header">
      <h1>❓ Aide & FAQ</h1>
      <p class="subtitle">Tout ce que vous devez savoir pour utiliser Nook</p>
    </div>

    <div class="search-bar">
      <input
        type="text"
        bind:value={searchQuery}
        placeholder="Rechercher dans la FAQ..."
        aria-label="Rechercher dans l'aide"
      />
    </div>

    <div class="faq-section">
      {#if filteredFaqs.length === 0}
        <div class="no-results">
          <p>Aucun résultat pour "<strong>{searchQuery}</strong>"</p>
          <button on:click={contactSupport} class="btn-secondary">
            Contacter le support GitHub
          </button>
        </div>
      {:else}
        <div class="faq-list">
          {#each filteredFaqs as faq, index}
            <div class="faq-item" class:open={openFaq === index}>
              <button
                class="faq-question"
                on:click={() => toggleFaq(index)}
                aria-expanded={openFaq === index}
                aria-controls={`faq-answer-${index}`}
              >
                <span class="question-text">{faq.question}</span>
                <span class="faq-toggle">{openFaq === index ? '−' : '+'}</span>
              </button>
              {#if openFaq === index}
                <div class="faq-answer" id={`faq-answer-${index}`}>
                  {@html faq.answer.replace(/\n/g, '<br>')}
                </div>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <div class="support-section">
      <div class="support-card">
        <h3>💬 Besoin d'aide supplémentaire ?</h3>
        <p>Consultez la documentation ou signalez un problème sur GitHub.</p>
        <div class="support-links">
          <a href="https://github.com/MX10-AC2N/Nook" target="_blank" rel="noopener" class="btn-outline">
            📚 Documentation complète
          </a>
          <a href="https://github.com/MX10-AC2N/Nook/issues" target="_blank" rel="noopener" class="btn-outline">
            🐛 Signaler un bug
          </a>
        </div>
      </div>

      <div class="support-card security">
        <h3>🔒 Sécurité & Confidentialité</h3>
        <p>Nook est 100% open source. Le chiffrement de bout en bout protège vos données. Aucun serveur tiers n'accède à vos conversations.</p>
        <a href="https://github.com/MX10-AC2N/Nook" target="_blank" rel="noopener" class="btn-primary">
          Voir le code source →
        </a>
      </div>
    </div>
  </div>
</div>

<style>
  .help-page {
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 2rem 1rem;
    background: linear-gradient(135deg, #f0fdf4 0%, #e0f2fe 100%);
  }

  .help-card {
    background: white;
    padding: 3rem 2.5rem;
    border-radius: 1.5rem;
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.1);
    width: 100%;
    max-width: 800px;
  }

  .header {
    text-align: center;
    margin-bottom: 2.5rem;
  }

  h1 {
    font-size: 2rem;
    color: #1e293b;
    margin: 0 0 0.75rem 0;
  }

  .subtitle {
    color: #64748b;
    font-size: 1.1rem;
  }

  .search-bar {
    margin-bottom: 2rem;
  }

  .search-bar input {
    width: 100%;
    padding: 1rem 1.5rem;
    font-size: 1rem;
    border: 2px solid #e2e8f0;
    border-radius: 1rem;
    background: #f8fafc;
    transition: all 0.2s;
  }

  .search-bar input:focus {
    outline: none;
    border-color: #2d5a27;
    box-shadow: 0 0 0 4px rgba(45, 90, 39, 0.15);
  }

  .faq-section {
    margin-bottom: 3rem;
  }

  .no-results {
    text-align: center;
    padding: 3rem 1rem;
    color: #64748b;
  }

  .no-results strong {
    color: #1e293b;
  }

  .btn-secondary {
    margin-top: 1rem;
    padding: 0.75rem 1.5rem;
    background: #2d5a27;
    color: white;
    border: none;
    border-radius: 0.75rem;
    cursor: pointer;
  }

  .faq-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .faq-item {
    background: #f8fafc;
    border-radius: 1rem;
    overflow: hidden;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
    transition: all 0.2s;
  }

  .faq-item.open {
    background: white;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.1);
  }

  .faq-question {
    width: 100%;
    padding: 1.25rem 1.5rem;
    background: none;
    border: none;
    display: flex;
    justify-content: space-between;
    align-items: center;
    cursor: pointer;
    font-size: 1rem;
    font-weight: 600;
    color: #1e293b;
    text-align: left;
    transition: background 0.2s;
  }

  .faq-question:hover {
    background: rgba(45, 90, 39, 0.05);
  }

  .question-text {
    flex: 1;
    padding-right: 1rem;
  }

  .faq-toggle {
    font-size: 1.5rem;
    color: #2d5a27;
    font-weight: 300;
  }

  .faq-answer {
    padding: 0 1.5rem 1.5rem;
    color: #475569;
    font-size: 0.95rem;
    line-height: 1.7;
    animation: slideDown 0.3s ease-out;
  }

  @keyframes slideDown {
    from { opacity: 0; max-height: 0; }
    to { opacity: 1; max-height: 500px; }
  }

  .support-section {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: 1.5rem;
  }

  .support-card {
    background: #f0fdf4;
    padding: 1.75rem;
    border-radius: 1rem;
    border: 1px solid #bbf7d0;
  }

  .support-card.security {
    background: #ecfdf5;
    border-color: #86efac;
  }

  .support-card h3 {
    font-size: 1.1rem;
    color: #166534;
    margin-bottom: 0.75rem;
  }

  .support-card p {
    color: #166534;
    font-size: 0.9rem;
    line-height: 1.6;
    margin-bottom: 1.25rem;
  }

  .support-links {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .btn-outline {
    padding: 0.75rem 1rem;
    background: white;
    color: #2d5a27;
    border: 1px solid #2d5a27;
    border-radius: 0.75rem;
    text-decoration: none;
    font-size: 0.9rem;
    text-align: center;
    transition: all 0.2s;
  }

  .btn-outline:hover {
    background: #2d5a27;
    color: white;
  }

  .btn-primary {
    display: block;
    padding: 0.75rem 1rem;
    background: #2d5a27;
    color: white;
    border-radius: 0.75rem;
    text-decoration: none;
    font-size: 0.9rem;
    text-align: center;
    transition: all 0.2s;
  }

  .btn-primary:hover {
    background: #1e4620;
  }

  @media (max-width: 480px) {
    .help-card {
      padding: 2rem 1.5rem;
    }
    
    h1 {
      font-size: 1.75rem;
    }
  }
</style>