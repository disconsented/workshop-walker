<script lang="ts">
	import Icon from 'svelte-awesome';

	import type { PageData } from '../../../../.svelte-kit/types/src/routes';
	import { faSteamSymbol } from '@fortawesome/free-brands-svg-icons';
	import {
		fa1,
		faArrowLeft,
		faArrowRight,
		faCross,
		faEllipsis,
		faLink,
		faSearch
	} from '@fortawesome/free-solid-svg-icons';
	import { tags, orderBy, language } from './store.svelte';
	import { Pagination } from '@skeletonlabs/skeleton-svelte';


	let { data }: { data: PageData } = $props();


	console.log('Hello, wolrd!', data);
	$inspect(tags, orderBy, language);
	let viewMode = $state('grid');

	let page = $state(1);
	let size = $state(10);
	const slicedSource = $derived((s) => s.slice((page - 1) * size, page * size));
</script>
<div class="min-h-screen">
	<div class="max-w-7xl mx-auto px-4 py-8">
		{@render SearchPanel()}

		<div class="mt-6">
			<div class="flex gap-2 mb-4">
				<button
					class="btn {viewMode === 'table' ? 'preset-filled-primary-500' : 'preset-outlined-surface-500'} "
					onclick={() => viewMode = 'table'}
				>
					Table View
				</button>
				<button
					class="btn {viewMode === 'grid' ? 'preset-filled-primary-500' : 'preset-outlined-surface-500'}"
					onclick={() => viewMode = 'grid'}
				>
					Grid View
				</button>
			</div>

			<span>Results</span>
			{#if viewMode === "table"}
				{@render rTable()}
			{:else}
				{@render rgrid()}
			{/if}
		</div>
	</div>
</div>

{#snippet oldSearch()}
	<div class="card w-full max-w-md preset-filled-surface-100-900 p-4 text-center">
		<p>Filter Options</p>
		<form class="flex flex-col">
			<label class="label">
				<span class="label-text">Language</span>
				<select class="select" bind:value={language.v}>
					<option value="Russian">Russian</option>
					<option value="Chinese">Chinese</option>
					<option value="English">English</option>
				</select>
			</label>

			<label class="label">
				<span class="label-text">Order By</span>
				<select class="select" bind:value={orderBy.v}>
					<option value="votes">Votes</option>
					<option value="lastUpdated">Last Updated</option>
					<option value="alphabetical">Alphabetical</option>
				</select>
			</label>

			<label class="label">
				<span class="label-text">Updated Since</span>
				<input type="datetime-local">
			</label>

			<hr />
			<label class="label">
				<span class="label-text">Tags</span>
				<div class="space-y-2">
					{#each ["Mod",
						"Translation",
						"Scenario",
						"0.14",
						"0.15",
						"0.16",
						"0.17",
						"0.18",
						"0.19",
						"1.0",
						"1.1",
						"1.2",
						"1.3",
						"1.4",
						"1.5"] as tag}
						<label class="flex items-center space-x-2">
							<input name="tag" class="checkbox" type="checkbox" value="{tag}" bind:group={tags.v} />
							<p>{tag}</p>
						</label>
					{/each}
				</div>
				<!--			<button class="btn preset-filled">Clear</button>-->
			</label>
			<hr />
			<div class="input-group grid-cols-[auto_1fr_auto_auto]">
				<div class="ig-cell preset-tonal">
					<Icon data={faSearch} class="fa-fw"></Icon>
				</div>
				<input class="ig-input" type="search" placeholder="Search..." />
			</div>
			<div class="input-group grid-cols-[auto_auto]">
				<button class="ig-btn preset-filled">Submit</button>
				<button class="ig-btn preset-filled-warning-500" type="reset">Clear</button>
			</div>

		</form>
	</div>
{/snippet}

{#snippet SearchPanel()}
	<form class="card rounded-lg shadow p-6  preset-filled-surface-100-900 text-center">
		<div class="grid grid-cols-1 md:grid-cols-4 gap-4">
			<div>
				<label class="block text-sm font-medium mb-2">Title:</label>
				<input
					type="text"
					placeholder="Search by title"
					class="border rounded-lg px-3 py-2 w-full input"
				/>
			</div>

			<div>
				<label class="block text-sm font-medium mb-2">Updated Since:</label>
				<input
					type="datetime-local"
					class="border rounded-lg px-3 py-2 w-full input"
				/>
			</div>

			<div>
				<label class="block text-sm font-medium mb-2">Language:</label>
				<select
					class="border rounded-lg px-3 py-2 w-full select"
					bind:value={language.v}
				>
					<option value="English">English</option>
					<option value="Russian">Russian</option>
					<option value="Chinese">Chinese</option>
				</select>
			</div>

			<div>
				<label class="block text-sm font-medium mb-2">Order By:</label>
				<select
					class="border rounded-lg px-3 py-2 w-full select"
					bind:value={orderBy.v}
				>
					<option value="lastUpdated">Last Updated</option>
					<option value="alphabetical">Alphabetical</option>
					<option value="votes">Votes</option>
				</select>
			</div>

			<div class="md:col-span-4 flex flex-wrap gap-2">
				{#each ["Mod",
					"Translation",
					"Scenario",
					"0.14",
					"0.15",
					"0.16",
					"0.17",
					"0.18",
					"0.19",
					"1.0",
					"1.1",
					"1.2",
					"1.3",
					"1.4",
					"1.5"] as tag}
					<label class="flex items-center space-x-2">
						<input name="tag" class="checkbox" type="checkbox" value="{tag}" />
						<p>{tag}</p>
					</label>
				{/each}
			</div>

			<div class="md:col-span-full flex gap-4">
				<label class="block text-sm font-medium mb-2">Entries per page:</label>
				<input
					type="number"
					min="1"
					max="100"
					value={25}
					class="border rounded-lg px-3 py-2 w-24 input"
				/>
			</div>

			<div class="md:col-span-full flex gap-4">
				<button
					type="submit"
					class="ig-btn preset-filled"
				>
					Search
				</button>
				<button
					type="reset"
					class="ig-btn preset-filled-warning-500"
				>
					Reset
				</button>
			</div>
		</div>
	</form>

{/snippet}

{#snippet rTable()}
	<div class="rounded-lg shadow overflow-hidden table-wrap">
		<table class="table caption-bottom">
			<thead class="">
			<tr>
				<th class="px-6 py-3 text-left text-xs font-medium uppercase tracking-wider">Title</th>
				<th class="px-6 py-3 text-left text-xs font-medium uppercase tracking-wider">Author</th>
				<th class="px-6 py-3 text-left text-xs font-medium uppercase tracking-wider">Last Updated</th>
				<th class="px-6 py-3 text-left text-xs font-medium uppercase tracking-wider">Description</th>
			</tr>
			</thead>
			<tbody class="divide-y divide-gray-200 [&>tr]:hover:preset-tonal-primary">
			{#each slicedSource(data.result) as item(item.id)}
				<tr class="hover:bg-gray-50">
					<td class="px-6 py-4 text-sm">
						<a href="https://steamcommunity.com/sharedfiles/filedetails/?id={item.id}" target="_blank"
							 rel="noopener noreferrer"
							 class="">
							{item.title}
						</a>
						<br />
						<span class="text-xs text-gray-500">Lookup: <a href="/item/{item.id}" target="_blank"
																													 rel="noopener noreferrer"
																													 class="btn text-xs">Details <Icon data={faLink}
																																														 class="fa-fw"></Icon></a></span>
					</td>
					<td class="px-6 py-4 text-sm">
						{item.author}
						<br />
						<small class="text-gray-500">
							<a href="/item/{item.id}" target="_blank"
								 rel="noopener noreferrer"
								 class="">Details
								<Icon data={faLink}
											class="fa-fw"></Icon>
							</a>
						</small>
					</td>
					<td class="px-6 py-4 text-sm">{item.last_updated}</td>
					<td class="px-6 py-4 text-sm truncate">{item.description}</td>
				</tr>
			{:else}
				<tr>
					<td colspan="4" class="px-6 py-4 text-center text-gray-500">No results found</td>
				</tr>
			{/each}
			</tbody>
		</table>

		<footer class="flex justify-between">
			<select name="size" id="size" class="select max-w-[150px]" value={size}
							onchange={(e) => (size = Number(e.currentTarget.value))}>
				{#each [5, 10, 15, 30] as v}
					<option value={v}>Items {v}</option>
				{/each}
				<option value={data.result.length}>Show All</option>
			</select>
			<!-- Pagination -->
			<Pagination
				data={data.result}
				{page}
				onPageChange={(e) => (page = e.page)}
				pageSize={size}
				onPageSizeChange={(e) => (size = e.pageSize)}
				siblingCount={4}
			>
				{#snippet labelEllipsis()}
					<Icon data={faEllipsis} class="fa-fw"></Icon>
				{/snippet}
				{#snippet labelNext()}
					<Icon data={faArrowRight} class="fa-fw"></Icon>
				{/snippet}
				{#snippet labelPrevious()}
					<Icon data={faArrowLeft} class="fa-fw"></Icon>
				{/snippet}
				{#snippet labelFirst()}
					<Icon data={fa1} class="fa-fw"></Icon>
				{/snippet}
				{#snippet labelLast()}
					<Icon data={faCross} class="fa-fw"></Icon>
				{/snippet}
			</Pagination>
		</footer>
	</div>

{/snippet}

{#snippet rgrid()}
	<div class="grid grid-cols-1 md:grid-cols-3 lg:grid-cols-4 gap-4">
		{#each slicedSource(data.result) as item(item.id)}
			<div
				class="card preset-filled-surface-100-900 border-[1px] border-surface-200-800 card-hover divide-surface-200-800 block max-w-md divide-y overflow-hidden">
				<header>
					<img
						src={item.preview_url || 'https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/294100/header.jpg?t=1734154189'}
						class="w-full h-48 object-cover w-full" alt="banner" class:hue-rotate-90={!item.preview_url}
						class:grayscale={!item.preview_url} />
				</header>
				<article class="space-y-4 p-4">
					<h6 class="h6">
						<a href="/item/{item.id}" target="_blank"
							 rel="noopener noreferrer"
							 class="">
							{item.title}
							<Icon data={faLink} class="fa-fw"></Icon>
						</a>
					</h6>
					<div class="flex justify-between items-center mb-2">
						<span class="text-sm text-gray-500">by {item.author}</span>
						<small class="text-xs text-gray-500">
							<a href="https://steamcommunity.com/sharedfiles/filedetails/?id={item.id}" target="_blank"
								 rel="noopener noreferrer" class="hover:text-gray-700">Steam
								<Icon data={faSteamSymbol} class="fa-fw"></Icon>
							</a>
						</small>
					</div>
					<p class="text-sm text-gray-600 truncate mb-2">{item.description}</p>
				</article>
				<footer class="flex gap-1 m-2 flex-wrap">
					{#each item.tags as tag(tag.id)}
						<button type="button" class="chip preset-filled">{tag.display_name}</button>
					{:else}
						<button type="button" class="chip preset-filled">-</button>
					{/each}
				</footer>
			</div>
		{:else}
			<div class="text-center text-gray-500 py-8">
				No results found
			</div>
		{/each}
	</div>


	<footer class="flex justify-between">
		<select name="size" id="size" class="select max-w-[150px]" value={size}
						onchange={(e) => (size = Number(e.currentTarget.value))}>
			{#each [5, 10, 15, 30] as v}
				<option value={v}>Items {v}</option>
			{/each}
			<option value={data.result.length}>Show All</option>
		</select>
		<!-- Pagination -->
		<Pagination
			data={data.result}
			{page}
			onPageChange={(e) => (page = e.page)}
			pageSize={size}
			onPageSizeChange={(e) => (size = e.pageSize)}
			siblingCount={4}
		>
			{#snippet labelEllipsis()}
				<Icon data={faEllipsis} class="fa-fw"></Icon>
			{/snippet}
			{#snippet labelNext()}
				<Icon data={faArrowRight} class="fa-fw"></Icon>
			{/snippet}
			{#snippet labelPrevious()}
				<Icon data={faArrowLeft} class="fa-fw"></Icon>
			{/snippet}
			{#snippet labelFirst()}
				<Icon data={fa1} class="fa-fw"></Icon>
			{/snippet}
			{#snippet labelLast()}
				<Icon data={faCross} class="fa-fw"></Icon>
			{/snippet}
		</Pagination>
	</footer>
{/snippet}