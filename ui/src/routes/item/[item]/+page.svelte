<script lang="ts">
	import Icon from 'svelte-awesome';
	import { faSteamSymbol } from '@fortawesome/free-brands-svg-icons';
	import {
		fa1,
		faArrowLeft,
		faArrowRight,
		faCross,
		faEllipsis,
		faFilter,
		faLink,
		faThumbsDown,
		faThumbsUp,
		faTriangleExclamation
	} from '@fortawesome/free-solid-svg-icons';
	import { Switch } from '@skeletonlabs/skeleton-svelte';
	import { Accordion } from '@skeletonlabs/skeleton-svelte';
	import { Pagination } from '@skeletonlabs/skeleton-svelte';
	import TimeAgo from '$lib/timeAgo.svelte';
	import { Shadow } from 'svelte-loading-spinners';
	import PropertyPrompt from './PropertyPrompt.svelte';
	import Property from './Property.svelte';
	import { Modal } from '@skeletonlabs/skeleton-svelte';
	import { onNavigate } from '$app/navigation';

	let { data }: { data } = $props();
	console.log('Hello, wolrd!', data);
	let item = data;

	function whichLang(lang: Number): String {
		switch (lang) {
			case 1:
				return 'English';
			case 2:
				return 'Russian';
			case 3:
				return 'Chinese';
			case 4:
				return 'Japanese';
			case 5:
				return 'Korean';
			case 6:
				return 'Spanish';
			case 7:
				return 'Portuguese';
			default:
				return 'Unknown';
		}
	}

	function get_tags(data) {
		let out = new Map();
		data.dependants.forEach((e) =>
			e.tags.forEach((e) => {
				out.set(e.id, e);
			})
		);
		data.dependencies.forEach((e) =>
			e.tags.forEach((e) => {
				out.set(e.id, e);
			})
		);
		return Array.from(out).map(([name]) => name);
	}

	// Merge deps, flatten, dedup via set, back to array, map to names
	function get_languages(data) {
		return Array.from(
			new Set(
				[data.dependants.map((e) => e.languages), data.dependencies.map((e) => e.languages)]
					.flat()
					.flat()
			)
		).map((e) => whichLang(e));
	}

	const tags = get_tags(data);
	const languages = get_languages(data);

	let selectedTags = $state(['tags:Mod', 'tags:⟨1.6⟩']);
	let selectedLangs = $state(['English']);

	function filter(item) {
		const itemTags = item.tags.map((e) => e.id);
		const tags = selectedTags.every((e) => {
			console.log(itemTags, 'includes', e, ':', itemTags.includes(e));
			return itemTags.includes(e);
		});
		const langs = item.languages.every((e) => selectedLangs.includes(whichLang(e)));
		console.log('item', item.title, 'tags', tags, 'langs', langs, ':', !tags && !langs);
		return !(tags && langs);
	}

	let compact = $state(false);
	let filterPanel = $state(['open']);
	let filteredDependents = $derived(data.dependants.filter((e) => !filter(e)));
	let page = $state(1);
	let size = $state(20);
	const slicedSource = $derived((s) => s.slice((page - 1) * size, page * size));
	$inspect(slicedSource);
	let openPanels = $state(['relations', 'companions', 'description']);

	let loginModalState = $state(false);

	let location = $state(encodeURI(document.location.pathname));
	onNavigate((navigation) => {
		console.log(navigation);
		location = encodeURI(navigation.to.url.pathname);
	});

	const logged_in = document.cookie.includes('token_set=');
</script>

