<script lang="ts">
	import { Tabs } from '@skeletonlabs/skeleton-svelte';

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
</script>

<div class="mx-auto my-8 max-w-6xl">
	<h1 class="mb-6 text-2xl font-bold">Property Management System</h1>

	<Tabs value={group} onValueChange={(e) => (group = e.value)}>
		{#snippet list()}
			<Tabs.Control value="properties">Properties</Tabs.Control>
			<Tabs.Control value="users">Users</Tabs.Control>
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
			{@debug property}
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