<script lang="ts">
	import Icon from 'svelte-awesome';

	import type { PageData } from '../../../../.svelte-kit/types/src/routes';
	import { faSteamSymbol } from '@fortawesome/free-brands-svg-icons';
	import {
		fa1,
		faArrowLeft,
		faArrowRight,
		faChevronLeft,
		faChevronRight,
		faCross,
		faEllipsis,
		faGrip,
		faLink,
		faTableList,
		faTriangleExclamation
	} from '@fortawesome/free-solid-svg-icons';
	import { app, language, limit, orderBy, tags, title } from './store.svelte';
	import ItemCard from './itemCard.svelte';

	import { Pagination, SegmentedControl, Switch } from '@skeletonlabs/skeleton-svelte';
	import TimeAgo from '$lib/timeAgo.svelte';
	import TimePicker from '$lib/timePicker.svelte';
	import { Shadow } from 'svelte-loading-spinners';
	import { invalidate } from '$app/navigation';
	import Search from './search.svelte';

	let { data }: { data: PageData } = $props();

	$inspect(tags.v, app.v);
	if (tags.v.length === 0 && app.v.tags) {
		tags.v = app.v.tags.filter((tag) => app.v.default_tags.some((e) => e === tag));
	}

	let viewMode = $state('grid');
	let showTableImages = $state(false);

	let page = $state(1);
	let pageSize = $state(15);
	const slicedSource = $derived((s) => s.slice((page - 1) * pageSize, page * pageSize));

	function runSearch(e) {
		e.preventDefault();
		invalidate((url) => {
			return url.pathname === '/api/list';
		});
	}

	const logged_in = document.cookie.includes('token_set=');
</script>

<svelte:head>
	<title>Workshop Walker - Search</title>
	<meta property="og:title" content="Workshop Walker - Search" />
	<meta property="og:type" content="website" />
	<meta property="og:url" content={window.location.href} />
</svelte:head>

