<script lang="ts">
	import Icon from 'svelte-awesome';
	import { faSteamSymbol } from '@fortawesome/free-brands-svg-icons';
	import { faThumbsDown, faThumbsUp } from '@fortawesome/free-solid-svg-icons';

	interface Props {
		loggedIn: boolean; // Used for allowing voting
		item: {};
	}

	let { loggedIn = $bindable(), item }: Props = $props();
</script>

<div
	class="card preset-filled-surface-100-900 border-surface-200-800 card-hover divide-surface-200-800 border-l-primary-500 flex max-w-md flex-col
            place-content-between divide-y overflow-hidden border-[1px] border-l-4"
>
	<!-- Title with voting -->
	<div class="flex flex-col">
		<a href="/item/{item.id}" target="_self" rel="noopener noreferrer" class="hover:filter-none">
			<img
				src={item.preview_url}
				alt="banner"
				loading="lazy"
				class="aspect-[21/9] w-full object-cover grayscale hue-rotate-90 hover:filter-none"
				class:hidden={false}
				onerror={(e) =>
					(e.target.src =
						'https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/294100/header.jpg?t=1734154189')}
			/>
			<div class="preset-filled-surface-100-900 rounded-sm text-center font-medium">
				{item.title}
			</div>
		</a>

		<!-- Voting controls -->
		<div class="flex items-center justify-center gap-4 p-2">
			<button
				class="btn preset-tonal-surface hover:bg-error-500/20 rounded-full p-2"
				aria-label="Downvote"
			>
				<Icon data={faThumbsDown} class="fa-fw text-error-500" />
			</button>
			<span class="text-primary-500 font-mono text-lg font-bold">
				{item.votes ?? 0}
			</span>
			<button
				class="btn preset-tonal-surface hover:bg-success-500/20 rounded-full p-2"
				aria-label="Upvote"
			>
				<Icon data={faThumbsUp} class="fa-fw text-success-500" />
			</button>
		</div>
	</div>

	<!-- Tags -->
	<article class="flex flex-wrap gap-1" style="align-items: end">
		{#each item.tags as tag (tag.id)}
			<span class="badge preset-tonal-surface">
				{tag.display_name}
			</span>
		{:else}
			<span class="text-gray-400">No tags</span>
		{/each}
	</article>

	<!-- Links -->
	<footer class="input-group w-min-full grid-cols-[auto_auto]">
		<a
			href="https://steamcommunity.com/sharedfiles/filedetails/?id={item.id}"
			target="_blank"
			rel="noopener noreferrer"
			class="btn preset-tonal-surface flex items-center gap-2 truncate whitespace-normal"
		>
			<Icon data={faSteamSymbol} class="fa-fw"></Icon>
			Workshop
		</a>
		<a
			href="/item/{item.id}"
			target="_self"
			rel="noopener noreferrer"
			class="btn preset-tonal-primary flex items-center gap-2 truncate whitespace-normal"
		>
			<Icon data={faLink} class="fa-fw"></Icon>
			View
		</a>
	</footer>
</div>
