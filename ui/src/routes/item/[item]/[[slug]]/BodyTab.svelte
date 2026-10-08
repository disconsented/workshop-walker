<script lang="ts">
	import { whichLang } from '$lib/lang';
	import {
		faArrowRight,
		faChevronLeft,
		faChevronRight,
		faEllipsis
	} from '@fortawesome/free-solid-svg-icons';
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
		class="card group border-surface-300-700 bg-surface-100-900 hover:border-primary-500/60 has-[a.stretched:focus-visible]:outline-primary-500 relative isolate flex h-48 w-full flex-col justify-end overflow-hidden border-1 transition-colors has-[a.stretched:focus-visible]:outline-2 has-[a.stretched:focus-visible]:outline-offset-2"
	>
		<img
			src={item.preview_url}
			alt="preview"
			loading="lazy"
			class="pattern-background absolute inset-0 -z-10 h-full w-full object-cover transition-transform duration-300 group-hover:scale-105"
			class:hue-rotate-90={!item.preview_url}
			class:grayscale={!item.preview_url}
		/>
		<div class="absolute inset-0 -z-10 bg-linear-to-t from-black from-30% to-transparent"></div>

		<a
			href="https://steamcommunity.com/sharedfiles/filedetails/?id={item.id}"
			target="_blank"
			rel="noopener noreferrer"
			title="Open on Steam Workshop"
			aria-label="Open {item.title} on Steam Workshop"
			class="text-surface-50 absolute top-2 right-2 z-20 grid size-8 place-items-center rounded-md border border-white/10 bg-black/60 transition-colors hover:bg-black hover:text-white"
		>
			<Icon data={faSteam} />
		</a>

		<div class="flex items-end gap-2.5 p-2.5">
			<div class="flex min-w-0 flex-1 flex-col gap-1">
				<h6 class="h6 text-sm text-pretty">
					<a
						href="/item/{item.id}/{slugify(item.title)}"
						class="stretched after:absolute after:inset-0 after:z-10 focus-visible:outline-none"
					>
						{item.title}
					</a>
				</h6>
				<footer class="flex items-center justify-between gap-2 text-xs">
					<a
						href="https://steamcommunity.com/profiles/{item.author
							.id}/myworkshopfiles/?appid={item.app}"
						target="_self"
						rel="noopener noreferrer"
						class="text-surface-200 truncate anchor z-10">{item.author?.name ?? 'Unknown'}</a
					>
					<div class="flex items-center gap-1 text-nowrap">
						{@render sortedDetail(item)}
					</div>
				</footer>
			</div>
			<span
				aria-hidden="true"
				class="preset-tonal-primary group-hover:preset-filled-primary-500 grid size-8 shrink-0 place-items-center rounded-md transition-colors"
			>
				<Icon data={faArrowRight} />
			</span>
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
