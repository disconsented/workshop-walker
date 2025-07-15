<script lang="ts">
	import Icon from 'svelte-awesome';
	import {
		faChevronDown,
		faChevronUp,
		faCircleXmark,
		faClock
	} from '@fortawesome/free-solid-svg-icons';

	interface Props {
		loggedIn: boolean; // Used for allowing voting
		property: {
			class: string;
			value: string;
			upvote_count: number;
			vote_count: number;
			status: 1 | 0 | -1;
			vote_state: 1 | 0 | -1;
		};
		hideVote: boolean | undefined;
		itemID: string | undefined;
	}

	let { loggedIn = $bindable(), property, hideVote, itemID }: Props = $props();
	let request = undefined;
	let voteState = $state(property.vote_state);

	// Downvote the property or remove the vote
	const downvote = () => {
		if (voteState === -1) {
			// Remove
			voteState = 0;
			property.upvote_count++;
		} else {
			// Downvote
			voteState--;
			property.upvote_count--;
		}

		voteRequest();
	};
	// Upvote the property or remove the vote
	const upvote = () => {
		if (voteState === 1) {
			voteState = 0;
			property.upvote_count--;
		} else {
			voteState = 1;
			property.upvote_count++;
		}

		voteRequest();
	};

	const colour = (() => {
		switch (property.status) {
			case 1:
				return 'preset-tonal-primary';
			case -1:
				return 'preset-error-500';
			case 0:
			default:
				return 'preset-tonal-surface';
		}
	})();

	const voteRequest = () => {
		request = fetch('/api/vote/property', {
			method: voteState === 0 ? 'delete' : 'post',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({
				item: itemID,
				class: property.class,
				value: property.value,
				score: voteState
			})
		});
	};
</script>

<div class={['badge hover:bg-secondary-500/20 flex w-fit items-center gap-1', colour]}>
	{#if property.status === -1}
		<Icon data={faCircleXmark} class="text-error-500 flex-shrink-0" />
		Rejected
	{:else if property.status === 0}
		<Icon data={faClock} class="text-warning-500 flex-shrink-0" />
		Pending
	{/if}
	<span class="text-xs uppercase opacity-70">{property.class}:</span>
	<span class="capitalize">{property.value}</span>

	{#if property.status === 1 && !hideVote}
		<!-- Voting -->
		<div class="ml-1 flex items-center gap-1">
			<button
				class={[voteState === -1 ? 'text-error-500' : 'hover:text-error-500', 'p-0.5']}
				disabled={!loggedIn}
				onclick={downvote}
			>
				<Icon data={faChevronDown} class="text-xs" />
			</button>
			<span class="min-w-[1ch] font-mono text-xs">{property.upvote_count ?? 0}</span>
			<button
				class={[voteState === 1 ? 'text-success-500' : 'hover:text-success-500', 'p-0.5']}
				disabled={!loggedIn}
				onclick={upvote}
			>
				<Icon data={faChevronUp} class="text-xs" />
			</button>
		</div>
	{/if}
</div>
