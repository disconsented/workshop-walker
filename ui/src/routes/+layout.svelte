<script lang="ts">
	import '../app.css';
	import { AppBar } from '@skeletonlabs/skeleton-svelte';
	import { faArrowLeft, faHome } from '@fortawesome/free-solid-svg-icons';
	import Icon from 'svelte-awesome';
	import { faGithub } from '@fortawesome/free-brands-svg-icons';

	let { children } = $props();
	let value = $state('files');
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
		<main class="space-y-4 p-4">
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
