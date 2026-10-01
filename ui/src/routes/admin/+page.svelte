<script lang="ts">
	import { Tabs } from '@skeletonlabs/skeleton-svelte';
	import AdminApps from './AdminApps.svelte';
	import DataTable from './DataTable.svelte';
	import {
		type ColumnDef,
		createTable,
		type PaginationState,
		renderSnippet,
		type Updater
	} from '@tanstack/svelte-table';
	import { features } from './tableFeatures';
	import TimeAgo from '$lib/timeAgo.svelte';

	let { data }: { data } = $props();
	let properties: Property[] = $state([]);

	let users: User[] = $state([]);
	data.users.then((data) => (users = data ?? []));
	// Assign a new array: the table only recomputes rows when the data reference changes.
	data.properties.then((data) => (properties = data ?? []));

	let searchTerm = $state('');
	let statusFilter = $state('all');
	// The table compares controlled state by reference. A new array on each read looks like a
	// filter change, and a filter change resets the page index to 0.
	const columnFilters = $derived(
		statusFilter === 'all' ? [] : [{ id: 'status', value: Number(statusFilter) }]
	);
	let pagination: PaginationState = $state({ pageIndex: 0, pageSize: 10 });
	let userPagination: PaginationState = $state({ pageIndex: 0, pageSize: 10 });

	// Status options
	const statusOptions = [
		{ value: '-1', label: 'Denied' },
		{ value: '0', label: 'Pending' },
		{ value: '1', label: 'Approved' },
		{ value: 'all', label: 'All Statuses' }
	];

	// Toggle functions
	async function togglePropertyStatus(index: number, status: number) {
		// Replace the array so the table sees new data and recomputes its cached values.
		const prop = { ...properties[index], status };
		properties = properties.with(index, prop);

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
		users = users.map((u) => (u.id === id ? { ...u, admin: value } : u));

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
		users = users.map((u) => (u.id === id ? { ...u, banned: value } : u));
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
	type Source = 'System' | { User: string };

	type Property = {
		in: string;
		out: { class: string; value: string };
		note: string | null;
		status: number;
		upvote_count: number;
		vote_count: number;
		source: Source;
		vote_state: number | null;
	};

	const columns: Array<ColumnDef<typeof features, Property>> = [
		{
			accessorKey: 'in',
			header: 'Item ID',
			sortFn: 'alphanumeric',
			cell: (info) => renderSnippet(itemLink, info.row.original.in)
		},
		{
			accessorFn: (row) => row.out.class,
			id: 'class',
			header: 'Class',
			sortFn: 'alphanumeric'
		},
		{
			accessorFn: (row) => row.out.value,
			id: 'value',
			header: 'Value',
			sortFn: 'alphanumeric'
		},
		{
			accessorFn: (row) => (row.source === 'System' ? 'System' : row.source.User),
			id: 'source',
			header: 'Submitted By',
			sortFn: 'alphanumeric'
		},
		{
			accessorKey: 'status',
			header: 'Status',
			filterFn: 'equals',
			sortFn: 'basic',
			enableGlobalFilter: false,
			cell: (info) => renderSnippet(statusLabel, info.row.original.status)
		},
		{
			id: 'actions',
			header: 'Actions',
			cell: (info) => renderSnippet(statusActions, info.row.index)
		}
	];

	const propertyTable = createTable({
		features,
		columns,
		get data() {
			return properties;
		},
		globalFilterFn: 'includesString',
		// The search box and status select own the filter state; the table only reads it.
		state: {
			get globalFilter() {
				return searchTerm;
			},
			get columnFilters() {
				return columnFilters;
			},
			get pagination() {
				return pagination;
			}
		},
		// The table resets the page index when the filters change, so write its updates back.
		onPaginationChange: (next: Updater<PaginationState>) => {
			pagination = typeof next === 'function' ? next(pagination) : next;
		}
	});

	type User = {
		id: number;
		username?: { id: number; name: string };
		admin: boolean;
		banned: boolean;
		last_logged_in: string;
	};

	const userColumns: Array<ColumnDef<typeof features, User>> = [
		{
			cell: (info) => renderSnippet(userLink, info.row.original.id),
			accessorKey: 'id',
			header: 'ID',
			sortFn: 'basic'
		},
		{
			accessorFn: (row) => row.username?.name ?? 'unpopulated',
			id: 'name',
			header: 'Name',
			sortFn: 'alphanumeric'
		},
		{
			accessorKey: 'admin',
			header: 'Admin',
			sortFn: 'basic',
			cell: (info) => renderSnippet(adminToggle, info.row.original)
		},
		{
			accessorKey: 'banned',
			header: 'Banned',
			sortFn: 'basic',
			cell: (info) => renderSnippet(banToggle, info.row.original)
		},
		{
			cell: (info) => renderSnippet(lastLoggedIn, info.row.original.last_logged_in),
			accessorKey: 'last_logged_in',
			header: 'Last Logged In',
			sortFn: 'basic'
		}
	];

	const userTable = createTable({
		features,
		columns: userColumns,
		get data() {
			return users;
		},
		state: {
			get pagination() {
				return userPagination;
			}
		},
		onPaginationChange: (next: Updater<PaginationState>) => {
			userPagination = typeof next === 'function' ? next(userPagination) : next;
		}
	});
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
			{#each statusOptions as option (option.value)}
				<option value={option.value}>{option.label}</option>
			{/each}
		</select>
	</div>

	<DataTable table={propertyTable} {pagination} />
{/snippet}

{#snippet itemLink(id: string)}
	<a class="anchor" href="/item/{id}">{id}</a>
{/snippet}

{#snippet statusLabel(status: number)}
	{#if status === -1}
		<span class="text-error-500">Denied</span>
	{:else if status === 0}
		<span class="text-warning-500">Pending</span>
	{:else}
		<span class="text-success-500">Approved</span>
	{/if}
{/snippet}

{#snippet statusActions(index: number)}
	{@const status = properties[index].status}
	<nav class="flex-col gap-2 p-2 md:flex-row">
		<button
			type="button"
			class="btn btn-sm {status === -1
				? 'preset-filled-error-500'
				: 'preset-tonal-error-500 hover:preset-filled-error-500'}"
			onclick={() => togglePropertyStatus(index, -1)}
			disabled={status === -1}
		>
			Deny
		</button>
		<button
			type="button"
			class="btn btn-sm {status === 0
				? 'preset-filled-warning-500'
				: 'preset-tonal-warning-500 hover:preset-filled-warning-500'}"
			onclick={() => togglePropertyStatus(index, 0)}
			disabled={status === 0}
		>
			Pending
		</button>
		<button
			type="button"
			class="btn btn-sm {status === 1
				? 'preset-filled-success-500'
				: 'preset-tonal-success-500 hover:preset-filled-success-500'}"
			onclick={() => togglePropertyStatus(index, 1)}
			disabled={status === 1}
		>
			Approve
		</button>
	</nav>
{/snippet}

{#snippet usersPanel()}
	<DataTable table={userTable} pagination={userPagination} />
{/snippet}

{#snippet adminToggle(user: User)}
	<input
		type="checkbox"
		class="checkbox"
		checked={user.admin}
		onchange={(e) => toggleUserAdmin(user.id, e.currentTarget.checked)}
	/>
{/snippet}

{#snippet banToggle(user: User)}
	<input
		type="checkbox"
		class="checkbox"
		checked={user.banned}
		onchange={(e) => toggleUserBan(user.id, e.currentTarget.checked)}
	/>
{/snippet}

{#snippet appsPanel()}
	<AdminApps></AdminApps>
{/snippet}

{#snippet userLink(id: string)}
	<a class="anchor" href="https://steamcommunity.com/profiles/{id}">{id}</a>
{/snippet}

{#snippet lastLoggedIn(dateTime: string)}
	<span class="capitalize">
		<TimeAgo date={Math.floor(new Date(dateTime).getTime() / 1000)} />
	</span>
{/snippet}
