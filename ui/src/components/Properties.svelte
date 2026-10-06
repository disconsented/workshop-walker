<script lang="ts">
	import { faChevronDown, faChevronUp, faLock } from '@fortawesome/free-solid-svg-icons';
	import { faSteamSymbol } from '@fortawesome/free-brands-svg-icons';
	import Icon from 'svelte-awesome';
	import Property from '../routes/item/[item]/[[slug]]/Property.svelte';
	import SuggestProperty from '../routes/app/[id]/[[slug]]/suggestProperty.svelte';

	interface Props {
		loggedIn: boolean; // Used for allowing voting
		itemID: string;
		properties: any[];
	}
	// We want to ignore/grey out any props that are too heavily downvoted
	const UPVOTE_THRESHOLD = -5;

	let { loggedIn = $bindable(), itemID, properties }: Props = $props();
	let sorted_props = $derived(
		properties
			?.toSorted((a, b) => b.upvote_count - a.upvote_count)
			.filter((value) => value.upvote_count > UPVOTE_THRESHOLD)
	);
	let first_props = $derived(sorted_props?.slice(0, 6));
	let remaining_props = $derived(sorted_props?.slice(6));
	let open = $state(false);
</script>

{#if first_props}
	<div class="flex w-full shrink-0 flex-wrap gap-1">
		{#each first_props as prop}
			<Property
				{loggedIn}
				property={{ class: prop.out.class, value: prop.out.value, ...prop }}
				hideVote={false}
				{itemID}
			/>
		{/each}
		{#if remaining_props.length > 0}
			{#if open}
				{#each remaining_props as prop}
					<Property
						{loggedIn}
						property={{ class: prop.out.class, value: prop.out.value, ...prop }}
						hideVote={false}
						{itemID}
					/>
				{/each}
			{/if}
			<button
				class="text-primary-500 ca w-full text-left text-sm"
				onclick={() => {
					open = !open;
				}}
			>
				{#if open}
					<Icon data={faChevronUp} class="fa-fw"></Icon>
				{:else}
					<Icon data={faChevronDown} class="fa-fw"></Icon>
				{/if}<span class="cursor-pointer pl-1">{remaining_props.length} more properties</span>
			</button>
		{/if}
	</div>
{/if}
{#if !loggedIn}
	<a
		href="/api/login?location={location.pathname + location.search}"
		class="btn btn-sm preset-outlined-primary-500 text-primary-500 flex w-full justify-between opacity-50"
		><span><Icon data={faLock} class="fa-fw"></Icon> Sign in to vote on properties</span>
		<span class="btn btn-sm preset-filled-primary-500"
			><Icon data={faSteamSymbol} class="fa-fw"></Icon> Sign in</span
		></a
	>
{:else}
	<div class="flex h-fit w-full grow-0 flex-col justify-end pt-1">
		{@render suggestProperty(itemID)}
	</div>
{/if}

{#snippet suggestProperty(itemID: string)}
	<SuggestProperty {itemID} />
{/snippet}
