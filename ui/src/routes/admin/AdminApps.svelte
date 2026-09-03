<script lang="ts">
	import { onMount } from 'svelte';

	type App = {
		id: number;
		name: string;
		developer: string;
		description: string;
		banner: string;
		enabled: boolean;
		available: boolean;
		default_tags: string[];
		tags: string[];
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

			apps = data.map((app) => {
				return {
					localKey: crypto.randomUUID(),
					app,
					original: snapshot(app),
					collapsed: true
				};
			});

			// Fetch all tags for each app to populate the checkbox list if it's empty
			for (const state of apps) {
				if (state.app.tags.length === 0) {
					fetch(`/api/app/${state.app.id}`)
						.then((res) => res.json())
						.then((appData) => {
							if (appData && appData.tags) {
								state.app.tags = appData.tags;
								state.original = snapshot(state.app); // Update original to avoid dirty state just from fetching tags
								apps = apps;
							}
						})
						.catch((err) => console.error(`Failed to fetch tags for app ${state.app.id}`, err));
				}
			}
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	}

	function newApp(): AppState {
		const app: App = {
			id: 294100, // temporary
			name: 'RimWorld',
			developer: 'Ludeon Studios',
			description:
				'RimWorld is a sci-fi colony sim driven by an intelligent AI storyteller. Inspired by Dwarf Fortress and Firefly, you manage colonists’ moods, needs, wounds, and survival while building and exploring emergent stories.',
			banner: 'https://cdn.akamai.steamstatic.com/steam/apps/294100/header.jpg',
			enabled: true,
			available: true,
			default_tags: [],
			tags: []
		};

		return {
			localKey: crypto.randomUUID(),
			app,
			original: 'nonsense',
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
		apps = apps;
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

		apps = apps.filter((a) => a !== state);
	}

	function getTag(tag: string, state: AppState) {
		return state.app.default_tags.includes(tag);
	}

	function setTag(checked: boolean, tag: string, state: AppState) {
		if (checked) {
			if (!state.app.default_tags.includes(tag)) {
				state.app.default_tags.push(tag);
			}
		} else {
			state.app.default_tags = state.app.default_tags.filter((id) => id !== tag);
		}
		apps = apps;
	}
</script>

{#if loading}
	<p class="text-surface-500 text-sm">Loading…</p>
{:else if error}
	<p class="text-error-500">{error}</p>
{/if}

<div class="space-y-6">
	<button class="btn preset-filled-primary-500" onclick={() => (apps = [...apps, newApp()])}>
		Add App
	</button>
	{@debug apps}
	{#each apps as state (state.localKey)}
		{@debug state}
		<div
			class="card preset-tonal-surface-50-950 border-surface-300 border"
			class:border-warning-400={isDirty(state)}
		>
			<!-- Header -->
			<button
				type="button"
				class="flex w-full items-center justify-between p-4 text-left"
				onclick={() => {
					return (state.collapsed = !state.collapsed);
				}}
			>
				<div>
					<h3 class="font-semibold">
						{state.app.name || 'New App'}
					</h3>
					{#if isDirty(state)}
						<p class="text-warning-500 text-xs">Unsaved changes</p>
					{/if}
				</div>
				<span class="text-sm opacity-70">
					{state.collapsed ? '▼' : '▲'}
				</span>
			</button>

			{#if !state.collapsed}
				<form
					class="border-surface-300 space-y-4 border-t p-4"
					onsubmit={(e) => {
						e.preventDefault();
						save(state);
					}}
				>
					<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
						<label class="label">
							<span>ID</span>
							<input type="number" class="input" bind:value={state.app.id} min="1" />
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
							<textarea class="textarea" rows="3" bind:value={state.app.description} />
						</label>

						<label class="label md:col-span-2">
							<span>Banner URL</span>
							<input class="input" bind:value={state.app.banner} />
						</label>
					</div>

					<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
						<label class="space-y-1">
							<div class="flex items-center gap-2">
								<input type="checkbox" class="checkbox" bind:checked={state.app.enabled} />
								<span class="font-medium">Enabled</span>
							</div>
							<p class="text-surface-500 text-xs">
								Allows interaction such as facets, votes, and companions.
							</p>
						</label>

						<label class="space-y-1">
							<div class="flex items-center gap-2">
								<input type="checkbox" class="checkbox" bind:checked={state.app.available} />
								<span class="font-medium">Available</span>
							</div>
							<p class="text-surface-500 text-xs">
								Controls whether the app is visible in public listings.
							</p>
						</label>
					</div>

					<fieldset class="space-y-2">
						<legend class="font-medium">Default Tags</legend>
						<div class="flex flex-wrap gap-4">
							{#each state.app.tags as tag}
								<label class="flex items-center space-x-2">
									<input
										class="checkbox"
										type="checkbox"
										checked={getTag(tag, state)}
										onchange={(e) => {
											setTag(e.currentTarget.checked, tag, state);
										}}
									/>
									<span>{tag}</span>
								</label>
							{:else}
								<p class="text-surface-500 text-sm">No available tags for this app.</p>
							{/each}
						</div>
					</fieldset>

					<div class="flex justify-between pt-2">
						<button
							type="button"
							class="btn preset-filled-error-500"
							disabled={!state.app.id}
							onclick={() => remove(state)}
						>
							Delete
						</button>

						<button
							type="submit"
							class="btn preset-filled-primary-500"
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
