<script lang="ts">
	import { timeAgo } from '$lib/timeAgo';

	let { date }: { date: number } = $props();
</script>

{timeAgo.format(new Date(date * 1000))}
