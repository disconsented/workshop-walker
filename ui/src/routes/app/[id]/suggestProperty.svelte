<script lang="ts">
	import { SegmentedControl } from '@skeletonlabs/skeleton-svelte';
	import {
		faCheckCircle,
		faExclamationTriangle,
		faQuoteLeft,
		faQuoteRight,
		faSpinner
	} from '@fortawesome/free-solid-svg-icons';
	import Icon from 'svelte-awesome';
	import { exclamationCircle } from 'svelte-awesome/icons';
	import Property from '../../item/[item]/Property.svelte';

	interface Props {
		itemID: string;
	}

	let { itemID }: Props = $props();

	const classes = ['Type', 'Feature', 'Theme', 'Genre'];
	let open = $state(false);
	let prop_class = $state('Type');

	let property_details = {
		class: '',
		value: '',
		note: undefined,
		// svelte-ignore state_referenced_locally
		workshop_item: itemID
	};

	let request: Promise<Response> | null = $state(null);

	const submitNew = () => {
		property_details.class = prop_class;
		request = fetch('/api/property', {
			method: 'post',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify(property_details)
		});
	};

	const accentColour = $derived.by(() => {
		console.log(prop_class);
		switch (prop_class.toLowerCase()) {
			case 'genre':
				return 'green';
			case 'theme':
				return 'blue';
			case 'type':
				return 'purple';
			case 'feature':
				return 'orange';
		}
	});
</script>

