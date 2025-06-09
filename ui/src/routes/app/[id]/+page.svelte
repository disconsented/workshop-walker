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
		faSearch,
		faTriangleExclamation
	} from '@fortawesome/free-solid-svg-icons';
	import { tags, orderBy, language, limit, title } from './store.svelte';

	import { Pagination } from '@skeletonlabs/skeleton-svelte';
	import TimeAgo from '$lib/timeAgo.svelte';
	import TimePicker from '$lib/timePicker.svelte';
	import { Shadow } from 'svelte-loading-spinners';
	import { invalidate } from '$app/navigation';

	let { data }: { data: PageData } = $props();

	console.log(data);
	let storeTags = tags; // Ugly hack to work around svelte folks not actually fixing https://github.com/sveltejs/svelte/issues/15037
	let viewMode = $state('grid');

	let page = $state(1);
	let size = $state(15);
	const slicedSource = $derived((s) => s.slice((page - 1) * size, page * size));

	function runSearch(e) {
		e.preventDefault();
		invalidate((url) => {
			return url.pathname === '/api/list';
		});
	}
</script>

<svelte:head>
	<title>Workshop Walker - Search</title>
	<meta property="og:title" content="Workshop Walker - Search" />
	<meta property="og:type" content="website" />
	<meta property="og:url" content={window.location.href} />
</svelte:head>

