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
	import { Pagination } from '@skeletonlabs/skeleton-svelte';

	interface Props {
		items: any;
		selectedTags: { id: string; display_name: string }[];
		selectedLangs: string[];
	}

	let { items, selectedLangs, selectedTags }: Props = $props();

	function filter(item) {
		const itemTags = (item.tags || []).map((e) => e.id);
		const matchesTags =
			selectedTags.length === 0 || selectedTags.every((tagId) => itemTags.includes(tagId));
		const itemLangs = (item.languages || []).map((langId) => whichLang(langId));
		const matchesLangs =
			selectedLangs.length === 0 || itemLangs.some((langName) => selectedLangs.includes(langName));
		return matchesTags && matchesLangs;
	}

	let filtered = $derived(
		(items || [])
			.filter((e) => filter(e))
			.toSorted((b, a) => (a.last_updated || 0) - (b.last_updated || 0))
	);

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
				<span class="overflow-hidden text-nowrap text-ellipsis">{item.title}</span>
				<span class="overflow-hidden text-sm text-nowrap text-ellipsis opacity-50"
					>{item.author.name}</span
				>
				<div class="grid grid-cols-[1fr_auto] gap-2">
					<div class="w-min-0 flex flex-row gap-1 overflow-ellipsis">
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
				href="/item/{item.id}"
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
