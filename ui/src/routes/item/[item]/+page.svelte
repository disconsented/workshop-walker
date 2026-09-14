<script lang="ts">
	import Icon from 'svelte-awesome';
	import { faTriangleExclamation } from '@fortawesome/free-solid-svg-icons';
	import { Shadow } from 'svelte-loading-spinners';
	import type { PageData } from '*.svelte';
	import Sidebar from './Sidebar.svelte';
	import Body from './Body.svelte';

	let { data }: { data: PageData } = $props();
	console.log(data);

	const loggedIn = document.cookie.includes('token_set=');
</script>

<svelte:head>
	{#await data then data}
		<title
			>{data.data.title
				? 'Workshop Walker - ' + data.data.title
				: 'Workshop Walker - Loading'}</title
		>
		<meta property="og:title" content={'Workshop Walker - ' + data.title} />
		<meta property="og:type" content="website" />
		<meta property="og:url" content={window.location.href} />
		<meta property="og:image" content={data.preview_url} />
	{/await}
</svelte:head>

{#await data.data}
	<div class="flex h-full w-full place-content-center">
		<Shadow></Shadow>
	</div>
{:then item}
	{#if item.status}
		{@render errorCard(item)}
	{:else}
		<div class="grid min-h-screen grid-cols-1 gap-4 pt-4 md:grid-cols-[auto_1fr]">
			<div class="w-lg p-4">
				<Sidebar {loggedIn} {item} />
			</div>
			<div class="w-full max-w-6xl">
				<Body {item} />
			</div>
		</div>
	{/if}
{/await}

{#snippet errorCard(value)}
	<div
		class="card preset-outlined-error-500 grid grid-cols-1 items-center gap-4 p-4 lg:grid-cols-[auto_1fr_auto]"
	>
		<Icon data={faTriangleExclamation} class="fa-fw"></Icon>
		<div>
			{#if value.status}
				<p class="font-bold">Error Code: {value.status}</p>
			{/if}
			{#if value.statusText}
				<p class="text-xs opacity-60">{value.statusText}</p>
			{/if}

			{#if value.body}
				<pre class="text-xs opacity-60">{value.body}</pre>
			{/if}

			{#if value.message}
				<p class="text-xs opacity-60">{value.message}</p>
			{/if}
		</div>
	</div>
{/snippet}
