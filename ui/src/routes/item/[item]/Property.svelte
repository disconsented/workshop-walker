<script lang="ts">
	import Icon from 'svelte-awesome';
	import {
		faCircleXmark,
		faClock,
		faLock,
		faThumbsDown,
		faThumbsUp
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

	const accentColour = (() => {
		console.log(property.class);
		switch (property.class.toLowerCase()) {
			case 'genre':
				return '--color-green-500';
			case 'theme':
				return '--color-blue-500';
			case 'type':
				return '--color-purple-500';
			case 'feature':
				return '--color-orange-500';
		}
	})();
</script>

<!-- Silly little hack to keep these classes from being removed by the compiler -->
<!--<div class="bg-(--color-green-500) bg-(--color-blue-500) bg-(--color-purple-500) bg-(--color-orange-500)"></div>-->
<!--<div class="text-(--color-green-500) text-(--color-blue-500) text-(--color-purple-500) text-(--color-orange-500)"></div>-->

<div class={['badge preset-outlined-surface-200-800 flex grow basis-0 overflow-clip p-0']}>
	<div class="w-4px inline-block h-full shrink-0 bg-({accentColour})">&nbsp</div>
	<div class="flex shrink grow justify-between" style="padding-block: calc(var(--spacing) * 1);">
		<div class="flex h-full">
			<div class="h-auto pr-2">
				{#if property.status === -1}
					<Icon data={faCircleXmark} class="text-error-500 flex-shrink-0" />
					Rejected
				{:else if property.status === 0}
					<Icon data={faClock} class="text-warning-500 flex-shrink-0" />
					Pending
				{/if}
				<span class="text-xs uppercase text-({accentColour})">{property.class}:</span>
				<span class="capitalize">{property.value}</span>
			</div>
		</div>

		{#if property.status === 1 && !hideVote}
			{@const score = property.vote_count ?? 0}
			{@const textColour = loggedIn
				? score > 0
					? 'text-success-500'
					: score < 0
						? 'text-error-500'
						: ''
				: 'text-gray-600'}
			<div class="flex">
				<span class="vr"></span>
				<!-- Voting -->
				<div class="ml-1 flex items-center gap-1 pr-1">
					{#if loggedIn}
						<button
							class={[voteState === 1 ? 'text-success-500' : 'hover:text-success-500', 'p-0.5']}
							disabled={!loggedIn}
							onclick={upvote}
						>
							<Icon data={faThumbsUp} class="text-xs" scale={0.8} />
						</button>
					{:else}
						<Icon data={faLock} class="text-xs text-gray-600" scale={0.8} />
					{/if}

					<span class={['min-w-[1ch] font-mono text-xs', textColour]}
						>{#if score > 0}+{/if}{score}</span
					>
					{#if loggedIn}
						<button
							class={[voteState === -1 ? 'text-error-500' : 'hover:text-error-500', 'p-0.5']}
							disabled={!loggedIn}
							onclick={downvote}
						>
							<Icon data={faThumbsDown} class="text-xs" scale={0.8} />
						</button>
					{/if}
				</div>
			</div>
		{/if}
	</div>
</div>