{#await data.req}
	<div class="flex h-full w-full place-content-center">
		<Shadow></Shadow>
	</div>
{:then value}
	{#if value.status}
		{@render errorCard(value)}
	{:else}
		<div class="min-h-screen">
			<div class="mx-auto px-4 py-8">
				{@render SearchPanel()}
				<div class="mt-6">
					<div class="mb-4 flex gap-2">
						<button
							class="btn {viewMode === 'table'
								? 'preset-filled-primary-500'
								: 'preset-outlined-surface-500'} "
							onclick={() => (viewMode = 'table')}
						>
							Table View
						</button>
						<button
							class="btn {viewMode === 'grid'
								? 'preset-filled-primary-500'
								: 'preset-outlined-surface-500'}"
							onclick={() => (viewMode = 'grid')}
						>
							Grid View
						</button>
					</div>

					<div class="flex flex-row place-content-between">
						<span>{value.length} Result(s)</span>
						<div>{@render pagination({ data: value })}</div>
					</div>

					{#if viewMode === 'table'}
						{@render rTable(value)}
					{:else}
						{@render rgrid(value)}
					{/if}
				</div>
			</div>
		</div>
	{/if}
{:catch error}
	{@render errorCard(error)}
{/await}

{#snippet SearchPanel()}
	<form class="card preset-filled-surface-100-900 rounded-lg p-6 text-center shadow">
		<div class="grid grid-cols-1 gap-4 md:grid-cols-4">
			<div>
				<span class="mb-2 block text-sm font-medium">Title:</span>
				<input
					type="text"
					placeholder="Search by title"
					class="input w-full rounded-lg border px-3 py-2"
					bind:value={title.v}
				/>
			</div>

			<div>
				<span class="mb-2 block text-sm font-medium">Updated Since:</span>
				<TimePicker></TimePicker>
			</div>

			<div>
				<span class="mb-2 block text-sm font-medium">Language:</span>
				<select class="select w-full rounded-lg border px-3 py-2" bind:value={language.v}>
					<option>None</option>
					<option value="1">English</option>
					<option value="2">Russian</option>
					<option value="3">Chinese</option>
					<option value="4">Japanese</option>
					<option value="5">Korean</option>
					<option value="6">Spanish</option>
					<option value="7">Portuguese</option>
				</select>
			</div>

			<div>
				<span class="mb-2 block text-sm font-medium">Order By:</span>
				<select class="select w-full rounded-lg border px-3 py-2" bind:value={orderBy.v}>
					<option value="LastUpdated">Last Updated</option>
					<option value="Alphabetical">Alphabetical</option>
					<option value="Score">Score</option>
					<option value="Dependents">Dependents</option>
				</select>
			</div>

			<div class="flex flex-wrap gap-2 md:col-span-4">
				<!--ToDo: Load tags from backend-->
				{#each ['Mod', 'Translation', 'Scenario', '0.14', '0.15', '0.16', '0.17', '0.18', '0.19', '1.0', '1.1', '1.2', '1.3', '1.4', '1.5'] as tag}
					<span class="flex items-center space-x-2">
						<input
							name="tag"
							class="checkbox"
							type="checkbox"
							value={tag}
							bind:group={storeTags.v}
						/>
						<p>{tag}</p>
					</span>
				{/each}
			</div>

			<div class="flex gap-4 md:col-span-full">
				<span class="mb-2 block text-sm font-medium">Limit:</span>
				<input
					type="number"
					min="1"
					max="100"
					bind:value={limit.v}
					class="input w-24 rounded-lg border px-3 py-2"
				/>
			</div>

			<div class="flex gap-4 md:col-span-full">
				<button type="submit" class="ig-btn preset-filled" onclick={runSearch}> Search</button>
				<button type="reset" class="ig-btn preset-filled-warning-500"> Reset</button>
			</div>
		</div>
	</form>
{/snippet}

{#snippet rTable(data)}
	<div class="table-wrap overflow-hidden rounded-lg shadow">
		<table class="table caption-bottom">
			<thead class="">
				<tr>
					<th class="px-6 py-3 text-left text-xs font-medium tracking-wider uppercase">Title</th>
					<th class="px-6 py-3 text-left text-xs font-medium tracking-wider uppercase">Author</th>
					<th class="px-6 py-3 text-left text-xs font-medium tracking-wider uppercase"
						>Last Updated
					</th>
					<th class="px-6 py-3 text-left text-xs font-medium tracking-wider uppercase"
						>Description
					</th>
				</tr>
			</thead>
			<tbody class="[&>tr]:hover:preset-tonal-primary divide-y divide-gray-200">
				{#each slicedSource(data) as item (item.id)}
					<tr class="hover:bg-gray-50">
						<td class="px-6 py-4 text-sm">
							<a
								href="https://steamcommunity.com/sharedfiles/filedetails/?id={item.id}"
								target="_blank"
								rel="noopener noreferrer"
								class=""
							>
								{item.title}
							</a>
							<br />
							<span class="text-xs text-gray-500"
								>Lookup: <a
									href="/item/{item.id}"
									target="_self"
									rel="noopener noreferrer"
									class="btn text-xs">Details <Icon data={faLink} class="fa-fw"></Icon></a
								></span
							>
						</td>
						<td class="px-6 py-4 text-sm">
							<a href="https://steamcommunity.com/profiles/{item.author}" class="anchor">
								<Icon data={faSteamSymbol} class="fa-fw"></Icon>
								Author
							</a>
							<br />
							<small class="text-gray-500">
								<a href="/item/{item.id}" target="_self" rel="noopener noreferrer" class=""
									>Details
									<Icon data={faLink} class="fa-fw"></Icon>
								</a>
							</small>
						</td>
						<td class="px-6 py-4 text-sm">
							<TimeAgo date={item.last_updated}></TimeAgo>
						</td>
						<td class="truncate px-6 py-4 text-sm">{item.description}</td>
					</tr>
				{:else}
					<tr>
						<td colspan="4" class="px-6 py-4 text-center text-gray-500">No results found</td>
					</tr>
				{/each}
			</tbody>
		</table>

		<footer class="flex justify-between">
			<select
				name="size"
				id="size"
				class="select max-w-[150px]"
				value={size}
				onchange={(e) => (size = Number(e.currentTarget.value))}
			>
				{#each [5, 10, 15, 30] as v}
					<option value={v}>Items {v}</option>
				{/each}
				<option value={data.length}>Show All</option>
			</select>
			{@render pagination({ data: data })}
		</footer>
	</div>
{/snippet}

{#snippet rgrid(data)}
	<div class="flex flex-wrap place-content-center gap-4">
		{#each slicedSource(data) as item (item.id)}
			<div
				class="card preset-filled-surface-100-900 border-surface-200-800 card-hover divide-surface-200-800 block max-w-md divide-y overflow-hidden border-[1px]"
			>
				<header>
					<img
						src={item.preview_url ||
							'https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/294100/header.jpg?t=1734154189'}
						class="h-48 w-full w-full object-cover"
						alt="banner"
						class:hue-rotate-90={!item.preview_url}
						class:grayscale={!item.preview_url}
					/>
				</header>
				<article class="space-y-4 p-4">
					<h6 class="h6">
						<a href="/item/{item.id}" target="_self" rel="noopener noreferrer" class="hover:anchor">
							{item.title}
							<Icon data={faLink} class="fa-fw"></Icon>
						</a>
					</h6>
					<div class="mb-2 flex items-center justify-between">
						<span class="text-sm text-gray-500"
							>Updated: <TimeAgo date={item.last_updated}></TimeAgo></span
						>
						<small class="text-xs text-gray-500">
							<a
								href="https://steamcommunity.com/sharedfiles/filedetails/?id={item.id}"
								target="_blank"
								rel="noopener noreferrer"
								class="anchor hover:text-gray-700"
								>Steam
								<Icon data={faSteamSymbol} class="fa-fw"></Icon>
							</a>
						</small>
					</div>
					<p class="mb-2 truncate text-sm text-gray-600">{item.description}</p>
				</article>
				<footer class="m-2 flex flex-wrap gap-1">
					{#each item.tags as tag (tag.id)}
						<span class="badge preset-filled">{tag.display_name}</span>
					{:else}
						<span class="badge preset-filled">-</span>
					{/each}
				</footer>
			</div>
		{:else}
			<div class="text-center text-gray-500 py-8">No results found</div>
		{/each}
	</div>

	<footer class="flex justify-between">
		<select
			name="size"
			id="size"
			class="select max-w-[150px]"
			value={size}
			onchange={(e) => (size = Number(e.currentTarget.value))}
		>
			{#each [5, 10, 15, 30] as v}
				<option value={v}>Items {v}</option>
			{/each}
			<option value={data.length}>Show All</option>
		</select>
		{@render pagination({ data: data })}
	</footer>
{/snippet}

{#snippet pagination(obj)}
	<!-- Pagination -->
	<Pagination
		data={obj.data}
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
{/snippet}

{#snippet errorCard(value)}
	<div
		class="card preset-outlined-error-500 grid grid-cols-1 items-center gap-4 p-4 lg:grid-cols-[auto_1fr_auto]"
	>
		<Icon data={faTriangleExclamation} class="fa-fw"></Icon>
		<div>
			{#if value.status}
				<p class="font-bold">Error Code: {value.status}</p>
			{/if}
			{#if value.statusText}
				<p class="text-xs opacity-60">{value.statusText}</p>
			{/if}

			{#if value.body}
				<pre class="text-xs opacity-60">{value.body}</pre>
			{/if}

			{#if value.message}
				<p class="text-xs opacity-60">{value.message}</p>
			{/if}
		</div>
	</div>
{/snippet}
