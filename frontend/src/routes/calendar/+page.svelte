<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { isAuthenticated } from '$lib/authStore';

	let currentDate = $state(new Date());
	let events = $state<any[]>([]);
	let showAddModal = $state(false);
	let newEvent = $state({ title: '', date: '', time: '', description: '' });
	let loading = $state(true);

	const monthNames = ['Janvier', 'Février', 'Mars', 'Avril', 'Mai', 'Juin', 'Juillet', 'Août', 'Septembre', 'Octobre', 'Novembre', 'Décembre'];

	onMount(async () => {
		if (!$isAuthenticated) {
			goto('/login');
			return;
		}
		await loadEvents();
		loading = false;
	});

	async function loadEvents() {
		try {
			const response = await fetch('/api/events', { credentials: 'include' });
			if (response.ok) {
				const data = await response.json();
				events = data.events || [];
			}
		} catch (err) {
			console.error('Erreur chargement événements:', err);
			events = [];
		}
	}

	async function addEvent() {
		if (!newEvent.title || !newEvent.date) {
			alert('Veuillez remplir le titre et la date');
			return;
		}

		try {
			const response = await fetch('/api/events', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				credentials: 'include',
				body: JSON.stringify({
					title: newEvent.title,
					date: newEvent.date,
					time: newEvent.time,
					description: newEvent.description
				})
			});

			if (response.ok) {
				await loadEvents();
				showAddModal = false;
				newEvent = { title: '', date: '', time: '', description: '' };
			}
		} catch (err) {
			console.error('Erreur création événement:', err);
		}
	}

	function prevMonth() {
		currentDate = new Date(currentDate.getFullYear(), currentDate.getMonth() - 1, 1);
	}

	function nextMonth() {
		currentDate = new Date(currentDate.getFullYear(), currentDate.getMonth() + 1, 1);
	}

	function getDaysInMonth(date: Date): number {
		return new Date(date.getFullYear(), date.getMonth() + 1, 0).getDate();
	}

	function getFirstDayOfMonth(date: Date): number {
		return new Date(date.getFullYear(), date.getMonth(), 1).getDay();
	}

	function getEventsForDay(day: number): any[] {
		const year = currentDate.getFullYear();
		const month = String(currentDate.getMonth() + 1).padStart(2, '0');
		const dayStr = String(day).padStart(2, '0');
		const dateStr = `${year}-${month}-${dayStr}`;
		return events.filter(e => e.date === dateStr);
	}

	function formatEventDate(dateStr: string): string {
		const date = new Date(dateStr);
		return date.toLocaleDateString('fr-FR', { day: '2-digit', month: '2-digit' });
	}
</script>

<svelte:head>
	<title>Calendrier - Nook</title>
</svelte:head>

