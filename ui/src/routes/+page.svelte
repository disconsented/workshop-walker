<script lang="ts">
	import AppCard from '$lib/app_card.svelte';
</script>

<svelte:head>
	<title>Workshop Walker - Steam Items</title>
</svelte:head>

<div class="flex gap-4">
	<AppCard
		appid="294100"
		image_url="https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/294100/header.jpg?t=1734154189"
		description="A sci-fi colony sim driven by an intelligent AI storyteller. Generates stories by simulating psychology, ecology, gunplay, melee combat, climate, biomes, diplomacy, interpersonal relationships, art, medicine, trade, and more."
		developer="Ludeon Studios"
		name="Rimworld"
	></AppCard>
	<AppCard
		appid="#"
		image_url="https://community.cloudflare.steamstatic.com/public/shared/images/header/logo_steam.svg?t=962016"
		description="We accept suggestions for new games on our <a class='anchor' href='https://github.com/disconsented/workshop-walker/discussions'>GitHub discussion page</a>!"
		developer="Disconsented"
		name="Suggest a new game for Workshop Walker?"
		url="https://github.com/disconsented/workshop-walker"
	></AppCard>
</div>
