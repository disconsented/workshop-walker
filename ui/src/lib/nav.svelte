<script lang="ts">
	import { AppBar } from '@skeletonlabs/skeleton-svelte';
	import Icon from 'svelte-awesome';
	import { faGithub, faSteam } from '@fortawesome/free-brands-svg-icons';
	import Logotype from './logotype.svelte';

	export interface Segment {
		title: string;
		href: string;
	}

	interface Props {
		loggedIn: boolean;
		location: string;
		segments?: Segment[];
	}

	let { loggedIn = $bindable(), segments = undefined }: Props = $props();
</script>

<header class="">
	<AppBar>
		<AppBar.Toolbar class="grid-cols-[1fr_1fr]">
			<AppBar.Lead>
				<ol class="flex items-center gap-4">
					<li class="flex">
						<Logotype></Logotype>
					</li>
					{#each segments as segment}
						<li class="opacity-50" aria-hidden>/</li>
						<li><a class="opacity-60 hover:underline" href={segment.href}>{segment.title}</a></li>
					{/each}
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
						<Icon data={faSteam} class="fa-fw"></Icon>
						Sign in with Steam
					</a>
				{/if}
			</AppBar.Trail>
		</AppBar.Toolbar>
	</AppBar>
</header>
