<script lang="ts">
	import '../app.css';
	import { onNavigate } from '$app/navigation';
	import Footer from '$lib/footer.svelte';
	import Nav from '$lib/nav.svelte';

	let { children } = $props();
	const loggedIn: boolean = document.cookie.includes('token_set=');
	console.debug('logged in?', document.cookie, loggedIn);
	let location = $state(encodeURI(document.location.pathname));
	onNavigate((navigation) => {
		console.log(navigation);
		location = encodeURI(navigation.to.url.pathname);
	});
</script>

<div class="grid h-screen grid-rows-[auto_1fr_auto]">
	<!-- Header -->
	<Nav {loggedIn} {location} ></Nav>
	<!-- Grid Columns -->
	<div class="grid grid-cols-1 md:grid-cols-[auto_1fr]">
		<!-- Left Sidebar. -->
		<aside class=""></aside>
		<!-- Main Content -->
		<main class="max-w-dvw space-y-4">
			{@render children()}
		</main>
	</div>
	<!-- Footer -->
	<Footer></Footer>
</div>
