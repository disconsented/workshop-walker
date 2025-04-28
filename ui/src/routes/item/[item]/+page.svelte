<script lang="ts">
	import Icon from 'svelte-awesome';
	import type { PageData } from '../../../../.svelte-kit/types/src/routes';
	import { faSteamSymbol } from '@fortawesome/free-brands-svg-icons';
	import { faArrowLeft, faLink } from '@fortawesome/free-solid-svg-icons';

	let { data }: { data: PageData } = $props();
	console.log('Hello, wolrd!', data);
	let item = data;
</script>

<div class="min-h-screen p-8 text-white">
	<div class="mx-auto max-w-9/10">
		<!-- Navigation Buttons -->
		<div class="mb-8 flex gap-4">
			<a href="/app/{item.appid}" class="btn preset-tonal-primary flex items-center gap-2">
				<Icon data={faArrowLeft} class="fa-fw"></Icon>
				Back to Search
			</a>
			<a href="/item/{item.id}" class="btn preset-filled-secondary-500">
				<Icon data={faLink} class="fa-fw"></Icon>
				Permalink
			</a>

			<a
				href="https://steamcommunity.com/sharedfiles/filedetails/?id={item.id}"
				target="_blank"
				rel="noopener noreferrer"
				class="btn preset-tonal-success flex items-center gap-2"
			>
				<Icon data={faSteamSymbol} class="fa-fw"></Icon>
				View on Steam Workshop
			</a>
		</div>

		<div class="grid grid-cols-1 gap-8 md:grid-cols-3">
			<!-- Main Info Column -->
			<div class="space-y-6 md:col-span-2">
				<!-- Title and Author -->
				<div
					class="card preset-filled-surface-100-900 border-surface-200-800 card-hover divide-surface-200-800 rounded-lg border-[1px] p-6"
				>
					<h1 class="mb-4 text-4xl font-bold">{item.title}</h1>
					<div class="flex items-center gap-3">
						<span class="text-xl">by</span>
						<a
							href="https://steamcommunity.com/profiles/{item.author}"
							target="_blank"
							rel=""
							class="btn preset-tonal-primary"
						>
							Unknown Name
							<Icon data={faSteamSymbol} class="fa-fw"></Icon>
						</a>
					</div>
				</div>

				<!-- Preview Image -->
				{#if item.preview_url}
					<div
						class="card preset-filled-surface-100-900 border-surface-200-800 card-hover rounded-lg border-[1px] p-6"
					>
						<img src={item.preview_url} alt={item.title} class="h-auto max-w-full rounded-lg" />
					</div>
				{:else}
					<div
						class="card preset-filled-surface-100-900 border-surface-200-800 card-hover rounded-lg border-[1px] p-6 text-center"
					>
						No preview image available
					</div>
				{/if}

				<!-- Description -->
				<div
					class="card preset-filled-surface-100-900 border-surface-200-800 card-hover rounded-lg border-[1px] p-6"
				>
					<h2 class="mb-4 text-xl font-bold">Description</h2>
					<p class="whitespace-pre-wrap text-gray-300">{@html data.description}</p>
				</div>

				<!-- Tags -->
				<div
					class="card preset-filled-surface-100-900 border-surface-200-800 card-hover rounded-lg border-[1px] p-6"
				>
					<h2 class="mb-4 text-xl font-bold">Tags</h2>
					<div class="flex flex-wrap gap-2">
						{#each item.tags as tag (tag.id)}
							<button type="button" class="chip preset-filled-primary-500">
								{tag.display_name}
							</button>
						{:else}
							<span class="text-gray-400">No tags</span>
						{/each}
					</div>
				</div>
			</div>

			<!-- Dependencies Column -->
			<div class="space-y-6">
				<!-- Dependencies -->
				<div
					class="card preset-filled-surface-100-900 border-surface-200-800 card-hover rounded-lg border-[1px] p-6"
				>
					<h2 class="mb-4 text-xl font-bold">Dependencies</h2>
					{#if item.dependencies.length > 0}
						<ul class="space-y-3">
							{#each item.dependencies as dependency (dependency.id)}
								{@render linkSet(dependency)}
							{/each}
						</ul>
					{:else}
						<p class="text-gray-400">No dependencies</p>
					{/if}
				</div>

				<!-- Dependents -->
				<div
					class="card preset-filled-surface-100-900 border-surface-200-800 card-hover rounded-lg border-[1px] p-6"
				>
					<h2 class="mb-4 text-xl font-bold">Dependents</h2>
					{#if item.dependants.length > 0}
						<ul class="space-y-3">
							{#each item.dependants as dependent (dependent.id)}
								{@render linkSet(dependent)}
							{/each}
						</ul>
					{:else}
						<p class="text-gray-400">No dependents</p>
					{/if}
				</div>
			</div>
		</div>

		<!-- Additional Info -->
		<div
			class="card preset-filled-surface-100-900 border-surface-200-800 card-hover mt-8 rounded-lg border-[1px] p-6"
		>
			<h2 class="mb-4 text-xl font-bold">Additional Information</h2>
			<dl class="grid grid-cols-1 gap-x-8 md:grid-cols-2">
				<div>
					<dt class="text-sm text-gray-400">App ID</dt>
					<dd class="text-lg">{item.appid}</dd>
				</div>
				<div>
					<dt class="text-sm text-gray-400">Last Updated</dt>
					<dd class="text-lg">
						{item.last_updated}
					</dd>
				</div>
				<div>
					<dt class="text-sm text-gray-400">Language</dt>
					<dd class="text-lg">{item.language}</dd>
				</div>
			</dl>
		</div>
	</div>
</div>
{#snippet old()}
	<div
		class="card preset-filled-surface-100-900 border-surface-200-800 card-hover divide-surface-200-800 block max-w-lg divide-y overflow-hidden border-[1px]"
	>
		<header>
			<img
				src={data.preview_url}
				class="aspect-[21/9] w-full grayscale hue-rotate-90"
				alt="banner"
			/>
			<article class="space-y-4 p-4">
				<div>
					<h2 class="h6">{data.title}</h2>
					<a
						class="anchor btn preset-outlined"
						href="https://steamcommunity.com/sharedfiles/filedetails/?id={data.id}"
					>
						Steam
						<Icon data={faSteamSymbol} class="fa-fw"></Icon>
					</a>
					<a class="anchor btn preset-outlined" href="/item/{data.id}">
						Permalink
						<Icon data={faLink} class="fa-fw"></Icon>
					</a>
				</div>
				<p class="opacity-60">
					{@html data.description}
				</p>
			</article>
			<footer class="flex items-center justify-between gap-4 p-4">
				<small class="opacity-60">By AUTHOR</small>
				<small class="opacity-60">Last Updated {new Date().toLocaleDateString()}</small>
			</footer>
		</header>
	</div>

	<div class="grid grid-cols-1 md:grid-cols-[auto_auto]">
		<div>
			<h2 class="h2">Dependencies:</h2>
			<ul class="list-inside list-none space-y-2">
				{#each data.dependencies as dependency (dependency.id)}
					<li>
						<span>{dependency.title}</span>
						<a
							class="anchor btn preset-outlined"
							href="https://steamcommunity.com/sharedfiles/filedetails/?id={dependency.id}"
						>
							Steam
							<Icon data={faSteamSymbol} class="fa-fw"></Icon>
						</a>
						<a class="anchor btn preset-outlined" href="/item/{dependency.id}">
							Lookup
							<Icon data={faLink} class="fa-fw"></Icon>
						</a>
					</li>
				{/each}
			</ul>
		</div>
		<div>
			<h2 class="h2">Dependants:</h2>
			<ul class="list-inside list-none space-y-2">
				{#each data.dependants as dependent (dependent.id)}
					<li>
						<span>{dependent.title}</span>
						<a
							class="anchor btn preset-outlined"
							href="https://steamcommunity.com/sharedfiles/filedetails/?id={dependent.id}"
						>
							Steam
							<Icon data={faSteamSymbol} class="fa-fw"></Icon>
						</a>
						<a class="anchor btn preset-outlined" href="/item/{dependent.id}">
							Lookup
							<Icon data={faLink} class="fa-fw"></Icon>
						</a>
					</li>
				{/each}
			</ul>
		</div>
	</div>
{/snippet}


{#snippet linkSet(item)}
	<li class="flex flex-col">
		<div class="input-group grid-cols-[auto_auto]">
			<a
				href="https://steamcommunity.com/sharedfiles/filedetails/?id={item.id}"
				target="_blank"
				rel="noopener noreferrer"
				class="btn preset-tonal-primary flex items-center gap-2 truncate whitespace-normal">
				<Icon data={faSteamSymbol} class="fa-fw"></Icon>
				Workshop
			</a>
			<a
				href="/item/{item.id}"
				target="_blank"
				rel="noopener noreferrer"
				class="btn preset-tonal-primary flex items-center gap-2 truncate whitespace-normal"
			>
				<Icon data={faLink} class="fa-fw"></Icon>
				View
			</a>
		</div>
		<div class="input-group grid-cols-[auto]">
			<div class="ig-cell preset-tonal">{item.title}</div>
		</div>
	</li>
{/snippet}