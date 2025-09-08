<script lang="ts">
	import '../app.css';
	import { AppBar } from '@skeletonlabs/skeleton-svelte';
	import { faHome } from '@fortawesome/free-solid-svg-icons';
	import Icon from 'svelte-awesome';
	import { faGithub } from '@fortawesome/free-brands-svg-icons';
	import { onNavigate } from '$app/navigation';

	let { children } = $props();
	const logged_in = document.cookie.includes('token_set=');
	console.debug('logged in?', document.cookie, logged_in);
	let location = $state(encodeURI(document.location.pathname));
	onNavigate((navigation) => {
		console.log(navigation);
		location = encodeURI(navigation.to.url.pathname);
	});
</script>

<div class="grid h-screen grid-rows-[auto_1fr_auto]">
	<!-- Header -->
	<header class="p-4">
		<AppBar>
			{#snippet lead()}
				<a href="/">
					<Icon data={faHome} class="fa-fw"></Icon>
				</a>
			{/snippet}
			<span>Workshop Walker</span>
			{#snippet trail()}
				{#if logged_in}
					<a href="/api/logout?location={location}" class="btn preset-outlined-warning-500">
						Sign Out
					</a>
				{:else}
					<a href="/api/login?location={location}" aria-label="Sign In Through Steam">
						<img
							alt="Sign In Through Steam"
							src="https://steamcdn-a.akamaihd.net/steamcommunity/public/images/steamworks_docs/english/sits_small.png"
							loading="lazy"
						/>
					</a>
				{/if}

				<a href="https://github.com/disconsented/workshop-walker">
					<Icon data={faGithub} class="fa-fw"></Icon>
				</a>
			{/snippet}
		</AppBar>
	</header>
	<!-- Grid Columns -->
	<div class="grid grid-cols-1 md:grid-cols-[auto_1fr]">
		<!-- Left Sidebar. -->
		<aside class=""></aside>
		<!-- Main Content -->
		<main class="max-w-dvw space-y-4 p-4">
			{@render children()}
		</main>
	</div>
	<!-- Footer -->
	<footer class="p-4">
		<a href="https://github.com/disconsented/workshop-walker" class="anchor">Workshop Walker</a> by
		<a href="https://disconsented.com" class="anchor">Disconsented</a>
		- Made with love using Rust, SurrealDB, Svelte & Skeleton.dev
	</footer>
</div>
