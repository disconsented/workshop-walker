<script lang="ts">
	import Icon from 'svelte-awesome';
	import type { PageData } from '../../../../.svelte-kit/types/src/routes';
	import { faSteamSymbol } from '@fortawesome/free-brands-svg-icons';
	import { faArrowLeft, faLink } from '@fortawesome/free-solid-svg-icons';

	let { data }: { data: PageData } = $props();
	console.log('Hello, wolrd!', data);
	let item = data;

</script>

<div class="min-h-screen text-white p-8">
	<div class="max-w-9/10 mx-auto">
		<!-- Navigation Buttons -->
		<div class="flex gap-4 mb-8">
			<a href="/app/{item.appid}" class="btn preset-tonal-primary flex items-center gap-2">
				<Icon data={faArrowLeft} class="fa-fw"></Icon>
				Back to Search
			</a>
			<a href="/item/{item.id}" class="btn preset-filled-secondary-500">
				<Icon data={faLink} class="fa-fw"></Icon>
				Permalink
			</a>

			<a href="https://steamcommunity.com/sharedfiles/filedetails/?id={item.id}" target="_blank" rel="noopener noreferrer"
				 class="btn preset-tonal-success flex items-center gap-2">
				<Icon data={faSteamSymbol} class="fa-fw"></Icon>
				View on Steam Workshop
			</a>
		</div>

		<div class="grid grid-cols-1 md:grid-cols-3 gap-8">
			<!-- Main Info Column -->
			<div class="md:col-span-2 space-y-6">
				<!-- Title and Author -->
				<div class="card preset-filled-surface-100-900 border-[1px] border-surface-200-800 card-hover divide-surface-200-800 rounded-lg p-6">
					<h1 class="text-4xl font-bold mb-4">{item.title}</h1>
					<div class="flex items-center gap-3">
						<span class="text-xl">by</span>
						<a
							href={item.authorUrl}
							target="_blank"
							rel="noopener noreferrer"
							class="btn preset-tonal-primary"
						>
							{item.author} <Icon data={faSteamSymbol} class="fa-fw"></Icon>
						</a>
					</div>
				</div>

				<!-- Preview Image -->
				{#if item.preview_url}
					<div class="card preset-filled-surface-100-900 border-[1px] border-surface-200-800 card-hover rounded-lg p-6">
						<img
							src={item.preview_url}
							alt={item.title}
							class="rounded-lg max-w-full h-auto"
						/>
					</div>
				{:else}
					<div class="card preset-filled-surface-100-900 border-[1px] border-surface-200-800 card-hover rounded-lg p-6 text-center">
						No preview image available
					</div>
				{/if}

				<!-- Description -->
				<div class="card preset-filled-surface-100-900 border-[1px] border-surface-200-800 card-hover rounded-lg p-6">
					<h2 class="text-xl font-bold mb-4">Description</h2>
					<p class="whitespace-pre-wrap text-gray-300">{item.description}</p>
				</div>

				<!-- Tags -->
				<div class="card preset-filled-surface-100-900 border-[1px] border-surface-200-800 card-hover rounded-lg p-6">
					<h2 class="text-xl font-bold mb-4">Tags</h2>
					<div class="flex flex-wrap gap-2">
						{#each item.tags as tag(tag.id)}
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
				<div class="card preset-filled-surface-100-900 border-[1px] border-surface-200-800 card-hover rounded-lg p-6">
					<h2 class="text-xl font-bold mb-4">Dependencies</h2>
					{#if item.dependencies.length > 0}
						<ul class="space-y-3">
							{#each item.dependencies as dependency(dependency.id)}
								<li class="flex items-center gap-3">
									<a
										href="https://steamcommunity.com/sharedfiles/filedetails/?id={item.id}"
										target="_blank"
										rel="noopener noreferrer"
										class="btn preset-tonal-primary flex items-center gap-2 truncate whitespace-normal"
									>
										<Icon data={faSteamSymbol} class="fa-fw"></Icon>
										{dependency.title}
									</a>
								</li>
							{/each}
						</ul>
					{:else}
						<p class="text-gray-400">No dependencies</p>
					{/if}
				</div>

				<!-- Dependents -->
				<div class="card preset-filled-surface-100-900 border-[1px] border-surface-200-800 card-hover rounded-lg p-6">
					<h2 class="text-xl font-bold mb-4">Dependents</h2>
					{#if item.dependants.length > 0}
						<ul class="space-y-3">
							{#each item.dependants as dependent(dependent.id)}
								<li class="flex items-center gap-3">
									<a
										href="https://steamcommunity.com/sharedfiles/filedetails/?id={item.id}"
										target="_blank"
										rel="noopener noreferrer"
										class="btn preset-tonal-primary flex items-center gap-2 truncate whitespace-normal"
									>
										<Icon data={faSteamSymbol} class="fa-fw"></Icon>
										{dependent.title}
									</a>
								</li>
							{/each}
						</ul>
					{:else}
						<p class="text-gray-400">No dependents</p>
					{/if}
				</div>
			</div>
		</div>

		<!-- Additional Info -->
		<div class="mt-8 card preset-filled-surface-100-900 border-[1px] border-surface-200-800 card-hover rounded-lg p-6">
			<h2 class="text-xl font-bold mb-4">Additional Information</h2>
			<dl class="grid grid-cols-1 md:grid-cols-2 gap-x-8">
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
	class="card preset-filled-surface-100-900 border-[1px] border-surface-200-800 card-hover max-w-lg divide-surface-200-800 block divide-y overflow-hidden"
>
	<header>
		<img src={data.preview_url} class="aspect-[21/9] w-full grayscale hue-rotate-90" alt="banner" />
		<article class="space-y-4 p-4">
			<div>
				<h2 class="h6">{data.title}</h2>
				<a class="anchor btn preset-outlined"
					 href="https://steamcommunity.com/sharedfiles/filedetails/?id={data.id}">
					Steam
					<Icon data={faSteamSymbol} class="fa-fw"></Icon>
				</a>
				<a class="anchor btn preset-outlined"
					 href="/item/{data.id}">
					Permalink
					<Icon data={faLink} class="fa-fw"></Icon>
				</a>
			</div>
			<p class="opacity-60">
				{data.description}&mldr;
			</p>
		</article>
		<footer class="flex items-center justify-between gap-4 p-4">
			<small class="opacity-60">By AUTHOR</small>
			<small class="opacity-60">Last Updated {new Date().toLocaleDateString()}</small>
		</footer>
</div>

<div class="grid grid-cols-1 md:grid-cols-[auto_auto]">
	<div>
		<h2 class="h2">Dependencies:</h2>
		<ul class="list-inside  list-none space-y-2">
			{#each data.dependencies as dependency(dependency.id)}
				<li><span>{dependency.title}</span>
					<a class="anchor btn preset-outlined"
																							href="https://steamcommunity.com/sharedfiles/filedetails/?id={dependency.id}">
					Steam
					<Icon data={faSteamSymbol} class="fa-fw"></Icon>
				</a>
					<a class="anchor btn preset-outlined"
						 href="/item/{dependency.id}">
						Lookup
						<Icon data={faLink} class="fa-fw"></Icon>
					</a>
				</li>
			{/each}
		</ul>
	</div>
	<div>
		<h2 class="h2">Dependants:</h2>
		<ul class="list-inside  list-none space-y-2">
			{#each data.dependants as dependent(dependent.id)}
				<li><span>{dependent.title}</span>
					<a class="anchor btn preset-outlined"
						 href="https://steamcommunity.com/sharedfiles/filedetails/?id={dependent.id}">
						Steam
						<Icon data={faSteamSymbol} class="fa-fw"></Icon>
					</a>
					<a class="anchor btn preset-outlined"
						 href="/item/{dependent.id}">
						Lookup
						<Icon data={faLink} class="fa-fw"></Icon>
					</a>
				</li>
			{/each}
		</ul>
	</div>

</div>


	{/snippet}