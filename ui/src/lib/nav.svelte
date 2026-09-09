<script lang="ts">
	import { AppBar } from '@skeletonlabs/skeleton-svelte';
	import Icon from 'svelte-awesome';
	import { faGithub, faSteam } from '@fortawesome/free-brands-svg-icons';
	import Logotype from './logotype.svelte';
	import type { Breadcrumbs, Segment } from './breadcrumbs';

	interface Props {
		loggedIn: boolean;
		location: string;
		segments?: Breadcrumbs;
	}

	let { loggedIn = $bindable(), location, segments = [] }: Props = $props();
</script>

{#snippet trail(segments: Segment[])}
	{#each segments as segment, index (segment.href)}
		<li class="opacity-50" aria-hidden="true">/</li>
		<li>
			{#if index === segments.length - 1}
				<a href={segment.href} aria-current="page" class="hover:underline">{segment.title}</a>
			{:else}
				<a class="opacity-60 hover:underline" href={segment.href}>{segment.title}</a>
			{/if}
		</li>
	{/each}
{/snippet}

<header class="">
	<AppBar>
		<AppBar.Toolbar class="grid-cols-[1fr_1fr]">
			<AppBar.Lead>
				<ol class="flex items-center gap-4" aria-label="Breadcrumb">
					<li class="flex">
						<a href="/" aria-label="Workshop Walker"><Logotype></Logotype></a>
					</li>
					{#if Array.isArray(segments)}
						{@render trail(segments)}
					{:else}
						{#await segments then resolved}
							{@render trail(resolved)}
						{/await}
					{/if}
				</ol>
			</AppBar.Lead>
			<AppBar.Trail class="justify-end">
				<a
					href="https://github.com/disconsented/workshop-walker"
					class="btn preset-outlined-primary-100-900"
				>
					<Icon data={faGithub} class="fa-fw"></Icon>
				</a>
				{#if loggedIn}
					<a
						href="/api/logout?location={location}"
						aria-label="Sign Out"
						class="btn preset-outlined-primary-100-900"
					>
						Sign Out
					</a>
				{:else}
					<a
						href="/api/login?location={location}"
						aria-label="Sign In Through Steam"
						class="btn preset-outlined-primary-100-900"
					>
						<Icon data={faSteam} class="fa-fw" />
						Sign in with Steam
					</a>
				{/if}
			</AppBar.Trail>
		</AppBar.Toolbar>
	</AppBar>
</header>
