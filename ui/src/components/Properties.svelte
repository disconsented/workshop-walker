<script lang="ts">
	import { faChevronDown, faChevronUp, faLock } from '@fortawesome/free-solid-svg-icons';
	import { faSteamSymbol } from '@fortawesome/free-brands-svg-icons';
	import Icon from 'svelte-awesome';
	import SuggestProperty from '../routes/app/[id]/suggestProperty.svelte';
	import Property from '../routes/item/[item]/Property.svelte';

	interface Props {
		loggedIn: boolean; // Used for allowing voting
		itemID: string;
		properties: any[];
	}

	let { loggedIn = $bindable(), itemID, properties }: Props = $props();
	let first_props = $derived(properties?.slice(0, 6));
	let remaining_props = $derived(properties?.slice(6));
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
			></Property>
		{/each}
		{#if remaining_props.length > 0}
			{#if open}
				{#each remaining_props as prop}
					<Property
						{loggedIn}
						property={{ class: prop.out.class, value: prop.out.value, ...prop }}
						hideVote={false}
						{itemID}
					></Property>
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
				{/if}<span class="pl-1 cursor-pointer">{remaining_props.length} more properties</span>
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
	<div class="flex h-fit w-full grow-0 flex-col justify-end">
		{@render suggestProperty(itemID)}
	</div>
{/if}

{#snippet suggestProperty(itemID: string)}
	<SuggestProperty {itemID} />
{/snippet}
