<script lang="ts">
	import Icon from 'svelte-awesome';

	import type { PageData } from '../../../../.svelte-kit/types/src/routes';
	import {
		faChevronLeft,
		faChevronRight,
		faEllipsis,
		faExternalLink,
		faGrip,
		faImage,
		faTableList,
		faTriangleExclamation
	} from '@fortawesome/free-solid-svg-icons';
	import { app, tags } from './store.svelte';
	import ItemCard from './itemCard.svelte';

	import { Pagination, SegmentedControl, Switch } from '@skeletonlabs/skeleton-svelte';
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

	let viewMode = $state('table');
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
							>
								<SegmentedControl.Control class="gap-0 p-0">
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

{#snippet rTable(data)}
	<div class="card bg-surface-100-900 table-wrap border-surface-300-700 border-1">
		<div class="flex flex-row items-center justify-between p-2">
			<div><span class="font-bold text-white">{data.length}</span> results</div>

			<Switch
				name="showImages"
				checked={showTableImages}
				onCheckedChange={(e) => (showTableImages = e.checked)}
				class="preset-outlined-surface-200-800 data-[state=checked]:preset-outlined-surface-700-300 data-[state=checked]:preset-filled-surface-600-400
				p-1
				opacity-60
				data-[state=checked]:opacity-100"
			>
				<Switch.Label>
					<Icon data={showTableImages ? faImage : faTableList} class="fa-fw" />
					Thumbnails
				</Switch.Label>
				<Switch.Control class="data-[state=checked]:preset-filled-surface-500">
					<Switch.Thumb />
				</Switch.Control>

				<Switch.HiddenInput />
			</Switch>
		</div>
		<div
			class="grid"
			class:grid-cols-[auto_minmax(0,1fr)_auto_auto_auto_auto]={showTableImages}
			class:grid-cols-[minmax(0,1fr)_auto_auto_auto_auto]={!showTableImages}
		>
			<div
				class="border-surface-300-700 col-span-full grid grid-cols-subgrid justify-between border-1 p-2 text-sm uppercase opacity-60 [&>*]:p-1"
			>
				<div class:hidden={!showTableImages}>Preview</div>
				<div>Item</div>
				<div>Author</div>
				<div>Langs</div>
				<div>Updated</div>
				<div>Links</div>
			</div>

			{#each slicedSource(data) as item, i (item.id)}
				<div
					role="row"
					class="even:bg-surface-200-800 hover:preset-tonal-brand border-surface-300-700
					col-span-full grid grid-cols-subgrid justify-between border-b-1 px-2 [&>*]:p-1"
				>
					<div role="cell" class="h-[2lh] w-full grow-0" class:hidden={!showTableImages}>
						<img
							src={item.preview_url}
							class="h-[2lh] w-full rounded-md object-cover"
							alt="Item Preview"
						/>
					</div>
					<div role="cell" class="flex min-w-0 flex-col">
						<span class="font-bold">{item.title}</span>
						<div class="flex flex-row gap-1 text-ellipsis">
							{#each item.tags as tag (tag.id)}
								<span class="badge preset-outlined">{tag.display_name}</span>
							{:else}
								<span class="badge preset-outlined">-</span>
							{/each}
							<span class="line-clamp-1 h-[1lh] min-w-0">{@html item.description}</span>
						</div>
					</div>
					<div role="cell" class="flex place-items-center">
						<a
							href="https://steamcommunity.com/id/{item.author.id}"
							target="_self"
							rel="noopener noreferrer"
							class="anchor flex place-items-center gap-1"
						>
							<Icon data={faSteam} class="fa-fw" />
							{item.author.name}</a
						>
					</div>
					<div role="cell" class="flex flex-row place-items-center gap-1">
						{#each item.languages as language, i}
							{#if i != 0}·{/if}
							<span class="text-sm opacity-60"> {intToLanguage(language)}</span>{/each}
					</div>
					<div role="cell" class="flex place-items-center">
						<TimeAgo date={item.last_updated} short={true}></TimeAgo>
					</div>
					<div role="cell" class="flex flex-row place-items-center gap-1">
						<a
							href="/item/{item.id}"
							target="_self"
							rel="noopener noreferrer"
							class="btn preset-outlined-surface-300-700 p-2"
						>
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
				</div>
			{/each}
		</div>
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
