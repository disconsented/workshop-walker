<script lang="ts">
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
	const link = url ? url : appid === '#' ? '#' : '/app/' + appid;
</script>

<a
	href={link}
	class:card-hover={appid !== '#'}
	class="card preset-filled-surface-100-900 border-surface-200-800 divide-surface-200-800 block w-md max-w-md divide-y overflow-hidden border-[1px]"
>
	<header class="flex">
		<img
			src={image_url}
			class="aspect-[21/9] w-full object-fill grayscale hue-rotate-90"
			alt="{name} Banner"
			loading="lazy"
		/>
	</header>
	<article class="space-y-4 p-4">
		<div>
			<h2 class="h6">{name}</h2>
		</div>
		<p class="opacity-60">
			{@html description}
		</p>
	</article>
	<footer class="flex items-center justify-between gap-4 p-4">
		<small class="opacity-60">Developed By {developer}</small>
	</footer>
</a>
