<script>
	import Icon from 'svelte-awesome';
	import { faGithub } from '@fortawesome/free-brands-svg-icons';
	import Logotype from '$lib/logotype.svelte';
</script>

<footer class="border-t-surface-200-800 bg-surface-100-900 border-t-1 p-6">
	<div class="flex flex-col">
		<div class="grid w-full grid-cols-[1fr_auto] pb-4">
			<div class="flex flex-col gap-2">
				<Logotype></Logotype>
				<span>A better browser for Steam Workshop items.</span>
				<span>Open source, community-driven.</span>
			</div>
			<div class="flex flex-col">
				<div class="text-surface-800-200 pb-2 font-bold">Project</div>
				<a href="/" class="anchor">Browse Games</a>
				<a
					href="https://github.com/disconsented/workshop-walker"
					class="anchor flex place-items-center gap-1"
				>
					<Icon data={faGithub} class="fa-fw"></Icon>
					GitHub
				</a>
			</div>
		</div>
		<div class="border-t-surface-200-800 border-t-1 pt-2">
			<div class="text-surface-300-700 grid w-full grid-cols-[1fr_auto]">
				<span>Developed by Disconsented</span>
				<span>Not affiliated with Valve Corporation</span>
			</div>
		</div>
	</div>
</footer>
