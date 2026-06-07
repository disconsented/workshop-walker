<script lang="ts">
	import { faChevronDown, faChevronUp, faLink, faLock } from '@fortawesome/free-solid-svg-icons';
	import { faSteamSymbol } from '@fortawesome/free-brands-svg-icons';
	import TimeAgo from '$lib/timeAgo.svelte';
	import Property from '../../item/[item]/Property.svelte';
	import Icon from 'svelte-awesome';

	interface Props {
		loggedIn: boolean; // Used for allowing voting
		item: any;
	}

	let { loggedIn = $bindable(), item }: Props = $props();

	let first_props = $derived(item.properties?.slice(0, 6));
	let remaining_props = $derived(item.properties?.slice(6));
	$inspect(first_props, remaining_props);
	let open = $state(false);
</script>

<div
	class="card preset-filled-surface-100-900 border-surface-200-800 card-hover divide-surface-200-800 block w-md divide-y overflow-hidden border-[1px]"
>
	<header class="relative h-48">
		<div>
			<img
				src={item.preview_url ||
					'https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/294100/header.jpg?t=1734154189'}
				class="absolute h-48 w-full object-cover"
				alt="banner"
				class:hue-rotate-90={!item.preview_url}
				class:grayscale={!item.preview_url}
				onerror={(e) =>
					(e.target.src =
						'https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/294100/header.jpg?t=1734154189')}
				loading="lazy"
			/>
			<div class="absolute h-48 w-full bg-linear-to-t from-black to-[transparent]">&nbsp</div>
		</div>

		<!--Details overlaid-->
		<div class="t-0 absolute left-0 flex h-full w-full flex-col justify-between">
			<!--Top-->
			<div class="flex w-full justify-end">
				<button
					class="btn preset-filled-surface-50-950 mt-1 text-xs text-gray-500 opacity-80  border-1 border-gray-500  border-dashed">
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
			<div class="flex w-full flex-col">
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
					<div class="mb-2 flex items-center">
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
		{#if first_props}
			{@debug first_props}
			<div class="flex flex-wrap gap-1">
				{#each first_props as prop}
					<Property {loggedIn} property={{ class: prop.out.class, value: prop.out.value, ...prop }} hideVote={false} itemID={item.id}></Property>
				{/each}
				{#if remaining_props.length > 0}
					{#if open}
						{#each remaining_props as prop}
							<Property {loggedIn} property={{ class: prop.out.class, value: prop.out.value, ...prop }} hideVote={false} itemID={item.id}
							></Property>
						{/each}
					{/if}
					<button
						class="text-primary-500 ca w-full text-left text-sm"
						onclick={() => {
							open = !open;
						}}
					>
						{#if open}
							<Icon data={faChevronUp} class="fa-fw"></Icon>
						{:else}
							<Icon data={faChevronDown} class="fa-fw"></Icon>
						{/if}<span class="pl-1">{remaining_props.length} more properties</span>
					</button>
				{/if}
			</div>
		{/if}
		<button
			class="btn btn-sm preset-outlined-primary-500 text-primary-500 mt-1 w-full justify-between pt-1 opacity-50"
		><span><Icon data={faLock} class="fa-fw"></Icon> Sign in to vote on properties</span>
			<span class="btn btn-sm preset-filled-primary-500"
			><Icon data={faSteamSymbol} class="fa-fw"></Icon> Sign in</span
			></button
		>
	</footer>
</div>

<style>
    .preset-glass-surface {
        background: color-mix(in oklab, var(--color-surface-900) 40%, transparent);
        box-shadow: 0 0px 30px color-mix(in oklab, var(--color-surface-900) 50%, transparent) inset;
        backdrop-filter: blur(16px);
    }
</style>
