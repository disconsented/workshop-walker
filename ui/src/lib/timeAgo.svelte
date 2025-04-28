<script lang="ts">
	import TimeAgo from 'javascript-time-ago';
	import en from 'javascript-time-ago/locale/en';
	TimeAgo.addDefaultLocale(en);
	let { date }: { date: number } = $props();
	const timeAgo = new TimeAgo('en-US');
</script>

{timeAgo.format(new Date(date * 1000))}
