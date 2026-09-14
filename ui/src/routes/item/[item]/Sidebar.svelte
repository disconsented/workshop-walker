<script lang="ts">
	import { faSteam } from '@fortawesome/free-brands-svg-icons';
	import { Progress } from '@skeletonlabs/skeleton-svelte';
	import Icon from 'svelte-awesome';
	import TimeAgo from '$lib/timeAgo.svelte';
	import Properties from '../../../components/Properties.svelte';
	import { inToLangShort } from '$lib/lang';

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
			class="pattern-background h-48 w-full object-scale-down object-center"
			alt="banner"
			loading="lazy"
		/>
		<h3 class="h3">{item.title}</h3>
		<span>
			by
			<a
				href="https://steamcommunity.com/id/{item.author.id}"
				target="_self"
				rel="noopener noreferrer"
				class="anchor"
			>
				{item.author.name}</a
			>
		</span>

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
	<div class="flex flex-col">
		<div>
			<span> {Math.trunc(item.score * 100)}%</span>
			<Progress value={progress}>
				<Progress.Track class="bg-primary-50-950 h-1">
					<Progress.Range class="bg-primary-500" />
				</Progress.Track>
			</Progress>
		</div>

		<div class="flex flex-row justify-between">
			<span class="opacity-50">Updated</span>
			<TimeAgo date={item.last_updated} />
		</div>
	</div>
	<hr class="hr border-b-surface-200-800" />
	<div class="flex flex-col">
		<span class="opacity-50">Popularity · 12 months</span>
		<div>
			<svg
				width="150"
				height="30"
				style="display: block; overflow: visible;"
				data-om-id="jsx:/https:/8c265f73-daeb-4ae6-b6fa-e3ba342303e7.claudeusercontent.com/v1/design/projects/8c265f73-daeb-4ae6-b6fa-e3ba342303e7/serve/item-v2-shared.jsx:22300:370:5"
			>
				<polygon
					points="0,30 0,28.5 13.636363636363637,27.23617021276596 27.272727272727273,25.11063829787234 40.90909090909091,23.559574468085106 54.54545454545455,21.03191489361702 68.18181818181817,18.33191489361702 81.81818181818181,16.32127659574468 95.45454545454545,13.621276595744682 109.0909090909091,10.92127659574468 122.72727272727273,7.991489361702126 136.36363636363635,4.717021276595744 150,1.5 150,30"
					fill="color-mix(in oklch, oklch(80.3% 0.08 266deg) 12%, transparent)"
					data-om-id="jsx:/https:/8c265f73-daeb-4ae6-b6fa-e3ba342303e7.claudeusercontent.com/v1/design/projects/8c265f73-daeb-4ae6-b6fa-e3ba342303e7/serve/item-v2-shared.jsx:22392:371:7"
				></polygon>
				<polyline
					points="0,28.5 13.636363636363637,27.23617021276596 27.272727272727273,25.11063829787234 40.90909090909091,23.559574468085106 54.54545454545455,21.03191489361702 68.18181818181817,18.33191489361702 81.81818181818181,16.32127659574468 95.45454545454545,13.621276595744682 109.0909090909091,10.92127659574468 122.72727272727273,7.991489361702126 136.36363636363635,4.717021276595744 150,1.5"
					fill="none"
					stroke="oklch(80.3% 0.08 266deg)"
					stroke-width="1.25"
					stroke-linejoin="round"
					stroke-linecap="round"
					data-om-id="jsx:/https:/8c265f73-daeb-4ae6-b6fa-e3ba342303e7.claudeusercontent.com/v1/design/projects/8c265f73-daeb-4ae6-b6fa-e3ba342303e7/serve/item-v2-shared.jsx:22494:372:7"
				></polyline>
				<circle
					cx="150"
					cy="1.5"
					r="2"
					fill="oklch(80.3% 0.08 266deg)"
					data-om-id="jsx:/https:/8c265f73-daeb-4ae6-b6fa-e3ba342303e7.claudeusercontent.com/v1/design/projects/8c265f73-daeb-4ae6-b6fa-e3ba342303e7/serve/item-v2-shared.jsx:22618:373:7"
				></circle>
			</svg>
		</div>
	</div>
	<hr class="hr border-b-surface-200-800" />
	<div class="flex flex-col">
		<span class="text-sm opacity-50">TAGS · {item.tags.length}</span>

		<div>
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
		<span class="text-sm opacity-50">LANGS · {item.languages.length}</span>

		<div>
			{#each item.languages as lang, i}
				{#if i != 0}
					·
				{/if}
				{inToLangShort(lang)}
			{/each}
		</div>
	</div>
	<hr class="hr border-b-surface-200-800" />
	<div>
		<span class="text-sm opacity-50">Community Properties</span>
		<Properties {loggedIn} itemID={item.id} properties={item.properties} />
	</div>
</div>

<style>

</style>
