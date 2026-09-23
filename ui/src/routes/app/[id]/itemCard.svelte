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
	class="card preset-filled-surface-100-900 border-surface-200-800 divide-surface-200-800 flex w-full flex-col justify-between divide-y overflow-hidden border-[1px] md:w-md"
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
					<h6 class="h6 p-1">
						<a
							href="/item/{item.id}"
							target="_self"
							rel="noopener noreferrer"
							class="card inline-block place-items-center text-balance"
						>
							{item.title}
							<Icon data={faLink} class="fa-fw" />
						</a>
					</h6>
				</div>
				<div class="flex w-full place-items-center justify-between gap-2 p-1">
					{#if item.author}
						<a
							href="https://steamcommunity.com/profiles/{item.author
								.id}/myworkshopfiles/?appid={item.app}"
							target="_self"
							rel="noopener noreferrer"
							class="anchor flex min-w-0 items-center gap-1"
						>
							<Icon data={faSteam} class="fa-fw shrink-0" />
							<span class="truncate">{item.author?.name ?? 'Unknown'}</span></a
						>
					{/if}
					<div class="flex w-fit items-center text-nowrap">
						<span class="text-[0.5rem] text-gray-500">
							Updated: <TimeAgo date={item.last_updated}></TimeAgo></span
						>
					</div>
				</div>
			</div>
		</div>
	</header>
	<article class="flex grow flex-col justify-between space-y-4 p-4">
		<article
			class="prose dark:prose-invert mb-2 line-clamp-4 truncate
						overflow-hidden text-sm text-wrap text-ellipsis transition-all
						duration-150 ease-in-out hover:line-clamp-10 hover:overflow-scroll"
		>
			{@html item.description}
		</article>
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
