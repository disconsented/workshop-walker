<script lang="ts">
	import { onMount } from 'svelte';

	type Tag = {
		app_id: number;
		display_name: string;
		tag: string;
	};

	type App = {
		id: number;
		name: string;
		developer: string;
		description: string;
		banner: string;
		enabled: boolean;
		available: boolean;
		default_tags: Tag[];
	};

	type AppState = {
		localKey: string; // client-only stable key
		app: App;
		original: string;
		collapsed: boolean;
	};

	let apps: AppState[] = [];
	let loading = false;
	let error: string | null = null;

	onMount(loadApps);

	function snapshot(app: App): string {
		return JSON.stringify(app);
	}

	function isDirty(state: AppState): boolean {
		return snapshot(state.app) !== state.original;
	}

	async function loadApps() {
		loading = true;
		try {
			const res = await fetch('/api/admin/apps');
			if (!res.ok) throw new Error('Failed to load apps');
			const data: App[] = await res.json();

			apps = data.map(app => ({
				localKey: crypto.randomUUID(),
				app,
				original: snapshot(app),
				collapsed: true
			}));
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	function newApp(): AppState {
		const app: App = {
			id: 294100,   // temporary
			name: 'RimWorld',
			developer: 'Ludeon Studios',
			description: 'RimWorld is a sci-fi colony sim driven by an intelligent AI storyteller. Inspired by Dwarf Fortress and Firefly, you manage colonists’ moods, needs, wounds, and survival while building and exploring emergent stories.',
			banner: 'https://cdn.akamai.steamstatic.com/steam/apps/294100/header.jpg',
			enabled: true,
			available: true,
			default_tags: [
				// { app_id: 294100, tag: '1.6', display_name: '1.6' },
				// { app_id: 294100, tag: 'mod', display_name: 'mod' }
			]
		};

		return {
			localKey: crypto.randomUUID(),
			app,
			original: "asd",
			collapsed: false
		};
	}

	async function save(state: AppState) {
		if (!state.app.id) {
			alert('ID must be set before saving');
			return;
		}

		const res = await fetch('/api/admin/apps', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify(state.app)
		});

		if (!res.ok) {
			alert('Failed to save app');
			return;
		}

		state.original = snapshot(state.app);
	}

	async function remove(state: AppState) {
		if (!state.app.id) return;

		if (!confirm(`Delete "${state.app.name}"?`)) return;

		const res = await fetch(`/api/admin/app?id=${state.app.id}`, {
			method: 'DELETE'
		});

		if (!res.ok) {
			alert('Failed to delete app');
			return;
		}

		apps = apps.filter(a => a !== state);
	}

	function addTag(state: AppState) {
		state.app.default_tags = [
			...state.app.default_tags,
			{
				app_id: state.app.id,
				display_name: '',
				tag: ''
			}
		];
	}

	function removeTag(state: AppState, i: number) {
		state.app.default_tags =
			state.app.default_tags.filter((_, idx) => idx !== i);
	}
</script>

{#if loading}
	<p class="text-sm text-surface-500">Loading…</p>
{:else if error}
	<p class="text-error-500">{error}</p>
{/if}

<div class="space-y-6">
	<button class="btn btn-primary" on:click={() => apps = [...apps, newApp()]}>
		Add App
	</button>

	{#each apps as state (state.localKey)}
		<div
			class="card border border-surface-300"
			class:border-warning-400={isDirty(state)}
		>
			<!-- Header -->
			<button
				type="button"
				class="w-full flex justify-between items-center p-4 text-left"
				on:click={() => state.collapsed = !state.collapsed}
			>
				<div>
					<h3 class="font-semibold">
						{state.app.name || 'New App'}
					</h3>
					{#if isDirty(state)}
						<p class="text-xs text-warning-500">
							Unsaved changes
						</p>
					{/if}
				</div>
				<span class="text-sm opacity-70">
					{state.collapsed ? '▼' : '▲'}
				</span>
			</button>

			{#if !state.collapsed}
				<form
					class="p-4 space-y-4 border-t border-surface-300"
					on:submit|preventDefault={() => save(state)}
				>
					<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
						<label class="label">
							<span>ID</span>
							<input
								type="number"
								class="input"
								bind:value={state.app.id}
								min="1"
							/>
						</label>

						<label class="label">
							<span>Name</span>
							<input class="input" bind:value={state.app.name} />
						</label>

						<label class="label">
							<span>Developer</span>
							<input class="input" bind:value={state.app.developer} />
						</label>

						<label class="label md:col-span-2">
							<span>Description</span>
							<textarea
								class="textarea"
								rows="3"
								bind:value={state.app.description}
							/>
						</label>

						<label class="label md:col-span-2">
							<span>Banner URL</span>
							<input class="input" bind:value={state.app.banner} />
						</label>
					</div>

					<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
						<label class="space-y-1">
							<div class="flex items-center gap-2">
								<input
									type="checkbox"
									class="checkbox"
									bind:checked={state.app.enabled}
								/>
								<span class="font-medium">Enabled</span>
							</div>
							<p class="text-xs text-surface-500">
								Allows interaction such as facets, votes, and companions.
							</p>
						</label>

						<label class="space-y-1">
							<div class="flex items-center gap-2">
								<input
									type="checkbox"
									class="checkbox"
									bind:checked={state.app.available}
								/>
								<span class="font-medium">Available</span>
							</div>
							<p class="text-xs text-surface-500">
								Controls whether the app is visible in public listings.
							</p>
						</label>
					</div>

					<fieldset class="space-y-2">
						<legend class="font-medium">Default Tags</legend>

						{#each state.app.default_tags as tag, i}
							<div class="flex gap-2">
								<input
									class="input flex-1"
									placeholder="Display Name"
									bind:value={tag.display_name}
								/>
								<input
									class="input flex-1"
									placeholder="Tag ID"
									bind:value={tag.tag}
								/>
								<button
									type="button"
									class="btn btn-error btn-sm"
									on:click={() => removeTag(state, i)}
								>
									✕
								</button>
							</div>
						{/each}

						<button
							type="button"
							class="btn btn-secondary btn-sm"
							on:click={() => addTag(state)}
						>
							Add Tag
						</button>
					</fieldset>

					<div class="flex justify-between pt-2">
						<button
							type="button"
							class="btn btn-error"
							disabled={!state.app.id}
							on:click={() => remove(state)}
						>
							Delete
						</button>

						<button
							type="submit"
							class="btn btn-primary"
							disabled={!isDirty(state) || !state.app.id}
						>
							Save
						</button>
					</div>
				</form>
			{/if}
		</div>
	{/each}
</div>
