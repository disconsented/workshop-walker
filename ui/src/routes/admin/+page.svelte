<script lang="ts">
	import { Tabs } from '@skeletonlabs/skeleton-svelte';
	import AppCard from '$lib/app_card.svelte';

	let { data }: { data } = $props();
	console.log(data);
	let properties = $state([]);

	let users = $state([]);
	data.users.then((data) => users.push(...data));
	data.properties.then((data) => properties.push(...data));

	let searchTerm = '';
	let statusFilter = 'all';

	// Status options
	const statusOptions = [
		{ value: '-1', label: 'Denied' },
		{ value: '0', label: 'Pending' },
		{ value: '1', label: 'Approved' },
		{ value: 'all', label: 'All Statuses' }
	];

	// Toggle functions
	async function togglePropertyStatus(prop: any, status: number) {
		prop.status = status;

		let res = await fetch('/api/admin/properties', {
			method: 'put',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({
				item: prop.in,
				class: prop.out.class,
				value: prop.out.value,
				status: status
			})
		});
		if (!res.ok) {
			console.error(res);
		}
	}

	async function toggleUserAdmin(id: number, value: boolean) {
		users = users.map((u) => (u.id === id ? { ...u, admin: !u.admin } : u));

		let res = await fetch('/api/admin/users', {
			method: 'put',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({
				id: id,
				admin: value
			})
		});
		if (!res.ok) {
			console.error(res);
		}
	}

	async function toggleUserBan(id: number, value: boolean) {
		users = users.map((u) => (u.id === id ? { ...u, banned: !u.banned } : u));
		let res = await fetch('/api/admin/users', {
			method: 'put',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({
				id: id,
				banned: value
			})
		});
		if (!res.ok) {
			console.error(res);
		}
	}

	let group = $state('properties');

	import { onMount } from 'svelte';
	import AdminApps from './AdminApps.svelte';

	export type Game = {
		appid: string;
		image_url: string;
		description: string;
		developer: string;
		name: string;
	};

	let games: Game[] = $state([]);

	// Example initial data
	onMount(() => {
		games = [
			{
				appid: '294100',
				image_url:
					'https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/294100/header.jpg?t=1734154189',
				description:
					' A sci-fi colony sim driven by an intelligent AI storyteller. Generates stories by simulating psychology, ecology, gunplay, melee combat, climate, biomes, diplomacy, interpersonal relationships, art, medicine, trade, and more.',
				developer: 'Ludeon Studios',
				name: 'Rimworld'
			},
			{
				appid: '1133870',
				image_url:
				'https://shared.fastly.steamstatic.com/store_item_assets/steam/apps/1133870/f5fb2e294898df4d3bf28b195f277d3df3511792/header.jpg?t=1764615043',
				description:
					'Space Engineers 2 is a sandbox about engineering and colonization where you build ships, stations, and planetary bases in a fully destructible world. Begin the story of Miro and Ivan Sokol, expand humanity’s foothold in the Almagest system, and fight to survive. ',
				developer: '\n' +
					'Keen Software House\t',
				name: 'Space Engineers 2'
			},
			{
				appid: '262060',
				image_url:
					'https://shared.fastly.steamstatic.com/store_item_assets/steam/apps/262060/header.jpg?t=1756404119',
				description:
					' Darkest Dungeon is a challenging gothic roguelike turn-based RPG about the psychological stresses of adventuring. Recruit, train, and lead a team of flawed heroes against unimaginable horrors, stress, disease, and the ever-encroaching dark. Can you keep your heroes together when all hope is lost?',
				developer: 'Red Hook Studios',
				name: 'Darkest Dungeon®'
			},
			{
				appid: '108600',
				image_url:
					'https://shared.fastly.steamstatic.com/store_item_assets/steam/apps/108600/header.jpg?t=1762369969',
				description:
					'Project Zomboid is the ultimate in zombie survival. Alone or in MP: you loot, build, craft, fight, farm and fish in a struggle to survive. A hardcore RPG skillset, a vast map, massively customisable sandbox and a cute tutorial raccoon await the unwary. So how will you die? All it takes is a bite.. ',
				developer: 'The Indie Stone',
				name: 'Project Zomboid'
			}
		];
	});

	let isDirty = false;
	let isSaving = false;
	let saveError: string | null = null;

</script>

