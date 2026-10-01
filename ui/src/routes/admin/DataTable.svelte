<script lang="ts" generics="TData extends RowData">
	import { Pagination } from '@skeletonlabs/skeleton-svelte';
	import Icon from 'svelte-awesome';
	import { faChevronLeft, faChevronRight, faEllipsis } from '@fortawesome/free-solid-svg-icons';
	import {
		FlexRender,
		type PaginationState,
		type RowData,
		type SvelteTable
	} from '@tanstack/svelte-table';
	import { features } from './tableFeatures';

	let {
		table,
		pagination
	}: {
		table: SvelteTable<typeof features, TData>;
		/** The table's controlled pagination state. */
		pagination: PaginationState;
	} = $props();

	const sortIndicator: Record<string, string> = { asc: '▲', desc: '▼', false: '' };
</script>

<div class="table-wrap">
	<table class="table table-zebra">
		<thead>
			{#each table.getHeaderGroups() as group (group.id)}
				<tr>
					{#each group.headers as header (header.id)}
						<th>
							{#if !header.isPlaceholder}
								{#if header.column.getCanSort()}
									<button
										type="button"
										class="cursor-pointer select-none"
										onclick={header.column.getToggleSortingHandler()}
									>
										<FlexRender {header} />
										{sortIndicator[`${header.column.getIsSorted()}`]}
									</button>
								{:else}
									<FlexRender {header} />
								{/if}
							{/if}
						</th>
					{/each}
				</tr>
			{/each}
		</thead>
		<tbody>
			{#each table.getRowModel().rows as row (row.id)}
				<tr class="hover:preset-tonal-primary">
					{#each row.getAllCells() as cell (cell.id)}
						<td><FlexRender {cell} /></td>
					{/each}
				</tr>
			{/each}
		</tbody>
	</table>
</div>

<!-- Skeleton counts pages from 1, TanStack from 0. -->
<div class="mt-4 flex items-center justify-between">
	<div class="flex items-center gap-2">
		<span class="shrink-0 text-sm opacity-60">Per page</span>
		<select
			class="select"
			value={pagination.pageSize}
			onchange={(e) => table.setPageSize(Number(e.currentTarget.value))}
		>
			{#each [10, 25, 50, 100] as size (size)}
				<option value={size}>{size}</option>
			{/each}
		</select>
	</div>
	<Pagination
		page={pagination.pageIndex + 1}
		count={table.getPrePaginatedRowModel().rows.length}
		pageSize={pagination.pageSize}
		onPageChange={(event) => table.setPageIndex(event.page - 1)}
	>
		<Pagination.PrevTrigger>
			<Icon data={faChevronLeft} class="fa-fw" />
		</Pagination.PrevTrigger>
		<Pagination.Context>
			{#snippet children(context)}
				{#each context().pages as page, index (page)}
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
</div>
