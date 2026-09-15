<script lang="ts">
	import { faChevronLeft, faChevronRight, faEllipsis } from '@fortawesome/free-solid-svg-icons';
	import { Pagination } from '@skeletonlabs/skeleton-svelte';
	import Icon from 'svelte-awesome';
	import { SvelteMap } from 'svelte/reactivity';
	import { stringifySafeJSON } from '$lib/parser';

	type Property = {
		in: number | bigint;
		out: { class: string; value: string };
		source: { User?: number | bigint };
		status: number;
	};

	interface Props {
		properties: Property[];
	}

	let { properties }: Props = $props();

	const PENDING = 0;
	const PAGE_SIZE = 25;

	// One row per status: the badge, the button that sets it, and the button
	// class for both states.
	const statuses = [
		{
			value: -1,
			name: 'Denied',
			action: 'Deny',
			text: 'text-error-500',
			on: 'preset-filled-error-500',
			off: 'preset-tonal-error-500 hover:preset-filled-error-500'
		},
		{
			value: PENDING,
			name: 'Pending',
			action: 'Pending',
			text: 'text-warning-500',
			on: 'preset-filled-warning-500',
			off: 'preset-tonal-warning-500 hover:preset-filled-warning-500'
		},
		{
			value: 1,
			name: 'Approved',
			action: 'Approve',
			text: 'text-success-500',
			on: 'preset-filled-success-500',
			off: 'preset-tonal-success-500 hover:preset-filled-success-500'
		}
	];

	const filters = [{ value: 'all', label: 'All Statuses' }].concat(
		statuses.map((status) => ({ value: String(status.value), label: status.name }))
	);

	// A status change goes here and not into the row, so the filter and the sort
	// below keep each row in place until the next load.
	const changed = new SvelteMap<string, number>();

	let searchTerm = $state('');
	let statusFilter = $state('all');
	let page = $state(1);

	function key(property: Property) {
		return `${property.in}/${property.out.class}/${property.out.value}`;
	}

	function statusOf(property: Property) {
		return changed.get(key(property)) ?? property.status;
	}

	let filtered = $derived.by(() => {
		const needle = searchTerm.trim().toLowerCase();
		return properties
			.filter(
				(property) =>
					(statusFilter === 'all' || property.status === Number(statusFilter)) &&
					property.out.value.toLowerCase().includes(needle)
			)
			.toSorted((a, b) => Number(a.status !== PENDING) - Number(b.status !== PENDING));
	});

	let paginated = $derived(filtered.slice((page - 1) * PAGE_SIZE, page * PAGE_SIZE));

	async function setStatus(property: Property, status: number) {
		const previous = statusOf(property);
		changed.set(key(property), status);

		const res = await fetch('/api/admin/properties', {
			method: 'PUT',
			headers: { 'Content-Type': 'application/json' },
			body: stringifySafeJSON({
				item: property.in,
				class: property.out.class,
				value: property.out.value,
				status
			})
		});

		if (!res.ok) {
			changed.set(key(property), previous);
			console.error('Could not set the property status', res.status, res.statusText);
		}
	}
</script>

<div class="mb-4 flex items-center justify-between">
	<input
		value={searchTerm}
		oninput={(event) => {
			searchTerm = event.currentTarget.value;
			page = 1;
		}}
		placeholder="Search by value..."
		class="input w-64"
	/>
	<select
		value={statusFilter}
		onchange={(event) => {
			statusFilter = event.currentTarget.value;
			page = 1;
		}}
		class="select w-48"
	>
		{#each filters as filter (filter.value)}
			<option value={filter.value}>{filter.label}</option>
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
			{#each paginated as property (key(property))}
				{@const current = statusOf(property)}
				{@const badge = statuses.find((status) => status.value === current)}
				<tr class="hover:preset-tonal-primary">
					<td><a class="anchor" href="/item/{property.in}">{property.in}</a></td>
					<td>{property.out.class}</td>
					<td>{property.out.value}</td>
					<td>{property.source.User ?? 'System'}</td>
					<td><span class={badge?.text}>{badge?.name}</span></td>
					<td>
						<nav class="flex flex-col gap-2 p-2 md:flex-row">
							{#each statuses as status (status.value)}
								<button
									type="button"
									class="btn btn-sm {status.value === current ? status.on : status.off}"
									onclick={() => setStatus(property, status.value)}
									disabled={status.value === current}
								>
									{status.action}
								</button>
							{/each}
						</nav>
					</td>
				</tr>
			{:else}
				<tr><td colspan="6" class="text-surface-500">No properties match.</td></tr>
			{/each}
		</tbody>
	</table>
</div>

<Pagination
	{page}
	count={filtered.length}
	pageSize={PAGE_SIZE}
	onPageChange={(e) => (page = e.page)}
>
	<Pagination.PrevTrigger>
		<Icon data={faChevronLeft} class="fa-fw"></Icon>
	</Pagination.PrevTrigger>

	<Pagination.Context>
		{#snippet children(pagination)}
			{#each pagination().pages as item, index (item)}
				{#if item.type === 'page'}
					<Pagination.Item {...item}>
						{item.value}
					</Pagination.Item>
				{:else}
					<Pagination.Ellipsis {index}>
						<Icon data={faEllipsis} class="fa-fw"></Icon>
					</Pagination.Ellipsis>
				{/if}
			{/each}
		{/snippet}
	</Pagination.Context>

	<Pagination.NextTrigger>
		<Icon data={faChevronRight} class="fa-fw"></Icon>
	</Pagination.NextTrigger>
</Pagination>