<div class="mx-auto my-8 max-w-6xl">
	<h1 class="mb-6 text-2xl font-bold">Property Management System</h1>

	<Tabs value={group} onValueChange={(e) => (group = e.value)}>
		{#snippet list()}
			<Tabs.Control value="properties">Properties</Tabs.Control>
			<Tabs.Control value="users">Users</Tabs.Control>
			<Tabs.Control value="apps">Apps</Tabs.Control>
		{/snippet}
		{#snippet content()}
			<!-- Properties Tab -->
			<Tabs.Panel value="properties">
				{@render propertiesPanel()}
			</Tabs.Panel>

			<!-- Users Tab -->
			<Tabs.Panel value="users">
				{@render usersPanel()}
			</Tabs.Panel>

			<!-- Apps Tab -->
			<Tabs.Panel value="apps">
				{@render appsPanel()}
			</Tabs.Panel>
		{/snippet}
	</Tabs>
</div>

{#snippet propertiesPanel()}
	<div class="mb-4 flex items-center justify-between">
		<input bind:value={searchTerm} placeholder="Search properties..." class="input w-64" />
		<select bind:value={statusFilter} class="select w-48">
			{#each statusOptions as option}
				<option value={option.value}>{option.label}</option>
			{/each}
		</select>
	</div>

	<table class="table">
		<thead>
		<tr>
			<th>Item ID</th>
			<th>Class</th>
			<th>Value</th>
			<th>Submitted By</th>
			<th>Status</th>
			<th>Actions</th>
		</tr>
		</thead>
		<tbody>
		{#each properties as property}
			<tr class="hover:preset-tonal-primary">
				<td><a class="anchor" href="/item/{property.in}">{property.in}</a></td>
				<td>{property.out.class}</td>
				<td>{property.out.value}</td>
				<td>{property.source}</td>
				<td>
					{#if property.status === -1}
						<span class="text-red-500">Denied</span>
					{:else if property.status === 0}
						<span class="text-yellow-500">Pending</span>
					{:else}
						<span class="text-green-500">Approved</span>
					{/if}
				</td>
				<td class="flex gap-2">
					<nav
						class="btn-group btn-sm preset-outlined-surface-200-800 flex-col p-2 md:flex-row"
					>
						<button
							type="button"
							class={[
												'btn btn-sm',
												property.status === -1 ? 'preset-filled' : 'hover:preset-tonal'
											]}
							onclick={() => togglePropertyStatus(property, -1)}
							disabled={property.status === -1}
						>Deny
						</button>
						<button
							type="button"
							class={[
												'btn btn-sm',
												property.status === 0 ? 'preset-filled' : 'hover:preset-tonal'
											]}
							onclick={() => togglePropertyStatus(property, 0)}
							disabled={property.status === 0}
						>Pending
						</button>
						<button
							type="button"
							class={[
												'btn btn-sm',
												property.status === 1 ? 'preset-filled' : 'hover:preset-tonal'
											]}
							onclick={() => togglePropertyStatus(property, 1)}
							disabled={property.status === 1}
						>Approve
						</button>
					</nav>
				</td>
			</tr>
		{/each}
		</tbody>
	</table>
{/snippet}

{#snippet usersPanel()}
	<table class="table">
		<thead>
		<tr>
			<th>ID</th>
			<th>Name</th>
			<th>Admin</th>
			<th>Banned</th>
			<th>Last Logged In</th>
		</tr>
		</thead>
		<tbody>
		{#each users as user}
			<tr class="hover:preset-tonal-primary">
				<td>{user.id}</td>
				<td>{user.name ?? "unpopulated" } </td>
				<td>
					<input
						type="checkbox"
						class="checkbox"
						checked={user.admin}
						onchange={(e) => toggleUserAdmin(user.id, e.target.checked)}
					/>
				</td>
				<td>
					<input
						type="checkbox"
						class="checkbox"
						checked={user.banned}
						onchange={(e) => {
											toggleUserBan(user.id, e.target.checked);
										}}
					/>
				</td>
				<td>
					{user.last_logged_in}
				</td>
			</tr>
		{/each}
		</tbody>
	</table>
{/snippet}

{#snippet appsPanel()}
	<AdminApps></AdminApps>
<!--	<div class="space-y-6 p-4">-->
<!--		<div class="flex items-center justify-between">-->
<!--			<h2 class="text-2xl font-bold">Games</h2>-->

<!--			<button-->
<!--				class="btn variant-filled-primary"-->
<!--				disabled={!isDirty || isSaving}-->
<!--				onclick={saveAll}-->
<!--			>-->
<!--				{isSaving ? "Saving…" : "Save Changes"}-->
<!--			</button>-->
<!--		</div>-->

<!--		{#if saveError}-->
<!--			<div class="alert variant-filled-error">-->
<!--				{saveError}-->
<!--			</div>-->
<!--		{/if}-->

<!--		{#each games as game, i (game.appid + i)}-->
<!--			<div class="card border p-4 space-y-4">-->
<!--				<div class="flex justify-between items-center">-->
<!--					<h3 class="text-lg font-semibold">-->
<!--						{game.name || "New Game"}-->
<!--					</h3>-->

<!--					<button-->
<!--						type="button"-->
<!--						class="btn variant-outline error"-->
<!--					>-->
<!--						Delete-->
<!--					</button>-->
<!--				</div>-->

<!--				<div class="grid grid-cols-1 sm:grid-cols-2 gap-4">-->
<!--					<div>-->
<!--						<label class="label">Name</label>-->
<!--						<input-->
<!--							class="input w-full"-->
<!--							bind:value={game.name}-->
<!--						/>-->
<!--					</div>-->

<!--					<div>-->
<!--						<label class="label">App ID</label>-->
<!--						<input-->
<!--							class="input w-full"-->
<!--							bind:value={game.appid}-->
<!--						/>-->
<!--					</div>-->

<!--					<div class="col-span-full">-->
<!--						<label class="label">Image URL</label>-->
<!--						<input-->
<!--							class="input w-full"-->
<!--							bind:value={game.image_url}-->
<!--						/>-->

<!--						{#if game.image_url}-->
<!--							<img-->
<!--								src={game.image_url}-->
<!--								alt={game.name}-->
<!--								class="mt-2 max-w-xs rounded"-->
<!--							/>-->
<!--						{/if}-->
<!--					</div>-->

<!--					<div>-->
<!--						<label class="label">Developer</label>-->
<!--						<input-->
<!--							class="input w-full"-->
<!--							bind:value={game.developer}-->
<!--						/>-->
<!--					</div>-->

<!--					<div class="col-span-full">-->
<!--						<label class="label">Description</label>-->
<!--						<textarea-->
<!--							class="textarea w-full"-->
<!--							bind:value={game.description}-->
<!--						/>-->
<!--					</div>-->
<!--				</div>-->
<!--			</div>-->
<!--		{/each}-->

<!--		<button-->
<!--			class="btn variant-outline w-full"-->
<!--		>-->
<!--			Add New Game-->
<!--		</button>-->
<!--	</div>-->
{/snippet}