<svelte:head>
	<title>{item.title ? 'Workshop Walker - ' + item.title : 'Workshop Walker - Loading'}</title>
	{#await data then data}
		<meta property="og:title" content={'Workshop Walker - ' + data.title} />
		<meta property="og:type" content="website" />
		<meta property="og:url" content={window.location.href} />
		<meta property="og:image" content={data.preview_url} />
	{/await}
</svelte:head>

{@render loginModal()}
{#await data}
	<div class="flex h-full w-full place-content-center">
		<Shadow></Shadow>
	</div>
{:then item}
	{#if item.status}
		{@render errorCard(item)}
	{:else}
		<div class="min-h-screen p-8 text-white">
			<div class="mx-auto max-w-9/10">
				<!-- Navigation Buttons -->
				{@render navigation()}

				<div class="<!--md:grid-cols-3--> grid grid-cols-1 gap-8">
					<!-- Main Info Column -->
					<div class="space-y-6 md:col-span-2">
						<!-- Title and Author -->
						{@render titleCard()}
						<Accordion
							value={openPanels}
							onValueChange={(e) => (openPanels = e.value)}
							multiple
							padding=""
						>
							<Accordion.Item value="relations" panelPadding="">
								<!-- Control -->
								{#snippet lead()}
									<Icon data={faLink} class="fa-fw"></Icon>
								{/snippet}
								{#snippet control()}Relations{/snippet}
								<!-- Panel -->
								{#snippet panel()}{@render relations()}{/snippet}
							</Accordion.Item>
							<Accordion.Item value="description" panelPadding="">
								{#snippet lead()}
									<Icon data={faLink} class="fa-fw"></Icon>
								{/snippet}
								{#snippet control()}Description{/snippet}
								{#snippet panel()}{@render description()}{/snippet}
							</Accordion.Item>
						</Accordion>
					</div>
				</div>
			</div>
		</div>
	{/if}
{/await}

{#snippet linkSet(item)}
	<div
		class="card preset-filled-surface-100-900 border-surface-200-800 card-hover divide-surface-200-800 flex max-w-xs flex-col place-content-between
					divide-y overflow-hidden border-[1px]"
	>
		<!--Title-->
		<a
			class="h-full place-content-center hover:filter-none"
			href="/item/{item.id}"
			target="_self"
			rel="noopener noreferrer"
		>
			<img
				src={item.preview_url}
				alt="banner"
				loading="lazy"
				class="aspect-[21/9] w-full object-cover grayscale hue-rotate-90 hover:filter-none"
				class:hidden={compact}
				onerror={(e) =>
					(e.target.src =
						'https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/294100/header.jpg?t=1734154189')}
			/>
			<div class="preset-filled-surface-100-900 rounded-sm text-center">{item.title}</div>
		</a>
		<!--Tags-->
		<article class="flex flex-wrap gap-1" style="align-items: end">
			{#each item.tags as tag (tag.id)}
				<span class="badge preset-tonal-surface">
					{tag.display_name}
				</span>
			{:else}
				<span class="text-gray-400">No tags</span>
			{/each}
			{#each item.languages as lang}
				<span class="badge preset-tonal-surface">
					{whichLang(lang)}
				</span>
			{/each}
		</article>
		<!--Links-->
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
{/snippet}

{#snippet errorCard(value)}
	<div
		class="card preset-outlined-error-500 grid grid-cols-1 items-center gap-4 p-4 lg:grid-cols-[auto_1fr_auto]"
	>
		<Icon data={faTriangleExclamation} class="fa-fw"></Icon>
		<div>
			{#if value.status}
				<p class="font-bold">Error Code: {value.status}</p>
			{/if}
			{#if value.statusText}
				<p class="text-xs opacity-60">{value.statusText}</p>
			{/if}

			{#if value.body}
				<pre class="text-xs opacity-60">{value.body}</pre>
			{/if}

			{#if value.message}
				<p class="text-xs opacity-60">{value.message}</p>
			{/if}
		</div>
	</div>
{/snippet}

{#snippet titleCard()}
	<div
		class="card preset-filled-surface-100-900 border-surface-200-800 card-hover divide-surface-200-800 rounded-lg border-[1px] p-6"
	>
		<!--Title-->
		<h1 class="mb-4 text-4xl font-bold"><a href="#title">{item.title}</a></h1>
		<!--Details-->
		<div class="text-surface-600 dark:text-surface-400 flex flex-wrap items-center gap-4 text-sm">
			<span
				>Author: <a
					href="https://steamcommunity.com/profiles/{item.author.id}"
					target="_self"
					rel=""
					class="btn preset-tonal-primary"
				>
					{item.author.name}
					<Icon data={faSteamSymbol} class="fa-fw"></Icon>
				</a></span
			>
			<span>Last Updated: <TimeAgo date={item.last_updated}></TimeAgo></span>
			<span>Score: {Math.round(item.score * 100) / 100}</span>
			<span
				>Languages:
				{#each item.languages as lang}
					<span class="badge preset-outlined-primary-500">{whichLang(lang)}</span>
				{:else}
					<span class="badge preset-outlined-warning-500">Unknown</span>
				{/each}</span
			>
		</div>
		<!-- Preview Image -->
		{#if item.preview_url}
			<div class="pt-6">
				<img src={item.preview_url} alt={item.title} class="h-auto max-w-full rounded-lg" loading="lazy"/>
			</div>
		{:else}
			<div class="pt-6 text-xs opacity-60">No preview image available</div>
		{/if}
		<!-- Tags -->
		<div class="pt-6">
			<span>
				Tags:
				{#each item.tags as tag}
					<span class="badge preset-outlined-primary-500">{tag.display_name}</span>
				{:else}
					<span class="badge preset-outlined-warning-500">No Tags</span>
				{/each}
			</span>
		</div>
		<!-- Properties -->
		<div class="pt-6">
			<span>
				Properties:
				<div class="flex flex-row flex-wrap items-center gap-2">
					{#each item.properties as property}
						{@debug property}
						<Property
							loggedIn={logged_in}
							property={{ class: property.out.class, value: property.out.value, ...property }}
							hideVote={false}
							itemID={item.id}
						></Property>
					{:else}
						<span class="badge preset-outlined-warning-500">No Properties; Submit one?</span>
						{#if !logged_in}
							<PropertyPrompt
								loggedIn={logged_in}
								onClick={() => {
									loginModalState = true;
								}}
								item={item.id}
							></PropertyPrompt>
						{/if}
					{/each}
				</div>
			</span>
		</div>
		{#if logged_in}
			<!--Submit Properties-->
			<div class="mt-2 w-fit">
				<PropertyPrompt
					loggedIn={logged_in}
					onClick={() => {
						loginModalState = true;
					}}
					item={item.id}
				></PropertyPrompt>
			</div>
		{/if}
	</div>
{/snippet}

{#snippet navigation()}
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
{/snippet}

{#snippet description()}
	<div
		class="card preset-filled-surface-100-900 border-surface-200-800 card-hover rounded-lg border-[1px] p-6"
	>
		<h2 class="mb-4 text-xl font-bold"><a href="#description">Description</a></h2>
		<p class="whitespace-pre-wrap prose prose-invert max-w-none">{@html data.description}</p>
	</div>
{/snippet}

{#snippet relations()}
	<div
		class="card preset-filled-surface-100-900 border-surface-200-800 card-hover grid grid-cols-1 gap-4 rounded-lg border-[1px] p-6 md:grid-cols-4"
	>
		<!-- Controls -->
		<Accordion
			{filterPanel}
			onValueChange={(e) => (filterPanel = e.value)}
			collapsible
			classes="col-span-4"
		>
			<Accordion.Item value="open">
				<!-- Control -->
				{#snippet lead()}Filter{/snippet}
				{#snippet control()}
					<Icon data={faFilter} class="fa-fw"></Icon>
				{/snippet}
				<!-- Panel -->
				{#snippet panel()}
					<div class="col-span-4 grid w-full min-w-full grid-cols-2">
						<div class="label shrink-0">
							<span class="label-text">Tags</span>
							<div class="flex flex-row flex-wrap gap-3">
								{#each tags as tag}
									<label class="flex items-center space-x-2">
										<input
											name="tags"
											class="checkbox"
											type="checkbox"
											value={tag}
											bind:group={selectedTags}
										/>
										<p>{tag}</p>
									</label>
								{/each}
							</div>
						</div>
						<div class="label shrink-0">
							<span class="label-text">Languages</span>
							<div class="flex flex-row flex-wrap gap-3">
								{#each languages as lang}
									<label class="flex items-center space-x-2">
										<input
											name="langs "
											class="checkbox"
											type="checkbox"
											value={lang}
											bind:group={selectedLangs}
										/>
										<p>{lang}</p>
									</label>
								{/each}
							</div>
						</div>
						<div class="">
							<label class="label">
								<span class="label-text">Compact</span>
								<Switch
									name="compact"
									checked={compact}
									onCheckedChange={(e) => (compact = e.checked)}
								/>
							</label>
						</div>
					</div>
				{/snippet}
			</Accordion.Item>
			<hr class="hr" />
		</Accordion>
		<!-- Dependencies -->
		<div class="col-span-4">
			<h2 class="mb-4 text-xl font-bold">Dependencies</h2>
			{#if item.dependencies.length > 0}
				<div class="flex flex-row flex-wrap items-center items-center justify-between gap-2">
					{#each item.dependencies as dependency (dependency.id)}
						{@render linkSet(dependency)}
					{/each}
				</div>
			{:else}
				<p class="text-gray-400">No dependencies</p>
			{/if}
		</div>

		<section class="col-span-4 flex justify-between" class:hidden={filteredDependents.length === 0}>
			<select
				name="size"
				id="size"
				class="select max-w-[150px]"
				value={size}
				onchange={(e) => (size = Number(e.currentTarget.value))}
			>
				{#each [20] as v}
					<option value={v}>Items {v}</option>
				{/each}
				<option value={filteredDependents.length}>Show All</option>
			</select>
			<!-- Pagination -->
			<Pagination
				data={filteredDependents}
				{page}
				onPageChange={(e) => (page = e.page)}
				pageSize={size}
				onPageSizeChange={(e) => (size = e.pageSize)}
				siblingCount={4}
			>
				{#snippet labelEllipsis()}
					<Icon data={faEllipsis} class="fa-fw"></Icon>
				{/snippet}
				{#snippet labelNext()}
					<Icon data={faArrowRight} class="fa-fw"></Icon>
				{/snippet}
				{#snippet labelPrevious()}
					<Icon data={faArrowLeft} class="fa-fw"></Icon>
				{/snippet}
				{#snippet labelFirst()}
					<Icon data={fa1} class="fa-fw"></Icon>
				{/snippet}
				{#snippet labelLast()}
					<Icon data={faCross} class="fa-fw"></Icon>
				{/snippet}
			</Pagination>
		</section>
		<!-- Dependents -->
		<div class="col-span-4">
			<h2 class="mb-4 text-xl font-bold">
				{#if filteredDependents.length > 0}{filteredDependents.length}
				{/if} Dependents
			</h2>
			{#if filteredDependents.length > 0}
				<div class="flex flex-row flex-wrap items-center justify-between gap-2">
					{#each slicedSource(filteredDependents) as dependent (dependent.id)}
						{@render linkSet(dependent)}
					{/each}
				</div>
			{:else}
				<p class="text-gray-400">No dependents</p>
			{/if}
		</div>

		<footer class="col-span-4 flex justify-between" class:hidden={filteredDependents.length === 0}>
			<select
				name="size"
				id="size"
				class="select max-w-[150px]"
				value={size}
				onchange={(e) => (size = Number(e.currentTarget.value))}
			>
				{#each [20] as v}
					<option value={v}>Items {v}</option>
				{/each}
				<option value={filteredDependents.length}>Show All</option>
			</select>
			<!-- Pagination -->
			<Pagination
				data={filteredDependents}
				{page}
				onPageChange={(e) => (page = e.page)}
				pageSize={size}
				onPageSizeChange={(e) => (size = e.pageSize)}
				siblingCount={4}
			>
				{#snippet labelEllipsis()}
					<Icon data={faEllipsis} class="fa-fw"></Icon>
				{/snippet}
				{#snippet labelNext()}
					<Icon data={faArrowRight} class="fa-fw"></Icon>
				{/snippet}
				{#snippet labelPrevious()}
					<Icon data={faArrowLeft} class="fa-fw"></Icon>
				{/snippet}
				{#snippet labelFirst()}
					<Icon data={fa1} class="fa-fw"></Icon>
				{/snippet}
				{#snippet labelLast()}
					<Icon data={faCross} class="fa-fw"></Icon>
				{/snippet}
			</Pagination>
		</footer>
	</div>
{/snippet}

{#snippet companions()}
	<div
		class="card preset-filled-surface-100-900 border-surface-200-800 card-hover grid grid-cols-1 gap-4 rounded-lg border-[1px] p-6 md:grid-cols-4"
	>
		<h2 class="mb-4 text-xl font-bold"><a href="#relations">Relations</a></h2>
		<!-- Controls -->
		<Accordion
			{filterPanel}
			onValueChange={(e) => (filterPanel = e.value)}
			collapsible
			classes="col-span-4"
		>
			<Accordion.Item value="open">
				<!-- Control -->
				{#snippet lead()}Filter{/snippet}
				{#snippet control()}
					<Icon data={faFilter} class="fa-fw"></Icon>
				{/snippet}
				<!-- Panel -->
				{#snippet panel()}
					<div class="col-span-4 grid w-full min-w-full grid-cols-2">
						<div class="label shrink-0">
							<span class="label-text">Tags</span>
							<div class="flex flex-row flex-wrap gap-3">
								{#each tags as tag}
									<label class="flex items-center space-x-2">
										<input
											name="tags"
											class="checkbox"
											type="checkbox"
											value={tag}
											bind:group={selectedTags}
										/>
										<p>{tag}</p>
									</label>
								{/each}
							</div>
						</div>
						<div class="label shrink-0">
							<span class="label-text">Languages</span>
							<div class="flex flex-row flex-wrap gap-3">
								{#each languages as lang}
									<label class="flex items-center space-x-2">
										<input
											name="langs "
											class="checkbox"
											type="checkbox"
											value={lang}
											bind:group={selectedLangs}
										/>
										<p>{lang}</p>
									</label>
								{/each}
							</div>
						</div>
						<div class="">
							<label class="label">
								<span class="label-text">Compact</span>
								<Switch
									name="compact"
									checked={compact}
									onCheckedChange={(e) => (compact = e.checked)}
								/>
							</label>
						</div>
					</div>
				{/snippet}
			</Accordion.Item>
			<hr class="hr" />
		</Accordion>
		<!-- Dependencies -->
		<div class="col-span-4">
			<h2 class="mb-4 text-xl font-bold">Dependencies</h2>
			{#if item.dependencies.length > 0}
				<div class="wrap flex flex-col">
					{#each item.dependencies as dependency (dependency.id)}
						{@render companionCard(dependency)}
					{/each}
				</div>
			{:else}
				<p class="text-gray-400">No dependencies</p>
			{/if}
		</div>

		<section class="col-span-4 flex justify-between" class:hidden={filteredDependents.length === 0}>
			<select
				name="size"
				id="size"
				class="select max-w-[150px]"
				value={size}
				onchange={(e) => (size = Number(e.currentTarget.value))}
			>
				{#each [20] as v}
					<option value={v}>Items {v}</option>
				{/each}
				<option value={filteredDependents.length}>Show All</option>
			</select>
			<!-- Pagination -->
			<Pagination
				data={filteredDependents}
				{page}
				onPageChange={(e) => (page = e.page)}
				pageSize={size}
				onPageSizeChange={(e) => (size = e.pageSize)}
				siblingCount={4}
			>
				{#snippet labelEllipsis()}
					<Icon data={faEllipsis} class="fa-fw"></Icon>
				{/snippet}
				{#snippet labelNext()}
					<Icon data={faArrowRight} class="fa-fw"></Icon>
				{/snippet}
				{#snippet labelPrevious()}
					<Icon data={faArrowLeft} class="fa-fw"></Icon>
				{/snippet}
				{#snippet labelFirst()}
					<Icon data={fa1} class="fa-fw"></Icon>
				{/snippet}
				{#snippet labelLast()}
					<Icon data={faCross} class="fa-fw"></Icon>
				{/snippet}
			</Pagination>
		</section>
		<!-- Dependents -->
		<div class="col-span-4">
			<h2 class="mb-4 text-xl font-bold">
				{#if filteredDependents.length > 0}{filteredDependents.length}
				{/if} Dependents
			</h2>
			{#if filteredDependents.length > 0}
				<div class="flex flex-row flex-wrap items-center justify-between gap-2">
					<!--{@debug slicedSource}-->
					{#each slicedSource(filteredDependents) as dependent (dependent.id)}
						{@render companionCard(dependent)}
					{/each}
				</div>
			{:else}
				<p class="text-gray-400">No dependents</p>
			{/if}
		</div>

		<footer class="col-span-4 flex justify-between" class:hidden={filteredDependents.length === 0}>
			<select
				name="size"
				id="size"
				class="select max-w-[150px]"
				value={size}
				onchange={(e) => (size = Number(e.currentTarget.value))}
			>
				{#each [20] as v}
					<option value={v}>Items {v}</option>
				{/each}
				<option value={filteredDependents.length}>Show All</option>
			</select>
			<!-- Pagination -->
			<Pagination
				data={filteredDependents}
				{page}
				onPageChange={(e) => (page = e.page)}
				pageSize={size}
				onPageSizeChange={(e) => (size = e.pageSize)}
				siblingCount={4}
			>
				{#snippet labelEllipsis()}
					<Icon data={faEllipsis} class="fa-fw"></Icon>
				{/snippet}
				{#snippet labelNext()}
					<Icon data={faArrowRight} class="fa-fw"></Icon>
				{/snippet}
				{#snippet labelPrevious()}
					<Icon data={faArrowLeft} class="fa-fw"></Icon>
				{/snippet}
				{#snippet labelFirst()}
					<Icon data={fa1} class="fa-fw"></Icon>
				{/snippet}
				{#snippet labelLast()}
					<Icon data={faCross} class="fa-fw"></Icon>
				{/snippet}
			</Pagination>
		</footer>
	</div>
{/snippet}

{#snippet companionCard(item)}
	<div
		class="card preset-filled-surface-100-900 border-surface-200-800 card-hover divide-surface-200-800 border-l-primary-500 flex max-w-xs flex-col
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
				<div class="preset-filled-surface-100-900 rounded-sm text-center font-medium text-balance">
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
{/snippet}

{#snippet loginModal()}
	<Modal
		open={loginModalState}
		onOpenChange={(e) => (loginModalState = e.open)}
		triggerBase="btn preset-tonal"
		contentBase="card bg-surface-100-900 p-4 space-y-4 shadow-xl max-w-screen-sm"
		backdropClasses="backdrop-blur-sm"
	>
		{#snippet content()}
			<header class="flex justify-between">
				<h2 class="h2">Please Login To Continue</h2>
			</header>
			<footer class="flex justify-end gap-4">
				<a
					href="/api/login?location={location}"
					aria-label="Sign In Through Steam"
					class="mt-auto mb-auto"
				>
					<img
						alt="Sign In Through Steam"
						src="https://steamcdn-a.akamaihd.net/steamcommunity/public/images/steamworks_docs/english/sits_small.png"
					/>
				</a>
				<button type="button" class="btn preset-tonal" onclick={() => (loginModalState = false)}
					>Cancel</button
				>
			</footer>
		{/snippet}
	</Modal>
{/snippet}
