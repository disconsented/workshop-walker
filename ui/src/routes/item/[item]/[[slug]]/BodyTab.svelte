<script lang="ts">
	import { whichLang } from '$lib/lang';
	import {
		faChevronLeft,
		faChevronRight,
		faEllipsis,
		faThumbsUp
	} from '@fortawesome/free-solid-svg-icons';
	import { faSteamSymbol } from '@fortawesome/free-brands-svg-icons';
	import Icon from 'svelte-awesome';
	import { Pagination, Portal, Tooltip } from '@skeletonlabs/skeleton-svelte';
	import slugify from 'slugify';

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
</script>

<div class="flex flex-col gap-2">
	<div class="grid w-full grid-cols-1 gap-2 md:grid-cols-2 lg:grid-cols-3">
		{#each paginated as dep}
			{@render itemCard(dep)}
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
{#snippet itemCard(item)}
	<div
		class="card border-surface-300-700 bg-surface-100-900 flex w-full flex-row place-items-center justify-between gap-2 border-1 p-2"
	>
		<div class="flex w-full min-w-0 flex-row place-items-center justify-between gap-2">
			<img src={item.preview_url} class="h-10" alt="Preview" />
			<div class="flex w-full min-w-0 flex-col">
				<Tooltip positioning={{ placement: 'top' }}>
					<Tooltip.Trigger class="flex"
						><a
							class="hover:anchor overflow-hidden text-nowrap text-ellipsis"
							href="/item/{item.id}/{slugify(item.title)}"
							target="_blank"
							rel="noopener noreferrer">{item.title}</a
						>
					</Tooltip.Trigger>
					<Portal>
						<Tooltip.Positioner>
							<Tooltip.Content class="card preset-filled-surface-950-50 p-2">
								<span>{item.title}</span>
								<Tooltip.Arrow
									class="[--arrow-background:var(--color-surface-950-50)] [--arrow-size:--spacing(2)]"
								>
									<Tooltip.ArrowTip />
								</Tooltip.Arrow>
							</Tooltip.Content>
						</Tooltip.Positioner>
					</Portal>
				</Tooltip>
				<span class="overflow-hidden text-sm text-nowrap text-ellipsis opacity-50"
					>{item.author.name}</span
				>
				<div class="w-min-0 grid w-full grid-cols-[1fr_auto] gap-2">
					<div class="w-min-0 flex flex-row overflow-auto">
						{#each item.tags as tag}
							<span
								class="chip preset-outlined-surface-400-600 hover:preset-tonal data-[state=on]:preset-filled-primary-500"
							>
								{tag.display_name}
							</span>
						{/each}
					</div>
					<span class="text-success-500 flex place-items-center gap-1"
						><Icon data={faThumbsUp} class="fa-fw" />
						{Math.trunc(item.score * 100)}%</span
					>
				</div>
			</div>
		</div>
		<div class="flex flex-col">
			<a
				href="/item/{item.id}/{slugify(item.title)}"
				target="_blank"
				rel="noopener noreferrer"
				class="btn anchor preset-outlined-surface-200-800"
			>
				<Icon data={faChevronRight} class="fa-fw"></Icon>
			</a>
			<a
				href="https://steamcommunity.com/sharedfiles/filedetails/?id={item.id}"
				target="_blank"
				rel="noopener noreferrer"
				class="btn anchor preset-outlined-surface-200-800"
			>
				<Icon data={faSteamSymbol} class="fa-fw"></Icon>
			</a>
		</div>
	</div>
{/snippet}
