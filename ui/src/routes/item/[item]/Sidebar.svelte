<script lang="ts">
	import { faSteam } from '@fortawesome/free-brands-svg-icons';
	import { Progress } from '@skeletonlabs/skeleton-svelte';
	import Icon from 'svelte-awesome';
	import TimeAgo from '$lib/timeAgo.svelte';
	import Properties from '../../../components/Properties.svelte';
	import { inToLangShort } from '$lib/lang';
	import PopularityChart from './PopularityChart.svelte';

	interface Props {
		loggedIn: boolean; // Used for allowing voting
		item: any;
	}

	let { item, loggedIn }: Props = $props();

	let progress = Math.trunc(item.score * 100);
</script>

<div class="flex flex-col gap-4">
	<div class="flex flex-col gap-4">
		<img
			src={item.preview_url}
			class="pattern-background h-48 w-full object-cover"
			alt="banner"
			loading="lazy"
		/>
		<h3 class="h3">{item.title}</h3>
		{#if item.author}
			<span>
				by
				<a
					href="https://steamcommunity.com/profiles/{item.author
						.id}/myworkshopfiles/?appid={item.app}"
					target="_self"
					rel="noopener noreferrer"
					class="anchor"
				>
					{item.author?.name ?? 'Unknown'}</a
				>
			</span>
		{/if}

		<div>
			<a
				href="https://steamcommunity.com/sharedfiles/filedetails/?id={item.id}"
				target="_blank"
				rel="noopener noreferrer"
				class="btn preset-outlined-surface-400-600"
			>
				<Icon data={faSteam} class="fa-fw" />
				Open on Steam
			</a>
		</div>
	</div>
	<hr class="hr border-b-surface-200-800" />
	<div class="flex flex-col gap-2">
		<div class="flex flex-row justify-between">
			<span class="uppercase opacity-50">Created</span>
			<span class="capitalize"><TimeAgo date={item.created} /></span>
		</div>

		<div class="flex flex-row justify-between">
			<span class="uppercase opacity-50">Updated</span>
			<span class="capitalize"><TimeAgo date={item.last_updated} /></span>
		</div>

		<div>
			<span> {progress}% Upvoted</span>
			<Progress value={progress}>
				<Progress.Track class="bg-primary-50-950 h-1">
					<Progress.Range class="bg-primary-500" />
				</Progress.Track>
			</Progress>
		</div>

		<div class="flex flex-row justify-between">
			<span class="uppercase opacity-50">Conversions</span>
			<span>{Math.trunc(item.conversions * 100)}%</span>
		</div>

		<div class="flex flex-row justify-between">
			<span class="uppercase opacity-50">Retention</span>
			<span>{Math.trunc(item.retention * 100)}%</span>
		</div>

		<div class="flex flex-row justify-between">
			<span class="uppercase opacity-50">Dependencies</span>
			<span>{item.dependencies.length}</span>
		</div>
		<div class="flex flex-row justify-between">
			<span class="uppercase opacity-50">Dependants</span>
			<span>{item.dependants.length}</span>
		</div>
	</div>
	<hr class="hr border-b-surface-200-800" />
	<div class="flex flex-col">
		<span class="uppercase opacity-50">Popularity · 12 months</span>
		<div>
			<PopularityChart data={item.subscription_history ?? []} />
		</div>
	</div>
	<hr class="hr border-b-surface-200-800" />
	<div class="flex flex-col">
		<span class="text-sm uppercase opacity-50">Tags · {item.tags.length}</span>

		<div class="flex flex-row gap-2">
			{#each item.tags as tag}
				<span
					class="chip preset-outlined-surface-400-600 hover:preset-tonal data-[state=on]:preset-filled-primary-500"
				>
					{tag.display_name}
				</span>
			{/each}
		</div>
	</div>
	<hr class="hr border-b-surface-200-800" />
	<div class="flex flex-col">
		<span class="text-sm uppercase opacity-50">Langs · {item.languages.length}</span>

		<div class="flex flex-row gap-2">
			{#each item.languages as lang, i}
				{#if i != 0}·{/if}
				<span>{inToLangShort(lang)}</span>
			{/each}
		</div>
	</div>
	<hr class="hr border-b-surface-200-800" />
	<div>
		<span class="text-sm uppercase opacity-50">Community Properties</span>
		<Properties {loggedIn} itemID={item.id} properties={item.properties} />
	</div>
</div>

<style>
	.pattern-background {
		background-color: var(--color-surface-800);
		transform-origin: center;
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
