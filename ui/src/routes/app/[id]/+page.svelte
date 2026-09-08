<script lang="ts">
	import Icon from 'svelte-awesome';

	import type { PageData } from '../../../../.svelte-kit/types/src/routes';
	import {
		fa1,
		faArrowLeft,
		faArrowRight,
		faChevronLeft,
		faChevronRight,
		faCross,
		faEllipsis,
		faExternalLink,
		faGrip,
		faImage,
		faTableList,
		faTriangleExclamation
	} from '@fortawesome/free-solid-svg-icons';
	import { app, language, limit, orderBy, tags, title } from './store.svelte';
	import ItemCard from './itemCard.svelte';

	import { Pagination, SegmentedControl, Switch } from '@skeletonlabs/skeleton-svelte';
	import TimePicker from '$lib/timePicker.svelte';
	import { Shadow } from 'svelte-loading-spinners';
	import { invalidate } from '$app/navigation';
	import Search from './search.svelte';
	import { faSteam } from '@fortawesome/free-brands-svg-icons';
	import TimeAgo from '$lib/timeAgo.svelte';

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

	function intToLanguage(int: number) {
		switch (int) {
			case 1:
				return 'EN';
			case 2:
				return 'RU';
			case 3:
				return 'CN';
			case 4:
				return 'JP';
			case 5:
				return 'KR';
			case 6:
				return 'ES';
			case 7:
				return 'PT';
		}
	}
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
					<div class="mb-4 flex w-full justify-between gap-4">
						<!--Left-->
						<div class="flex items-center gap-4">
							<div>
								<span class="font-bold text-white">{value.length}</span>
								<span class="text-sm opacity-60">results</span>
							</div>

							<SegmentedControl
								value={viewMode}
								onValueChange={(details) => (viewMode = details.value ?? 'grid')}
								class="p-0"
							>
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

							<div class="flex w-fit items-center gap-2">
								<span class="shrink-0 text-sm opacity-60">Per page</span>
								<select
									class="select"
									value={pageSize}
									onchange={(e) => (pageSize = Number(e.currentTarget.value))}
								>
									{#each [5, 10, 15, 30] as v}
										<option value={v}>Items {v}</option>
									{/each}
									<option value={value.length}>Show All</option>
								</select>
							</div>
						</div>
						<!--Right-->
						<div>
							<Pagination
								{page}
								count={value.length}
								{pageSize}
								onPageChange={(event) => (page = event.page)}
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
						</div>
					</div>

					{#if viewMode === 'table'}
						{@render rTable(value)}
					{:else}
						{@render rGrid(value)}
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
	<div class="card bg-surface-100-900 table-wrap p-4">
		<table class="table-zebra table">
			<caption class="pt-4">
				<div class="flex flex-row justify-between">
					<span>{data.length} results</span>
					<Switch
						name="showImages"
						checked={showTableImages}
						onCheckedChange={(e) => (showTableImages = e.checked)}
						dir="rtl"
					>
						<Switch.Control>
							<Switch.Thumb />
						</Switch.Control>
						<Switch.Label>
							<Icon data={faImage} class="fa-fw" />
							Thumbnails
						</Switch.Label>
						<Switch.HiddenInput />
					</Switch>
				</div>
			</caption>
			<thead>
			<tr>
				{#if showTableImages}
					<th>&nbsp;</th>
				{/if}
				<th>Item</th>
				<th>Author</th>
				<th>Langs</th>
				<th>Updated</th>
				<th>Links</th>
			</tr>
			</thead>
			<tbody class="[&>tr]:hover:preset-tonal-brand">
			{#each slicedSource(data) as item (item.id)}
				<tr>
					{#if showTableImages}
						<td class="w-full h-[2lh]">
							<img src={item.preview_url} class="rounded-md object-cover" alt="Item Preview" />
						</td>
					{/if}
					<td>
						<div class="flex flex-col">
							<span class="font-bold">{item.title}</span>
							<div class="flex flex-row gap-1 text-ellipsis">
								{#each item.tags as tag (tag.id)}
									<span class="badge preset-outlined">{tag.display_name}</span>
								{:else}
									<span class="badge preset-outlined">-</span>
								{/each}
								<span class="line-clamp-1 max-h-[1lh]">{@html item.description}</span>
							</div>
						</div>
					</td>
					<td
					><a
						href="https://steamcommunity.com/id/{item.author.id}"
						target="_self"
						rel="noopener noreferrer"
						class="anchor flex items-center gap-1"
					>
						<Icon data={faSteam} class="fa-fw" />
						{item.author.name}</a
					></td
					>
					<td>
						<div class="flex flex-row gap-1">
							{#each item.languages as language, i}
								{#if i != 0}·{/if}
								<span class="text-sm opacity-60">
										{intToLanguage(language)}</span
								>{/each}
						</div>
					</td>
					<td>
						<TimeAgo date={item.last_updated} short={true}></TimeAgo>
					</td>
					<td>
						<div class="flex flex-row gap-1 place-items-center">
							<a href="/item/{item.id}" target="_self" rel="noopener noreferrer"
							   class="btn preset-outlined-surface-300-700 p-2">
								<Icon data={faExternalLink} class="fa-fw" />
							</a>

							<a
								href="https://steamcommunity.com/sharedfiles/filedetails/?id={item.id}"
								target="_blank"
								rel="noopener noreferrer"
								class="btn preset-outlined-surface-300-700 p-2"
							>
								<Icon data={faSteam} class="fa-fw" />
							</a>
						</div>
					</td>
				</tr>
			{/each}
			</tbody>
		</table>
	</div>

{/snippet}

{#snippet rGrid(data)}
	<div class="flex flex-wrap place-content-center gap-4">
		{#each slicedSource(data) as item (item.id)}
			<ItemCard {item} loggedIn={logged_in}></ItemCard>
		{:else}
			<div class="text-center text-gray-500 py-8">No results found</div>
		{/each}
	</div>
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
