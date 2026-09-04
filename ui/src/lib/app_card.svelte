<script lang="ts">
	import Icon from 'svelte-awesome';
	import { faArrowRight } from '@fortawesome/free-solid-svg-icons';

	interface Props {
		image_url: string;
		name: string;
		description: string;
		developer: string;
		appid: string;
		url: string | undefined;
	}

	let { image_url, name, developer, description, appid, url = undefined }: Props = $props();
	// Direct URL, anchor or app page link
	const link = url ? url : appid === '#' ? '.' : '/app/' + appid;
</script>

<a
	href={link}
	class="card preset-filled-surface-100-900 border-surface-200-800 divide-surface-200-800 block w-md max-w-md divide-y overflow-hidden border-[1px]"
>
	<header class="flex">
		<img
			src={image_url}
			class="aspect-[21/9] w-full object-fill"
			alt="{name} Banner"
			loading="lazy"
		/>
	</header>
	<article class="flex flex-col space-y-4 p-4">
		<div class="grid grid-cols-[1fr_auto] items-center">
			<div>
				<h2 class="h6">{name}</h2>
				<small class="text-sm opacity-60">{developer}</small>
			</div>
			<div
				class="btn preset-filled-surface-100-900 border-surface-200-800 h-fit w-fit border-1 p-1 opacity-60"
			>
				<Icon data={faArrowRight} class="fa-fw"></Icon>
			</div>
		</div>

		<p class="text-sm opacity-60">
			{@html description}
		</p>
	</article>
</a>
