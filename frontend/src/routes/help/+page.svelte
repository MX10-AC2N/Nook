<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { isAuthenticated } from '$lib/authStore';

	let searchQuery = $state('');
	let openFaq = $state<number | null>(null);

	const faqs = [
		{
			question: 'Comment créer un compte sur Nook ?',
			answer: 'Pour créer un compte, vous avez besoin d\'un lien d\'invitation généré par un administrateur. Cliquez sur le lien d\'invitation, puis remplissez le formulaire avec votre prénom, un identifiant unique et un mot de passe. Votre compte sera ensuite en attente d\'approbation par l\'administrateur.'
		},
		{
			question: 'Mes messages sont-ils vraiment chiffrés ?',
			answer: 'Oui ! Nook utilise le chiffrement de bout en bout avec la bibliothèque libsodium. Chaque message est chiffré sur votre appareil avant d\'être envoyé. Même nous, les développeurs, ne pouvons pas lire vos conversations. Seuls les destinataires peuvent déchiffrer les messages.'
		},
		{
			question: 'Comment faire un appel vidéo ?',
			answer: 'Dans une conversation, cliquez sur le bouton d\'appel (📞) pour iniciar un appel. Votre navigateur vous demandera l\'autorisation d\'accéder à votre caméra et votre microphone. Les appels sont entièrement en P2P (pair-à-pair), ce qui signifie que les flux médias ne passent pas par nos serveurs.'
		},
		{
			question: 'Puis-je envoyer des fichiers volumineux ?',
			answer: 'Nook permet d\'envoyer des fichiers jusqu\'à 50 Mo directement via le serveur. Pour les fichiers plus volumineux, une fonctionnalité P2P direct est en développement qui vous permettra d\'envoyer des fichiers sans passer par le serveur.'
		},
		{
			question: 'Comment fonctionnent les invitations ?',
			answer: 'Seuls les administrateurs peuvent générer des liens d\'invitation. Ces liens sont uniques et permettent aux nouveaux membres de demander à rejoindre votre espace familial. Chaque demande doit être approuvée manuellement par un administrateur pour des raisons de sécurité.'
		},
		{
			question: 'Puis-je utiliser Nook hors ligne ?',
			answer: 'Nook est conçu pour fonctionner principalement en ligne afin de garantir le chiffrement et la synchronisation des messages. Cependant, les messages déjà chargés sont accessibles hors ligne dans certains cas. Les notifications push sont en développement pour vous alerter de nouveaux messages.'
		},
		{
			question: 'Comment installer Nook comme application ?',
			answer: 'Sur mobile : ouvrez Nook dans votre navigateur, puis utilisez l\'option "Ajouter à l\'écran d\'accueil" (iOS) ou "Installer l\'application" (Android). Sur ordinateur : Chrome et Edge proposent également l\'installation via le menu du navigateur.'
		},
		{
			question: 'Mes données sont-elles sauvegardées ?',
			answer: 'Les messages et fichiers sont stockés sur votre serveur auto-hébergé. Nous vous recommandons de configurer des sauvegardes automatiques de votre volume de données. Aucune donnée n\'est stockée sur des serveurs cloud externes.'
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
	<title>Aide - Nook</title>
</svelte:head>

<div class="help-container">
	<div class="help-header">
		<h1>❓ Aide & FAQ</h1>
		<p>Tout ce que vous devez savoir sur Nook</p>
	</div>

	<div class="help-search">
		<input 
			type="text" 
			bind:value={searchQuery} 
			placeholder="Rechercher dans l'aide..."
		/>
	</div>

	<div class="help-content">
		{#if filteredFaqs.length === 0}
			<div class="no-results">
				<p>Aucun résultat trouvé pour "{searchQuery}"</p>
				<button onclick={contactSupport}>Contacter le support</button>
			</div>
		{:else}
			<div class="faq-list">
				{#each filteredFaqs as faq, index}
					<div class="faq-item">
						<button 
							class="faq-question" 
							onclick={() => toggleFaq(index)}
							aria-expanded={openFaq === index}
						>
							<span>{faq.question}</span>
							<span class="faq-icon">{openFaq === index ? '−' : '+'}</span>
						</button>
						{#if openFaq === index}
							<div class="faq-answer">
								<p>{faq.answer}</p>
							</div>
						{/if}
					</div>
				{/each}
			</div>
		{/if}
	</div>

	<div class="help-footer">
		<div class="help-card">
			<h3>💬 Besoin d\'aide supplémentaire ?</h3>
			<p> Consultez notre documentation complète sur GitHub ou créez une issue si vous rencontrez un problème.</p>
			<div class="help-links">
				<a href="https://github.com/MX10-AC2N/Nook" target="_blank" rel="noopener">📚 Documentation</a>
				<a href="https://github.com/MX10-AC2N/Nook/issues" target="_blank" rel="noopener">🐛 Signaler un bug</a>
			</div>
		</div>

		<div class="help-card">
			<h3>🔒 Sécurité</h3>
			<p>Nook est open source et peut être audité par n\'importe qui. Le chiffrement de bout en bout garantit que vos conversations restent privées.</p>
			<a href="https://github.com/MX10-AC2N/Nook" target="_blank" rel="noopener" class="security-link">Voir le code source →</a>
		</div>
	</div>
</div>

<style>
	.help-container { max-width: 800px; margin: 0 auto; padding: 1rem; }
	
	.help-header { text-align: center; margin-bottom: 2rem; }
	.help-header h1 { font-size: 1.75rem; color: #2d5a27; margin-bottom: 0.5rem; }
	.help-header p { color: #666; }
	
	.help-search { margin-bottom: 2rem; }
	.help-search input { width: 100%; padding: 1rem 1.25rem; border: 2px solid #e0e0e0; border-radius: 12px; font-size: 1rem; transition: border-color 0.2s; }
	.help-search input:focus { outline: none; border-color: #2d5a27; }
	
	.help-content { margin-bottom: 2rem; }
	.no-results { text-align: center; padding: 2rem; color: #888; }
	.no-results button { margin-top: 1rem; padding: 0.75rem 1.5rem; background: #2d5a27; color: white; border: none; border-radius: 8px; cursor: pointer; }
	
	.faq-list { display: flex; flex-direction: column; gap: 0.5rem; }
	.faq-item { background: white; border-radius: 12px; box-shadow: 0 2px 8px rgba(0,0,0,0.1); overflow: hidden; }
	.faq-question { width: 100%; padding: 1rem 1.25rem; background: none; border: none; display: flex; justify-content: space-between; align-items: center; cursor: pointer; font-size: 0.95rem; font-weight: 500; color: #333; text-align: left; transition: background 0.2s; }
	.faq-question:hover { background: #f8f9fa; }
	.faq-icon { font-size: 1.25rem; color: #2d5a27; font-weight: 300; }
	.faq-answer { padding: 0 1.25rem 1rem; color: #666; font-size: 0.9rem; line-height: 1.6; animation: slideDown 0.2s ease-out; }
	
	@keyframes slideDown {
		from { opacity: 0; transform: translateY(-10px); }
		to { opacity: 1; transform: translateY(0); }
	}
	
	.help-footer { display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 1rem; }
	.help-card { background: white; border-radius: 12px; padding: 1.5rem; box-shadow: 0 2px 8px rgba(0,0,0,0.1); }
	.help-card h3 { font-size: 1rem; color: #333; margin-bottom: 0.5rem; }
	.help-card p { color: #666; font-size: 0.9rem; line-height: 1.5; margin-bottom: 1rem; }
	
	.help-links { display: flex; gap: 0.75rem; flex-wrap: wrap; }
	.help-links a { padding: 0.5rem 1rem; background: #f0f7f0; color: #2d5a27; text-decoration: none; border-radius: 6px; font-size: 0.85rem; transition: background 0.2s; }
	.help-links a:hover { background: #e0f0e0; }
	
	.security-link { display: inline-block; color: #2d5a27; text-decoration: none; font-weight: 500; }
	.security-link:hover { text-decoration: underline; }
</style>
