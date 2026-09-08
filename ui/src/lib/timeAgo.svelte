<script lang="ts">
	import { timeAgo } from '$lib/timeAgo';

	let { date, short = false }: { date: number; short?: boolean } = $props();
</script>

{#if short}
	{timeAgo.format(new Date(date * 1000), 'mini')}
{:else}{timeAgo.format(new Date(date * 1000))}
{/if}
