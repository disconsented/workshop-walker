<script lang="ts">
	import { faLink } from '@fortawesome/free-solid-svg-icons';
	import { faSteamSymbol } from '@fortawesome/free-brands-svg-icons';
	import TimeAgo from '$lib/timeAgo.svelte';
	import Property from '../../item/[item]/Property.svelte';
	import Icon from 'svelte-awesome';
	import { Collapsible } from '@skeletonlabs/skeleton-svelte';

	interface Props {
		loggedIn: boolean; // Used for allowing voting
		item: any;
	}

	let { loggedIn = $bindable(), item }: Props = $props();
	item.properties = [
		{
			class: 'feature',
			value: 'Weapons',
			upvote_count: 44,
			vote_count: 44,
			status: 1,
			vote_state: 1
		},
		{
			class: 'feature',
			value: 'Mechanoids',
			upvote_count: 24,
			vote_count: 24,
			status: 0,
			vote_state: 1
		},
		{
			class: 'setting',
			value: 'Combat',
			upvote_count: 21,
			vote_count: 21,
			status: -1,
			vote_state: 1
		},
		{
			class: 'theme',
			value: 'Sci-Fi',
			upvote_count: 16,
			vote_count: 16,
			status: 1,
			vote_state: 1
		},
		{
			class: 'feature',
			value: 'Turrets',
			upvote_count: 11,
			vote_count: 11,
			status: 1,
			vote_state: 1
		},
		{
			class: 'feature',
			value: 'Turrets',
			upvote_count: 44,
			vote_count: 11,
			status: 1,
			vote_state: -1
		},
		{
			class: 'setting',
			value: 'Vanilla-Adjacent',
			upvote_count: 4,
			vote_count: -1,
			status: 1,
			vote_state: 0
		}
	];
</script>

<div
	class="card preset-filled-surface-100-900 border-surface-200-800 card-hover divide-surface-200-800 block w-md divide-y overflow-hidden border-[1px]"
>
	<header class="relative h-48">
		<img
			src={item.preview_url ||
				'https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/294100/header.jpg?t=1734154189'}
			class="absolute h-48 w-full w-full object-cover"
			alt="banner"
			class:hue-rotate-90={!item.preview_url}
			class:grayscale={!item.preview_url}
			onerror={(e) =>
				(e.target.src =
					'https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/294100/header.jpg?t=1734154189')}
			loading="lazy"
		/>
		<!--Details overlayed-->
		<div class="t-0 absolute left-0 flex h-full w-full flex-col justify-between">
			<!--Top-->
			<div class="flex w-full justify-end">
				<button class="btn preset-outlined mt-1 text-xs text-gray-500">
					<Icon data={faSteamSymbol} class="fa-fw"></Icon>
					<a
						href="https://steamcommunity.com/sharedfiles/filedetails/?id={item.id}"
						target="_blank"
						rel="noopener noreferrer"
						class="hover:text-gray-700"
						>Steam
					</a>
				</button>
			</div>
			<!--Bottom (Title, author, updated-->
			<div class="preset-glass-surface flex w-full flex-col">
				<div class="w-full">
					<h6 class="h6">
						<a href="/item/{item.id}" target="_self" rel="noopener noreferrer" class="card p-1">
							{item.title}
							<Icon data={faLink} class="fa-fw"></Icon>
						</a>
					</h6>
				</div>
				<div class="flex w-full items-center justify-between p-1">
					<a
						href="/user/{item.author.id}"
						target="_self"
						rel="noopener noreferrer"
						class="anchor flex items-center gap-1"
					>
						<Icon data={faSteamSymbol} class="fa-fw"></Icon>
						FUJIKENGAWA</a
					>
					<div class="mb-2 flex  items-center">
						<span class="text-[0.5rem] text-gray-500">
							Updated: <TimeAgo date={item.last_updated}></TimeAgo></span
						>
					</div>
				</div>
			</div>
		</div>
	</header>
	<article class="space-y-4 p-4">
		<div
			class="mb-2 max-h-[3lh] overflow-hidden text-sm text-wrap text-ellipsis
						text-gray-600 transition-[height] duration-150 ease-in-out hover:max-h-[10lh]"
		>
			{@html item.description}
		</div>
		<div class="flex flex-wrap gap-1">
			{#each item.tags as tag (tag.id)}
				<span class="badge preset-outlined">{tag.display_name}</span>
			{:else}
				<span class="badge preset-outlined">-</span>
			{/each}
		</div>
	</article>
	<footer class="m-2">
		{#if item.properties && item.properties.length > 0}
			<div class="flex flex-wrap gap-1">
				{#each item.properties as prop}
					{@debug prop}
					<Property {loggedIn} property={{ ...prop }} hideVote={false} itemID={item.id}></Property>
				{/each}
				<Collapsible>
					<Collapsible.Trigger />
					<Collapsible.Content />
				</Collapsible>
			</div>
		{/if}
	</footer>
</div>

<style>
	.preset-glass-surface {
		background: color-mix(in oklab, var(--color-surface-900) 40%, transparent);
		box-shadow: 0 0px 30px color-mix(in oklab, var(--color-surface-900) 50%, transparent) inset;
		backdrop-filter: blur(16px);
	}
</style>
