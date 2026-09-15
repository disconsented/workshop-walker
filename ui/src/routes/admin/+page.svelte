<script lang="ts">
	import { Tabs } from '@skeletonlabs/skeleton-svelte';
	import { onMount } from 'svelte';
	import AdminApps from './AdminApps.svelte';
	import timeAgo from '$lib/timeAgo.svelte';
	import TimeAgo from '$lib/timeAgo.svelte';

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

	export type Game = {
		appid: string;
		image_url: string;
		description: string;
		developer: string;
		name: string;
	};

	let games: Game[] = $state([]);

	let isDirty = false;
	let isSaving = false;
	let saveError: string | null = null;
</script>

<div class="mx-auto my-8 max-w-6xl">
	<h1 class="mb-6 text-2xl font-bold">Property Management System</h1>

	<Tabs value={group} onValueChange={(e) => (group = e.value)}>
		<Tabs.List>
			<Tabs.Trigger value="properties">Properties</Tabs.Trigger>
			<Tabs.Trigger value="users">Users</Tabs.Trigger>
			<Tabs.Trigger value="apps">Apps</Tabs.Trigger>
			<Tabs.Indicator />
		</Tabs.List>
		<!-- Properties Tab -->
		<Tabs.Content value="properties">
			{@render propertiesPanel()}
		</Tabs.Content>

		<!-- Users Tab -->
		<Tabs.Content value="users">
			{@render usersPanel()}
		</Tabs.Content>

		<!-- Apps Tab -->
		<Tabs.Content value="apps">
			{@render appsPanel()}
		</Tabs.Content>
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

	<div class="table-wrap">
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
					<td>
						{#if property.source.User}{property.source.User}{:else}{property.source}{/if}
					</td>
					<td>
						{#if property.status === -1}
							<span class="text-error-500">Denied</span>
						{:else if property.status === 0}
							<span class="text-warning-500">Pending</span>
						{:else}
							<span class="text-success-500">Approved</span>
						{/if}
					</td>
					<td class="flex gap-2">
						<nav class="flex-col gap-2 p-2 md:flex-row">
							<button
								type="button"
								class="btn btn-sm {property.status === -1
										? 'preset-filled-error-500'
										: 'preset-tonal-error-500 hover:preset-filled-error-500'}"
								onclick={() => togglePropertyStatus(property, -1)}
								disabled={property.status === -1}
							>
								Deny
							</button>
							<button
								type="button"
								class="btn btn-sm {property.status === 0
										? 'preset-filled-warning-500'
										: 'preset-tonal-warning-500 hover:preset-filled-warning-500'}"
								onclick={() => togglePropertyStatus(property, 0)}
								disabled={property.status === 0}
							>
								Pending
							</button>
							<button
								type="button"
								class="btn btn-sm {property.status === 1
										? 'preset-filled-success-500'
										: 'preset-tonal-success-500 hover:preset-filled-success-500'}"
								onclick={() => togglePropertyStatus(property, 1)}
								disabled={property.status === 1}
							>
								Approve
							</button>
						</nav>
					</td>
				</tr>
			{/each}
			</tbody>
		</table>
	</div>
{/snippet}

{#snippet usersPanel()}
	<div class="table-wrap">
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
					<td>{user.name ?? 'unpopulated'} </td>
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
						{new Date(user.last_logged_in)}
					</td>
				</tr>
			{/each}
			</tbody>
		</table>
	</div>
{/snippet}

{#snippet appsPanel()}
	<AdminApps/>
{/snippet}
