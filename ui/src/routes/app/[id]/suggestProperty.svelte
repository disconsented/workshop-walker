<script lang="ts">
	import { SegmentedControl } from '@skeletonlabs/skeleton-svelte';
	import { faQuoteLeft } from '@fortawesome/free-solid-svg-icons';
	import Icon from 'svelte-awesome';

	const classes = ['type', 'feature', 'theme', 'genre'];
	let open = $state(false);
	let prop_class = $state('type');


	const accentColour = $derived.by(() => {
		console.log(prop_class);
		switch (prop_class.toLowerCase()) {
			case 'genre':
				return 'green';
			case 'theme':
				return 'blue';
			case 'type':
				return 'purple';
			case 'feature':
				return 'orange';
		}
	});
</script>

<!-- Silly little hack to keep these classes from being removed by the compiler -->
<!--<div class="bg-(--color-green-100) bg-(--color-blue-100) bg-(--color-purple-100) bg-(--color-orange-100)"></div>-->
<!--<div class="text-(--color-green-500) text-(--color-blue-500) text-(--color-purple-500) text-(--color-orange-500)"></div>-->
<!--<div class="border-(--color-green-300) border-(--color-blue-300) border-(--color-purple-300) border-(--color-orange-300)"></div>-->

{#if !open}
	<button
		class="btn border-1 border-dashed border-gray-300 text-xs text-gray-300 opacity-80 rounded-md text-center p-1 w-full"
		onclick={() => open = true}>
		+ Suggest a property
	</button>
{:else}
	<form class="flex flex-row flex-wrap w-full border-1 rounded-md border-primary-50-950">
		<div class="flex w-full grow">
			<SegmentedControl value={prop_class} onValueChange={(details) => (prop_class = details.value)}
			                  defaultValue="type" class="w-full" required>
				<SegmentedControl.Control>
					<SegmentedControl.Indicator class="bg-(--color-{accentColour}-100) border-1 border-(--color-{accentColour}-300) opacity-80" />
					{#each classes as prop_class}
						<SegmentedControl.Item value={prop_class}>
							<SegmentedControl.ItemText class={["uppercase text-xs"]}>{prop_class}</SegmentedControl.ItemText>
							<SegmentedControl.ItemHiddenInput />
						</SegmentedControl.Item>
					{/each}
				</SegmentedControl.Control>
			</SegmentedControl>
		</div>
		<div class="flex flow-row flex-wrap p-2 gap-2">
			<div class="flex w-full">
				<div class="input-group grid-cols-[auto_1fr_auto] w-full bg-surface-50-950">
					<div class="ig-cell uppercase text-xs text-(--color-{accentColour}-500)">{prop_class}</div>
					<input class="ig-input" type="text" required />
				</div>
			</div>
			<div class="flex w-full">
				<div class="input-group grid-cols-[auto_1fr_auto] w-full bg-surface-50-950">
					<div class="ig-cell uppercase text-xs">
						<Icon data={faQuoteLeft} class="text-xs text-gray-600 fa-fw" scale={0.8} />
					</div>
					<input class="ig-input bg-surface-50-950" type="text" placeholder="Reasoning - optional" />
				</div>
			</div>
			<footer class="flex w-full justify-between">
				<button class="btn preset-tonal-surface" type="reset" onclick={() => open = false}>Cancel</button>
				<button class="btn preset-filled-primary-300">Suggest</button>
			</footer>
		</div>

	</form>
{/if}