{#await data.searchRequest}
	<div class="flex h-full w-full place-content-center">
		<Shadow></Shadow>
	</div>
{:then value}
	{#if value.status}
		{@render errorCard(value)}
	{:else}
		<div class="min-h-screen">
			<div class="mx-auto px-4 py-8">
				<Search tags={app.v.tags}></Search>
				<div class="mt-6">
					<div class="mb-4 flex justify-between gap-4 w-full">
						<!--Left-->
						<div class="flex items-center gap-4">
							<div>
								<span class="text-white font-bold">{value.length}</span>
								<span class="text-sm opacity-60">results</span>
							</div>

							<SegmentedControl value={viewMode} onValueChange={(details) => (viewMode = details.value)} class="p-0">
								<SegmentedControl.Control>
									<SegmentedControl.Indicator />
									<SegmentedControl.Item value="grid">
										<SegmentedControl.ItemText>
											<Icon data={faGrip} class="fa-fw" />
											Grid
										</SegmentedControl.ItemText>
										<SegmentedControl.ItemHiddenInput />
									</SegmentedControl.Item>
									<SegmentedControl.Item value="table">
										<SegmentedControl.ItemText>
											<Icon data={faTableList} class="fa-fw" />
											Table
										</SegmentedControl.ItemText>
										<SegmentedControl.ItemHiddenInput />
									</SegmentedControl.Item>
								</SegmentedControl.Control>
							</SegmentedControl>

							<div class="flex items-center gap-2 w-fit">
								<span class="text-sm opacity-60 shrink-0">Per page</span>
								<select class="select"
								        value={pageSize}
								        onchange={(e) => (pageSize = Number(e.currentTarget.value))}>
									{#each [5, 10, 15, 30] as v}
										<option value={v}>Items {v}</option>
									{/each}
									<option value={value.length}>Show All</option>
								</select>
							</div>

							{#if viewMode === 'table'}
								<div class="flex items-center justify-between gap-1">
									<Switch
										name="show_images"
										checked={showTableImages}
										onCheckedChange={(e) => (showTableImages = e.checked)}
									></Switch>
									<label for="show_images">Show images</label>
								</div>
							{/if}
						</div>
						<!--Right-->
						<div>

							<Pagination {page} count={value.length} {pageSize} onPageChange={(event) => (page = event.page)}>
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
						</div>


						<!--{@debug data}-->

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
					<option>Any</option>
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
				</select>
			</div>

			<div class="flex flex-wrap gap-2 md:col-span-4">
				{#each app.v.tags as tag}
					<span class="flex items-center space-x-2">
						<input name="tag" class="checkbox" type="checkbox" value={tag} bind:group={tags.v} />
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
				<button type="submit" class="btn preset-filled" onclick={runSearch}> Search</button>
				<button type="reset" class="btn preset-filled-warning-500"> Reset</button>
			</div>
		</div>
	</form>
{/snippet}

{#snippet rTable(data)}
	<div class="table-wrap overflow-hidden rounded-lg shadow">
		<table class="table caption-bottom">
			<thead class="">
			<tr>
				{#if showTableImages === true}
					<th class="px-6 py-3 text-left text-xs font-medium tracking-wider uppercase">Image</th>
				{/if}
				<th class="px-6 py-3 text-left text-xs font-medium tracking-wider uppercase">Title</th>
				<th class="px-6 py-3 text-left text-xs font-medium tracking-wider uppercase">Author</th>
				<th class="px-6 py-3 text-left text-xs font-medium tracking-wider uppercase">
					Last Updated
				</th>
				<th class="px-6 py-3 text-left text-xs font-medium tracking-wider uppercase">
					Description
				</th>
			</tr>
			</thead>
			<tbody class="[&>tr]:hover:preset-tonal-primary divide-y divide-gray-200">
			{#each slicedSource(data) as item (item.id)}
				<tr class="group hover:bg-gray-50">
					{#if showTableImages === true}
						<td class="w-52 p-0">
							<a href="/item/{item.id}" target="_self" rel="noopener noreferrer">
								<img
									class="aspect-video object-cover lg:h-32 lg:min-w-48"
									class:hue-rotate-90={!item.preview_url}
									class:grayscale={!item.preview_url}
									src={item.preview_url ||
											'https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/294100/header.jpg?t=1734154189'}
									alt="banner"
									loading="lazy"
								/></a
							>
						</td>
					{/if}
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
						<a
							href="https://steamcommunity.com/profiles/{item.author}"
							class="anchor whitespace-nowrap"
						>
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
					<td class="table-description block h-36 overflow-hidden text-sm wrap-anywhere">
						<div class="relative h-full">
							<p class="line-clamp-5 text-sm leading-relaxed">{item.description}</p>
							<div
								class="pointer-events-none absolute right-0 bottom-0 left-0 h-10 bg-gradient-to-t from-[var(--bg-root-bg-dark)] to-transparent group-hover:from-[var(--color-primary-50-950)]"
							></div>
						</div>
					</td>
				</tr>
			{:else}
				<tr>
					<td colspan="4" class="px-6 py-4 text-center text-gray-500">No results found</td>
				</tr>
			{/each}
			</tbody>
		</table>
	</div>
{/snippet}

{#snippet rgrid(data)}
	<div class="flex flex-wrap place-content-center gap-4">
		{#each slicedSource(data) as item (item.id)}
			<ItemCard {item} loggedIn={logged_in}></ItemCard>
		{:else}
			<div class="text-center text-gray-500 py-8">No results found</div>
		{/each}
	</div>
{/snippet}

{#snippet pagination(obj)}
	<!-- Pagination -->

	<Pagination
		data={obj.data}
		{page}
		onPageChange={(e) => (page = e.page)}
		pageSize={pageSize}
		onPageSizeChange={(e) => (pageSize = e.pageSize)}
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
