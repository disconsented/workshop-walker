<script lang="ts">
	import AppCard from '$lib/app_card.svelte';
	import Logotype from '$lib/logotype.svelte';
	import Icon from 'svelte-awesome';
	import { faGithub, faSteam } from '@fortawesome/free-brands-svg-icons';
	import { faExternalLink, faGamepad, faTag } from '@fortawesome/free-solid-svg-icons';

	let { data } = $props();
	console.log(data);
</script>

<svelte:head>
	<title>Workshop Walker - Steam Items</title>
	<meta property="og:title" content="Workshop Walker" />
	<meta property="og:type" content="website" />
	<meta property="og:url" content={window.location.href} />
</svelte:head>

<div class="flex flex-col">
	{@render hero()}
	{@render cards()}
</div>

{#snippet hero()}
	<div
		class="border-surface-200 from-surface-100-900 flex flex-col gap-4 border-t-1 border-b-1 bg-linear-to-t bg-gradient-to-b to-transparent p-4 py-8"
	>
		<div class="text-xl">
			<Logotype></Logotype>
		</div>

		<div class="opacity-50">
			Browse Steam Workshop items across {data.items.length} supported games. Filter by tags, vote on
			properties, and find exactly what you need.
		</div>
		<div class="flex flex-row gap-4 text-sm opacity-50">
			<div class="flex place-items-center gap-1">
				<Icon data={faGamepad} class="fa-fw"></Icon>
				{data.items.length} Games
			</div>
			<div class="flex place-items-center gap-1">
				<Icon data={faTag} class="fa-fw"></Icon>
				Community Tags
			</div>
			<div class="flex place-items-center gap-1">
				<Icon data={faSteam} class="fa-fw"></Icon>
				Steam Workshop
			</div>
		</div>
	</div>
{/snippet}

{#snippet cards()}
	<div class="flex grow-0 flex-wrap gap-4 py-4">
		{#each data.items as app}
			<AppCard
				appid={app.id}
				image_url={app.banner}
				description={app.description}
				developer={app.developer}
				name={app.name}
				url={undefined}
			></AppCard>
		{/each}
		<a
			href="https://github.com/disconsented/workshop-walker/discussions"
			aria-label="Suggest A Game"
			class="card card-hover border-surface-200-800 text-surface-500 group hover:border-surface-300-700
			hover:text-surface-400-600 hover:bg-surface-contrast-950-50 flex flex-col flex-wrap place-items-center
			border-1 border-dashed p-4"
		>
			<div class="btn preset-outlined-surface-200-800 my-4 p-2 opacity-50 group-hover:opacity-100">
				<Icon data={faGithub} class="fa-fw"></Icon>
			</div>
			<div class="font-bold opacity-50 group-hover:opacity-100">Suggest a game</div>
			<div class="text-sm opacity-50">
				Know a game with a great Workshop? Open an issue on GitHub
			</div>
			<div class="pt-2 text-sm opacity-50 group-hover:opacity-100">
				<Icon data={faExternalLink} class="fa-fw"></Icon>
				Open GitHub Issue
			</div>
		</a>
	</div>
{/snippet}
