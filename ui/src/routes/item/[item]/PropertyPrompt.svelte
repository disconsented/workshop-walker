<script lang="ts">
	import {
		faArrowRight,
		faArrowRotateLeft,
		faCheck,
		faPlus,
		faRightToBracket,
		faSpinner,
		faXmark
	} from '@fortawesome/free-solid-svg-icons';
	import Icon from 'svelte-awesome';

	interface Props {
		loggedIn: boolean; // Used for allowing voting
		onClick: () => void;
		item: String;
	}

	let { loggedIn = $bindable(), onClick, item }: Props = $props();
	let request: Promise<any> | undefined = $state(undefined);
	let propClass = $state();
	let value = $state();

	const submitNew = () => {
		request = fetch('/api/property', {
			method: 'post',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({
				app_id: 0,
				class: propClass,
				value: value,
				note: undefined,
				workshop_item: item
			})
		});
	};

	const reset = () => {
		request = undefined;
		value = '';
	};
</script>

{#if loggedIn}
	{#if request}
		{#await request}
			<div class="input-group grid-cols-[auto_1fr_auto]">
				<div class="ig-btn preset-tonal">
					<Icon data={faPlus} class="fa-fw"></Icon>
				</div>
				<div class="ig-input preset-tonal text-center">
					<Icon data={faSpinner} class="fa-fw" pulse></Icon>
				</div>
				<div class="ig-btn preset-tonal">
					<Icon data={faRightToBracket} class="fa-fw"></Icon>
				</div>
			</div>
		{:then value}
			{@debug value}
			{#if !value.ok}
				<div class="input-group grid-cols-[auto_1fr_auto]">
					<button class="ig-btn preset-tonal-warning" onclick={reset}>
						<Icon data={faXmark} class="fa-fw"></Icon>
					</button>
					<div class="ig-input preset-tonal-warning text-center">
						{value.statusText}
						{#await value.text() then body}- {body}{/await}
					</div>
					<button class="ig-btn preset-tonal-warning" onclick={reset}>
						<Icon data={faArrowRotateLeft} class="fa-fw"></Icon>
					</button>
				</div>
			{:else if true}
				<!--Accepted-->
				<div class="input-group grid-cols-[auto_1fr_auto]">
					<button class="ig-btn preset-tonal-success" onclick={reset}>
						<Icon data={faCheck} class="fa-fw"></Icon>
					</button>
					<div class="ig-input preset-tonal-success text-center">Submitted: Pending Approval</div>
					<button class="ig-btn preset-tonal-success" onclick={reset}>
						<Icon data={faArrowRight} class="fa-fw"></Icon>
					</button>
				</div>
			{:else}
				<!-- Rejected -->
				<div class="input-group grid-cols-[auto_1fr_auto]">
					<button class="ig-btn preset-tonal-warning" onclick={reset}>
						<Icon data={faXmark} class="fa-fw"></Icon>
					</button>
					<div class="ig-input preset-tonal-warning text-center">Rejected: {'reason'}</div>
					<button class="ig-btn preset-tonal-warning" onclick={reset}>
						<Icon data={faArrowRotateLeft} class="fa-fw"></Icon>
					</button>
				</div>
			{/if}
		{:catch error}
			<!-- promise was rejected -->
			<p>Something went wrong: {error.message}</p>
		{/await}
	{:else}
		<form
			class="input-group grid-cols-[auto_auto_1fr_auto_auto]"
			onsubmit={(e) => {
				e.preventDefault();
				submitNew();
			}}
		>
			<div class="ig-btn preset-tonal">
				<Icon data={faPlus} class="fa-fw"></Icon>
			</div>
			<select class="ig-select capitalize" bind:value={propClass}>
				<option value="Genre">Genre</option>
				<option value="Theme">Theme</option>
				<option value="Type">Type</option>
				<option value="Feature">Feature</option>
			</select>
			<input class="ig-input" type="text" bind:value />
			<div class="ig-btn preset-tonal">
				<button>
					Submit
					<Icon data={faRightToBracket} class="fa-fw"></Icon>
				</button>
			</div>
		</form>
	{/if}
{:else}
	<div class="input-group grid-cols-[auto_1fr_auto]">
		<button class="ig-cell preset-tonal" onclick={onClick}>
			<Icon data={faPlus} class="fa-fw"></Icon>
		</button>
		<button class="ig-btn ig-cell preset-filled" type="button" onclick={onClick}
			>Login To Suggest</button
		>
		<button class="ig-cell preset-tonal" onclick={onClick}>
			<Icon data={faRightToBracket} class="fa-fw"></Icon>
		</button>
	</div>
{/if}
