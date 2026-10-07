<script lang="ts">
	import { whichLang } from '$lib/lang';
	import { faChevronLeft, faChevronRight, faEllipsis } from '@fortawesome/free-solid-svg-icons';
	import { faSteam } from '@fortawesome/free-brands-svg-icons';
	import Icon from 'svelte-awesome';
	import { Pagination } from '@skeletonlabs/skeleton-svelte';
	import slugify from 'slugify';
	import TimeAgo from '$lib/timeAgo.svelte';

	interface Props {
		items: any;
		selectedTags: { id: string; display_name: string }[];
		selectedLangs: string[];
		sort: string;
		filterName: string | undefined;
	}

	let { items, selectedLangs, selectedTags, sort, filterName }: Props = $props();

	function filter(item) {
		const itemTags = (item.tags || []).map((e) => e.id);
		const matchesTags =
			selectedTags.length === 0 || selectedTags.every((tagId) => itemTags.includes(tagId));
		const itemLangs = (item.languages || []).map((langId) => whichLang(langId));
		const matchesLangs =
			selectedLangs.length === 0 || itemLangs.some((langName) => selectedLangs.includes(langName));
		// If a filter name is supplied check the title contains it
		const matchesName = filterName
			? item.title.toLowerCase().includes(filterName.toLowerCase())
			: true;
		return matchesTags && matchesLangs && matchesName;
	}

	const sortFunction = (b, a) => {
		switch (sort) {
			case 'popularity':
				return a.hotness - b.hotness;
			case 'subscriptions':
				return a.subscriptions - b.subscriptions;
			case 'trending':
				return a.trend_week - b.trend_week;
			case 'votes':
				return a.score - b.score;
			default:
				return (a.last_updated || 0) - (b.last_updated || 0);
		}
	};

	let filtered = $derived((items || []).filter((e) => filter(e)).toSorted(sortFunction));

	let page = $state(1);
	let pageSize = $state(3 * 5);
	let paginated = $derived(filtered.slice((page - 1) * pageSize, page * pageSize));

	const compact = new Intl.NumberFormat('en', {
		notation: 'compact',
		maximumFractionDigits: 1
	});
</script>

<div class="flex flex-col gap-2">
	<div class="grid w-full grid-cols-1 gap-2 md:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4">
		{#each paginated as dep}
			{@render itemCardCompact(dep)}
		{/each}
	</div>

	<Pagination
		{page}
		count={filtered.length}
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
{#snippet itemCardCompact(item)}
	<div
		class="card border-surface-300-700 bg-surface-100-900 flex w-full flex-col place-items-center justify-between gap-2 border-1"
	>
		<div class="relative h-48 w-full">
			<header>
				<img
					src={item.preview_url}
					class="pattern-background absolute h-48 w-full object-cover"
					alt="banner"
					class:hue-rotate-90={!item.preview_url}
					class:grayscale={!item.preview_url}
					loading="lazy"
				/>
				<div class="absolute h-48 w-full bg-linear-to-t from-black from-30% to-[transparent]"></div>
			</header>

			<!--Details overlaid-->
			<article class="t-0 absolute left-0 flex h-full w-full flex-col justify-between pr-2">
				<!--Top-->
				<div class="flex w-full justify-end gap-1">
					<a
						href="https://steamcommunity.com/sharedfiles/filedetails/?id={item.id}"
						target="_blank"
						rel="noopener noreferrer"
						class="btn preset-filled-surface-50-950 mt-1 rounded-md border-1 border-dashed border-gray-500 text-xs text-gray-500 opacity-80 hover:text-gray-700"
					>
						<Icon data={faSteam} class="fa-fw" />
						Steam
					</a>
				</div>
				<!--Bottom (Title, author, updated-->
				<div class="flex w-full flex-col">
					<div class="w-full">
						<h6 class="h6 p-1 text-sm">
							<a
								href="/item/{item.id}/{slugify(item.title)}"
								target="_self"
								rel="noopener noreferrer"
								class="card inline-block place-items-center text-pretty"
							>
								{item.title}
							</a>
						</h6>
					</div>
					<footer class="flex w-full place-items-center justify-between gap-2 p-1">
						{#if item.author}
							<a
								href="https://steamcommunity.com/profiles/{item.author
									.id}/myworkshopfiles/?appid={item.app}"
								target="_self"
								rel="noopener noreferrer"
								class="anchor flex min-w-0 items-center gap-1 text-xs"
							>
								<span class="truncate">{item.author?.name ?? 'Unknown'}</span></a
							>
						{/if}
						<div class="flex w-fit items-center gap-1 text-nowrap">
							{@render sortedDetail(item)}
						</div>
					</footer>
				</div>
			</article>
		</div>
	</div>
{/snippet}

{#snippet sortedDetail(item)}
	{#if sort === 'popularity'}
		<span class="text-xs text-gray-500">{compact.format(item.score * 100)}%</span>
		<span class="text-primary-500 text-xs capitalize">
			{compact.format(item.hotness)}
		</span>
	{:else if sort === 'subscriptions'}
		<span class="text-xs text-gray-500">{compact.format(item.score * 100)}%</span>
		<span class="text-primary-500 text-xs capitalize">
			{compact.format(item.subscriptions)}
		</span>
	{:else if sort === 'trending'}
		<span class="text-xs text-gray-500">{compact.format(item.score * 100)}%</span>
		<span class="text-primary-500 text-xs capitalize">
			+{compact.format(item.trend_week * 100)}%
		</span>
	{:else if sort === 'votes'}
		<span class="text-xs text-gray-500">{compact.format(item.subscriptions)}</span>
		<span class="text-primary-500 text-xs">{compact.format(item.score * 100)}%</span>
	{:else}
		<span class="text-xs text-gray-500">{compact.format(item.score * 100)}%</span>
		<span class="text-primary-500 text-xs capitalize">
			<TimeAgo date={item.last_updated}></TimeAgo>
		</span>
	{/if}
{/snippet}
