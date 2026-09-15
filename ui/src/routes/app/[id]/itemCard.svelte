<script lang="ts">
	import { faLink } from '@fortawesome/free-solid-svg-icons';
	import { faSteam } from '@fortawesome/free-brands-svg-icons';
	import TimeAgo from '$lib/timeAgo.svelte';
	import Icon from 'svelte-awesome';
	import Properties from '../../../components/Properties.svelte';

	interface Props {
		loggedIn: boolean; // Used for allowing voting
		item: any;
	}

	let { loggedIn = $bindable(), item }: Props = $props();
</script>

<div
	class="card preset-filled-surface-100-900 border-surface-200-800 divide-surface-200-800 flex w-md flex-col justify-between divide-y overflow-hidden border-[1px]"
>
	<header class="relative h-48">
		<div>
			<img
				src={item.preview_url}
				class="pattern-background absolute h-48 w-full object-cover"
				alt="banner"
				class:hue-rotate-90={!item.preview_url}
				class:grayscale={!item.preview_url}
				loading="lazy"
			/>
			<div class="absolute h-48 w-full bg-linear-to-t from-black to-[transparent]"></div>
		</div>

		<!--Details overlaid-->
		<div class="t-0 absolute left-0 flex h-full w-full flex-col justify-between">
			<!--Top-->
			<div class="flex w-full justify-end">
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
					<h6 class="h6">
						<a href="/item/{item.id}" target="_self" rel="noopener noreferrer" class="card p-1">
							{item.title}
							<Icon data={faLink} class="fa-fw"></Icon>
						</a>
					</h6>
				</div>
				<div class="flex w-full items-center justify-between p-1">
					{#if item.author}
						<a
							href="https://steamcommunity.com/profiles/{item.author
								.id}/myworkshopfiles/?appid={item.app}"
							target="_self"
							rel="noopener noreferrer"
							class="anchor flex items-center gap-1"
						>
							<Icon data={faSteam} class="fa-fw" />
							{item.author?.name ?? 'Unknown'}</a
						>
					{/if}
					<div class="mb-2 flex items-center">
						<span class="text-[0.5rem] text-gray-500">
							Updated: <TimeAgo date={item.last_updated}></TimeAgo></span
						>
					</div>
				</div>
			</div>
		</div>
	</header>
	<article class="flex grow flex-col justify-between space-y-4 p-4">
		<div
			class="mb-2 max-h-[4lh] overflow-hidden text-sm text-wrap text-ellipsis
						text-gray-600 transition-[height] duration-150 ease-in-out hover:max-h-[10lh] hover:overflow-scroll"
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
	<footer class="m-2 flex w-full grow-0 flex-row flex-wrap self-end pl-4">
		<Properties {loggedIn} itemID={item.id} properties={item.properties} />
	</footer>
</div>

<style>
	.pattern-background {
		background-color: var(--color-surface-800);
	}

	.pattern-background::before {
		content: '';
		position: absolute;
		inset: -100%;
		transform: rotate(90deg);
		transform-origin: center;
		background-color: var(--color-surface-800);
		opacity: 0.8;
		background-size: 10px 10px;
		background-image: repeating-linear-gradient(
			45deg,
			var(--color-surface-400) 0,
			var(--color-surface-400) 1px,
			var(--color-surface-800) 0,
			var(--color-surface-800) 50%
		);
	}
</style>
