<script lang="ts">
	import { lastUpdated } from '../routes/app/[id]/store.svelte';

	const selectChange = (e) => {
		if (e.target.value) {
			const date = new Date();
			date.setDate(date.getDate() - e.target.value);
			lastUpdated.v = date;
			return;
		}

		lastUpdated.v = undefined;
	};

	const inputChange = (e) => {
		console.log('inputChange', e);
		if (e.target.value) {
			if (!lastUpdated.v) {
				lastUpdated.v = new Date();
			}
			lastUpdated.v.setDate(e.target.value);
			return;
		}

		lastUpdated.v = undefined;
	};


	// let selectedTime = $state(undefined);
	let localDate = $derived.by(() => {
		console.log('derive!');
		if (lastUpdated.v) {
			return lastUpdated.v.toISOString().split('T')[0];
		}
		return undefined;
	});
</script>
<!-- Style override for colour: https://github.com/tailwindlabs/tailwindcss/issues/14499-->
<div class="input-group grid-cols-[1fr_auto] input rounded-lg border" style="border-color: oklch(0.551 0.027 264.364)">
	<input
		type="date"
		class="ig-input"
		bind:value={localDate}
		onchange={inputChange}
	/>
	<select class="ig-select" onchange={selectChange}>
		<option value={1}>Today</option>
		<option value={7}>One Week</option>
		<option value={30}>One Month</option>
		<option value={90}>Three Months</option>
		<option value={180}>Six Months</option>
		<option value={365}>One Year</option>
		<option value={undefined} selected>All Time</option>
	</select>
</div>
