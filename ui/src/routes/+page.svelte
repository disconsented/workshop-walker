<script lang="ts">
	import AppCard from '$lib/app_card.svelte';

	let { data } = $props();
	console.log(data);
</script>

<svelte:head>
	<title>Workshop Walker - Steam Items</title>
	<meta property="og:title" content="Workshop Walker" />
	<meta property="og:type" content="website" />
	<meta property="og:url" content={window.location.href} />
</svelte:head>

<div class="flex flex-wrap gap-4">
	{#each data.items as app}
		<AppCard
			appid={app.id}
			image_url={app.banner}
			description={app.description}
			developer={app.developer}
			name={app.name}
			url={undefined}
		></AppCard>
	{/each}
	<AppCard
		appid="#"
		image_url="https://community.cloudflare.steamstatic.com/public/shared/images/header/logo_steam.svg?t=962016"
		description="We accept suggestions for new games on our <a class='anchor' href='https://github.com/disconsented/workshop-walker/discussions'>GitHub discussion page</a>!"
		developer="Disconsented"
		name="Suggest a new game for Workshop Walker?"
		url="https://github.com/disconsented/workshop-walker"
	></AppCard>
</div>