<!-- Silly little hack to keep these classes from being removed by the compiler -->
<!--<div class="bg-(--color-green-100) bg-(--color-blue-100) bg-(--color-purple-100) bg-(--color-orange-100)"></div>-->
<!--<div class="text-(--color-green-500) text-(--color-blue-500) text-(--color-purple-500) text-(--color-orange-500)"></div>-->
<!--<div class="border-(--color-green-300) border-(--color-blue-300) border-(--color-purple-300) border-(--color-orange-300)"></div>-->
<div>
	{#if !open}
		<button
			class="btn w-full rounded-md border-1 border-dashed border-gray-300 p-1 text-center text-xs text-gray-300 opacity-80"
			onclick={() => (open = true)}
		>
			+ Suggest a property
		</button>
	{:else}
		{@render form(request)}
	{/if}
</div>

{#snippet form(request: Promise<Response> | null)}
	<form
		class="border-primary-50-950 flex w-full flex-row flex-wrap rounded-md border-1"
		onsubmit={(e) => {
			e.preventDefault();
			submitNew();
		}}
	>
		<div class="flex w-full grow">
			<SegmentedControl
				value={prop_class}
				onValueChange={(details) => (prop_class = details.value)}
				defaultValue="Type"
				class="w-full"
				required
			>
				<SegmentedControl.Control>
					<SegmentedControl.Indicator
						class="bg-(--color-{accentColour}-100) border-1 border-(--color-{accentColour}-300) opacity-80"
					/>
					{#each classes as prop_class}
						<SegmentedControl.Item value={prop_class}>
							<SegmentedControl.ItemText class={['text-xs uppercase']}
								>{prop_class}</SegmentedControl.ItemText
							>
							<SegmentedControl.ItemHiddenInput />
						</SegmentedControl.Item>
					{/each}
				</SegmentedControl.Control>
			</SegmentedControl>
		</div>
		<div class="w-full justify-center pt-1 text-center text-xs capitalize italic">
			<Icon data={faQuoteLeft} class="fa-fw"></Icon>
			{#if prop_class === 'Type'}
				Functional category of the mod, describing what it does to the game (e.g. overhaul, patch,
				media, QOL, expansion, utility, cheat, translation, or library)
			{:else if prop_class === 'Feature'}
				Core content or capabilities a mod adds or changes, distilled to broad concepts (e.g.
				weapons, textures, hunger system) with quantities, numbers, and descriptors stripped away.
			{:else if prop_class === 'Theme'}
				Core subject, setting, or narrative focus of the mod (e.g. Survival, Exploration, War,
				Steampunk, Cyberpunk, Western, Mystery).
			{:else if prop_class === 'Genre'}
				Narrative or aesthetic category of the mod's content, describing the kind of world or story
				it creates (e.g. Fantasy, Sci-Fi, Horror, Realism, Historical).
			{/if}
			<Icon data={faQuoteRight} class="fa-fw"></Icon>
		</div>
		<div class="flow-row flex flex-wrap gap-2 p-2" id="suggest-property">
			<div class="flex w-full">
				<div class="input-group bg-surface-50-950 w-full grid-cols-[auto_1fr_auto]">
					<div class="ig-cell text-xs uppercase text-(--color-{accentColour}-500)">
						{prop_class}
					</div>
					<input class="ig-input" type="text" required bind:value={property_details.value} />
				</div>
			</div>
			<div class="flex w-full" id="suggest-property-reasoning">
				<div class="input-group bg-surface-50-950 w-full grid-cols-[auto_1fr_auto]">
					<div class="ig-cell text-xs uppercase">
						<Icon data={faQuoteLeft} class="fa-fw text-xs text-gray-600" scale={0.8} />
					</div>
					<input
						class="ig-input bg-surface-50-950"
						type="text"
						placeholder="Reasoning - optional"
						bind:value={property_details.note}
					/>
				</div>
			</div>
			<footer class="flex w-full justify-between" id="suggest-property-footer">
				<button class="btn preset-tonal-surface" type="reset" onclick={() => (open = false)}
					>Cancel</button
				>
				{#if request}
					{#await request}
						<button class="btn preset-tonal-primary disabled" disabled>
							<Icon data={faSpinner} class="fa-fw text-xs" scale={0.8} pulse />
							Submitting...
						</button>
					{:then _}
						<button class="btn preset-tonal-primary">Suggest</button>
					{/await}
				{:else}
					<button class="btn preset-tonal-primary">Suggest</button>
				{/if}
			</footer>
			{#if request}
				<div class="flex w-full">
					{#await request then response}
						{@debug response}
						<div class="w-full">
							{#if response.ok}
								{@render success(property_details)}
							{:else if response.status >= 500}
								{@render error(response.statusText)}
							{:else}
								{@render conflict()}
							{/if}
						</div>
					{:catch error}
						{@debug error}
						<div class="w-full">
							{@render error(error.message)}
						</div>
					{/await}
				</div>
			{/if}
		</div>
	</form>
{/snippet}

{#snippet error(message)}
	<div class="border-error-200-800 bg-error-50-950/20 rounded-md border-1 p-2">
		<Icon data={faExclamationTriangle} class="fa-fw text-error-500 text-xs" scale={0.8} />
		Couldn't save your suggestion. The server didn't respond. {message}
		<button class="underline" onclick={() => submitNew()}>Retry</button>
	</div>
{/snippet}

{#snippet conflict()}
	<div class="border-warning-200-800 bg-warning-50-950/20 rounded-md border-1 p-2">
		<Icon data={exclamationCircle} class="fa-fw text-warning-500 text-xs" scale={0.8} />
		<span class="font-bold"
			><span class="uppercase">{property_details.class}</span>:
			<span>{property_details.value}</span></span
		>
		already exists on this item (or is too close to an existing item).
		<span class="underline">Upvote it</span>
	</div>
{/snippet}

{#snippet success(property)}
	<div>
		<div class="border-success-200-800 bg-success-50-950/20 rounded-md border-1 p-2">
			<Icon data={faCheckCircle} class="fa-fw text-success-500 text-xs" scale={0.8} />
			Suggested - now <span class="text-warning-500 font-bold">pending</span> review.
		</div>
		<div class="mr-2 flex w-fit shrink grow-0 pr-2">
			<Property
				loggedIn={true}
				property={{
					class: property.class,
					value: property.value,
					upvote_count: 0,
					vote_count: 0,
					status: 0,
					vote_state: 0
				}}
				itemID
				hideVote
			></Property>
		</div>
		<button
			onclick={() => {
				open = true;
				request = null;
			}}
		>
			+ Suggest another</button
		>
	</div>
{/snippet}
