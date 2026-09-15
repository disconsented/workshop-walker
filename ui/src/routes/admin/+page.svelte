<script lang="ts">
	import { Pagination, Tabs } from '@skeletonlabs/skeleton-svelte';
	import { faChevronLeft, faChevronRight, faEllipsis } from '@fortawesome/free-solid-svg-icons';
	import Icon from 'svelte-awesome';
	import { onMount } from 'svelte';
	import AdminApps from './AdminApps.svelte';
	import timeAgo from '$lib/timeAgo.svelte';
	import TimeAgo from '$lib/timeAgo.svelte';

	type PropertyRow = {
		in: string;
		out: { class: string; value: string };
		source: { User?: string };
		status: number;
	};

	type UserRow = {
		id: number;
		name?: string;
		admin: boolean;
		banned: boolean;
		last_logged_in: string;
	};

	let { data }: { data } = $props();
	console.log(data);
	let properties: PropertyRow[] = $state([]);

	let users: UserRow[] = $state([]);
	data.users.then((data) => users.push(...data));
	data.properties.then((data) => properties.push(...data));

	let searchTerm = $state('');
	let statusFilter = $state('all');

	// Status options
	const statusOptions = [
		{ value: 'all', label: 'All Statuses' },
		{ value: '0', label: 'Pending' },
		{ value: '1', label: 'Approved' },
		{ value: '-1', label: 'Denied' }
	];

	let filteredProperties = $derived.by(() => {
		const needle = searchTerm.trim().toLowerCase();
		return properties
			.filter((property) => {
				if (statusFilter !== 'all' && property.status !== Number(statusFilter)) {
					return false;
				}
				if (needle === '') {
					return true;
				}
				return String(property.out?.value ?? '')
					.toLowerCase()
					.includes(needle);
			})
			.toSorted((a, b) => (a.status === 0 ? 0 : 1) - (b.status === 0 ? 0 : 1));
	});

	let propertiesPage = $state(1);
	let propertiesPageSize = $state(25);
	let paginatedProperties = $derived(
		filteredProperties.slice(
			(propertiesPage - 1) * propertiesPageSize,
			propertiesPage * propertiesPageSize
		)
	);

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
		<input
			value={searchTerm}
			oninput={(event) => {
				searchTerm = event.currentTarget.value;
				propertiesPage = 1;
			}}
			placeholder="Search by value..."
			class="input w-64"
		/>
		<select
			value={statusFilter}
			onchange={(event) => {
				statusFilter = event.currentTarget.value;
				propertiesPage = 1;
			}}
			class="select w-48"
		>
			{#each statusOptions as option (option.value)}
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
				{#each paginatedProperties as property (`${property.in}/${property.out.class}/${property.out.value}`)}
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

	<Pagination
		page={propertiesPage}
		count={filteredProperties.length}
		pageSize={propertiesPageSize}
		onPageChange={(event) => (propertiesPage = event.page)}
	>
		<Pagination.PrevTrigger>
			<Icon data={faChevronLeft} class="fa-fw"></Icon>
		</Pagination.PrevTrigger>

		<Pagination.Context>
			{#snippet children(pagination)}
				{#each pagination().pages as page, index (page)}
					{#if page.type === 'page'}
						<Pagination.Item {...page}>
							{page.value}
						</Pagination.Item>
					{:else}
						<Pagination.Ellipsis {index}>
							<Icon data={faEllipsis} class="fa-fw" />
						</Pagination.Ellipsis>
					{/if}
				{/each}
			{/snippet}
		</Pagination.Context>
		<Pagination.NextTrigger>
			<Icon data={faChevronRight} class="fa-fw" />
		</Pagination.NextTrigger>
	</Pagination>
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
	<AdminApps />
{/snippet}
