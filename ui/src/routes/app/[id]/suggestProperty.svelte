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


	const classes = ['type', 'feature', 'theme', 'genre'];
	let open = $state(false);
	let prop_class = $state('type');

	let property_details = {
		class: '',
		value: '',
		note: undefined
	};

	let request: Promise<Response> | null = $state(null);

	const submitNew = () => {
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

{#if !open}
	<button
		class="btn border-1 border-dashed border-gray-300 text-xs text-gray-300 opacity-80 rounded-md text-center p-1 w-full"
		onclick={() => open = true}>
		+ Suggest a property
	</button>
{:else}
	<form class="flex flex-row flex-wrap w-full border-1 rounded-md border-primary-50-950">
		<div class="flex w-full grow">
			<SegmentedControl value={prop_class} onValueChange={(details) => (prop_class = details.value)}
			                  defaultValue="type" class="w-full" required>
				<SegmentedControl.Control>
					<SegmentedControl.Indicator
						class="bg-(--color-{accentColour}-100) border-1 border-(--color-{accentColour}-300) opacity-80" />
					{#each classes as prop_class}
						<SegmentedControl.Item value={prop_class}>
							<SegmentedControl.ItemText class={["uppercase text-xs"]}>{prop_class}</SegmentedControl.ItemText>
							<SegmentedControl.ItemHiddenInput />
						</SegmentedControl.Item>
					{/each}
				</SegmentedControl.Control>
			</SegmentedControl>
		</div>
		<div class="text-xs justify-center italic capitalize text-center w-full pt-1">
			<Icon data={faQuoteLeft} class="fa-fw"></Icon>
			{#if prop_class === "type"}
				what kind of item it is
			{:else if prop_class === "feature"}
				what features the item has
			{:else if prop_class === "theme"}
				core subject, setting or narrative focus
			{:else if prop_class === "genre"}
				Narrative or atheistic category of a mods content
			{/if}
			<Icon data={faQuoteRight} class="fa-fw"></Icon>
		</div>
		<div class="flex flow-row flex-wrap p-2 gap-2" id="suggest-property">
			<div class="flex w-full">
				<div class="input-group grid-cols-[auto_1fr_auto] w-full bg-surface-50-950">
					<div class="ig-cell uppercase text-xs text-(--color-{accentColour}-500)">{prop_class}</div>
					<input class="ig-input" type="text" required />
				</div>
			</div>
			<div class="flex w-full" id="suggest-property-reasoning">
				<div class="input-group grid-cols-[auto_1fr_auto] w-full bg-surface-50-950">
					<div class="ig-cell uppercase text-xs">
						<Icon data={faQuoteLeft} class="text-xs text-gray-600 fa-fw" scale={0.8} />
					</div>
					<input class="ig-input bg-surface-50-950" type="text" placeholder="Reasoning - optional" />
				</div>
			</div>
			<footer class="flex w-full justify-between" id="suggest-property-footer">
				<button class="btn preset-tonal-surface" type="reset" onclick={() => open = false}>Cancel</button>
				{#if request}
					<button class="btn preset-tonal-primary disabled" disabled>
						<Icon data={faSpinner} class="text-xs fa-fw" scale={0.8} pulse />
						Submitting...
					</button>
				{:else}
					<button class="btn preset-tonal-primary">Suggest</button>
				{/if}
			</footer>
		</div>
	</form>
	<div class="w-full">
		<div class="border-1 border-error-200-800 bg-error-50-950/20 rounded-md p-2">
			<Icon data={faExclamationTriangle} class="text-xs fa-fw text-error-500" scale={0.8} />
			Couldn't save your suggestion. The server didn't respond.
			<button class="underline">Retry</button>
		</div>

		<div class="border-1 border-warning-200-800 bg-warning-50-950/20 rounded-md p-2">
			<Icon data={exclamationCircle} class="text-xs fa-fw text-warning-500" scale={0.8} />
			<span class="font-bold"><span class="uppercase">Feature</span>: <span>Weapons</span></span> already exists on this
			item.
			<button class="underline">Upvote it</button>
		</div>

		<div class="border-1 border-warning-200-800 bg-warning-50-950/20 rounded-md p-2">
			<Icon data={exclamationCircle} class="text-xs fa-fw text-warning-500" scale={0.8} />
			<span class="font-bold"><span class="uppercase">Feature</span>: <span>Weapons</span></span> is too close to existing item. <span class="font-bold"><span class="uppercase">Feature</span>: <span>Weapon</span></span>
			<button class="underline">Upvote it</button> or submit something else
		</div>

		<div>
			<div class="border-1 border-success-200-800 bg-success-50-950/20 rounded-md p-2">
				<Icon data={faCheckCircle} class="text-xs fa-fw text-success-500" scale={0.8} />
				Suggested - now <span class="text-warning-500 font-bold">pending</span> review.
			</div>
			<div class="flex shrink grow-0 w-fit mr-2 pr-2">
				<Property loggedIn={true}
				          property={{class: "type", value: "test", upvote_count: 0, vote_count: 0, status: 0, vote_state: 0}}
				          itemID={itemid} hideVote></Property>
			</div>
			<button> + Suggest another</button>
		</div>

	</div>
{/if}