<div class="calendar-container">
	<div class="calendar-header">
		<h1>📅 Calendrier Familial</h1>
		<button class="add-btn" onclick={() => showAddModal = true}>+ Ajouter un événement</button>
	</div>

	<div class="calendar-wrapper">
		<div class="calendar">
			<div class="calendar-nav">
				<button onclick={prevMonth}>◀</button>
				<h2>{monthNames[currentDate.getMonth()]} {currentDate.getFullYear()}</h2>
				<button onclick={nextMonth}>▶</button>
			</div>

			<div class="calendar-weekdays">
				<span>Dim</span><span>Lun</span><span>Mar</span><span>Mer</span><span>Jeu</span><span>Ven</span><span>Sam</span>
			</div>

			<div class="calendar-days">
				{#each Array(getFirstDayOfMonth(currentDate)) as _}
					<div class="day empty"></div>
				{/each}
				{#each Array(getDaysInMonth(currentDate)) as _, i}
					{@const dayEvents = getEventsForDay(i + 1)}
					<div class="day" class:today={new Date().toDateString() === new Date(currentDate.getFullYear(), currentDate.getMonth(), i + 1).toDateString()}>
						<span class="day-number">{i + 1}</span>
						{#if dayEvents.length > 0}
							<div class="day-events">
								{#each dayEvents.slice(0, 2) as event}
									<span class="event-dot" title={event.title}>{event.title}</span>
								{/each}
								{#if dayEvents.length > 2}
									<span class="event-more">+{dayEvents.length - 2}</span>
								{/if}
							</div>
						{/if}
					</div>
				{/each}
			</div>
		</div>

		<div class="events-panel">
			<h3>Événements à venir</h3>
			{#if events.filter(e => new Date(e.date) >= new Date()).length === 0}
				<p class="no-events">Aucun événement à venir</p>
			{:else}
				<div class="events-list">
					{#each events.filter(e => new Date(e.date) >= new Date()).sort((a, b) => new Date(a.date).getTime() - new Date(b.date).getTime()) as event}
						<div class="event-card">
							<div class="event-date">
								<span class="event-day">{new Date(event.date).getDate()}</span>
								<span class="event-month">{monthNames[new Date(event.date).getMonth()].slice(0, 3)}</span>
							</div>
							<div class="event-details">
								<h4>{event.title}</h4>
								<p>{event.time || 'Toute la journée'}</p>
								{#if event.description}
									<p class="event-desc">{event.description}</p>
								{/if}
							</div>
						</div>
					{/each}
				</div>
			{/if}
		</div>
	</div>
</div>

{#if showAddModal}
	<div class="modal-overlay" onclick={() => showAddModal = false}>
		<div class="modal" onclick={(e) => e.stopPropagation()}>
			<h3>Nouvel événement</h3>
			<form onsubmit={(e) => { e.preventDefault(); addEvent(); }}>
				<div class="form-group">
					<label for="title">Titre</label>
					<input type="text" id="title" bind:value={newEvent.title} placeholder="Anniversaire, Réunion..." required />
				</div>
				<div class="form-row">
					<div class="form-group">
						<label for="date">Date</label>
						<input type="date" id="date" bind:value={newEvent.date} required />
					</div>
					<div class="form-group">
						<label for="time">Heure</label>
						<input type="time" id="time" bind:value={newEvent.time} />
					</div>
				</div>
				<div class="form-group">
					<label for="description">Description (optionnel)</label>
					<textarea id="description" bind:value={newEvent.description} placeholder="Détails de l'événement..."></textarea>
				</div>
				<div class="modal-actions">
					<button type="button" class="cancel-btn" onclick={() => showAddModal = false}>Annuler</button>
					<button type="submit" class="submit-btn">Créer</button>
				</div>
			</form>
		</div>
	</div>
{/if}

<style>
	.calendar-container { max-width: 1000px; margin: 0 auto; padding: 1rem; }
	.calendar-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 1.5rem; }
	.calendar-header h1 { font-size: 1.5rem; color: #2d5a27; }
	.add-btn { padding: 0.75rem 1.25rem; background: #2d5a27; color: white; border: none; border-radius: 8px; cursor: pointer; transition: background 0.2s; }
	.add-btn:hover { background: #3d7a37; }
	
	.calendar-wrapper { display: grid; grid-template-columns: 1fr 300px; gap: 1.5rem; }
	.calendar { background: white; border-radius: 12px; box-shadow: 0 2px 8px rgba(0,0,0,0.1); padding: 1rem; }
	.calendar-nav { display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; }
	.calendar-nav h2 { font-size: 1.1rem; color: #333; }
	.calendar-nav button { background: none; border: none; font-size: 1.2rem; cursor: pointer; padding: 0.5rem; color: #2d5a27; }
	
	.calendar-weekdays { display: grid; grid-template-columns: repeat(7, 1fr); text-align: center; font-size: 0.8rem; color: #888; margin-bottom: 0.5rem; }
	.calendar-days { display: grid; grid-template-columns: repeat(7, 1fr); gap: 2px; }
	.day { aspect-ratio: 1; padding: 0.25rem; border-radius: 8px; position: relative; cursor: pointer; transition: background 0.2s; }
	.day:hover { background: #f0f7f0; }
	.day.empty { background: transparent; }
	.day.today { background: #e8f5e9; }
	.day.today .day-number { background: #2d5a27; color: white; border-radius: 50%; width: 24px; height: 24px; display: flex; align-items: center; justify-content: center; }
	.day-number { font-size: 0.85rem; color: #333; }
	.day-events { margin-top: 2px; display: flex; flex-direction: column; gap: 1px; }
	.event-dot { font-size: 0.65rem; background: #2d5a27; color: white; padding: 1px 4px; border-radius: 3px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
	.event-more { font-size: 0.6rem; color: #888; text-align: center; }
	
	.events-panel { background: white; border-radius: 12px; box-shadow: 0 2px 8px rgba(0,0,0,0.1); padding: 1rem; height: fit-content; }
	.events-panel h3 { font-size: 1rem; color: #333; margin-bottom: 1rem; }
	.no-events { color: #888; font-size: 0.9rem; text-align: center; padding: 2rem 0; }
	.events-list { display: flex; flex-direction: column; gap: 0.75rem; }
	.event-card { display: flex; gap: 0.75rem; padding: 0.75rem; background: #f8f9fa; border-radius: 8px; }
	.event-date { display: flex; flex-direction: column; align-items: center; min-width: 40px; }
	.event-day { font-size: 1.25rem; font-weight: 600; color: #2d5a27; }
	.event-month { font-size: 0.7rem; color: #888; text-transform: uppercase; }
	.event-details h4 { font-size: 0.9rem; margin: 0 0 0.25rem 0; color: #333; }
	.event-details p { font-size: 0.8rem; color: #666; margin: 0; }
	.event-desc { font-size: 0.75rem !important; color: #888 !important; margin-top: 0.25rem !important; }
	
	.modal-overlay { position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000; }
	.modal { background: white; padding: 1.5rem; border-radius: 12px; width: 100%; max-width: 400px; }
	.modal h3 { font-size: 1.1rem; color: #333; margin-bottom: 1rem; }
	.form-group { margin-bottom: 1rem; }
	.form-row { display: grid; grid-template-columns: 1fr 1fr; gap: 0.75rem; }
	label { display: block; font-size: 0.85rem; color: #666; margin-bottom: 0.25rem; }
	input, textarea { width: 100%; padding: 0.625rem; border: 1px solid #ddd; border-radius: 6px; font-size: 0.9rem; }
	input:focus, textarea:focus { outline: none; border-color: #2d5a27; }
	textarea { min-height: 60px; resize: vertical; }
	.modal-actions { display: flex; gap: 0.75rem; justify-content: flex-end; margin-top: 1rem; }
	.cancel-btn { padding: 0.625rem 1rem; background: #f5f5f5; border: none; border-radius: 6px; cursor: pointer; }
	.submit-btn { padding: 0.625rem 1rem; background: #2d5a27; color: white; border: none; border-radius: 6px; cursor: pointer; }
	
	@media (max-width: 768px) {
		.calendar-wrapper { grid-template-columns: 1fr; }
		.events-panel { order: -1; }
	}
</style>
