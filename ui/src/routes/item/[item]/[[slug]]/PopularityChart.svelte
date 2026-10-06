<script lang="ts">
	import { Area, Chart, Highlight, Layer, LinearGradient, RectClipPath, Tooltip } from 'layerchart';
	import { format } from '@layerstack/utils';

	interface Props {
		data: number[];
	}

	let { data }: Props = $props();

	// LayerChart accessors take the datum alone, never an index, so a bare
	// number[] has no x channel. Pair each value with its position first.
	let series = $derived(data.map((value, index) => ({ index, value })));
</script>

<Chart
	data={series}
	x="index"
	y="value"
	yDomain={[0, null]}
	yNice
	padding={{ top: 20, bottom: 4 }}
	tooltipContext={{ mode: 'quadtree-x' }}
	height={150}
>
	{#snippet children({ context })}
		<Layer>
			<LinearGradient class="from-primary-500/50 to-primary-500/1" vertical>
				{#snippet children({ gradient })}
					<Area line={{ class: 'stroke-2 dark:stroke-primary-500 opacity-20' }} fill={gradient} />
					<RectClipPath
						x={0}
						y={0}
						width={context.tooltip.data ? context.tooltip.x : context.width}
						height={context.height}
						motion="spring"
					>
						<Area line={{ class: 'stroke-2 dark:stroke-primary-500' }} fill={gradient} />
					</RectClipPath>
				{/snippet}
			</LinearGradient>
			<Highlight points lines={{ class: 'stroke-primary [dark:stroke-dasharray:unset]' }} />
		</Layer>

		<Tooltip.Root
			y={24}
			xOffset={4}
			variant="none"
			class="text-primary text-sm leading-3 font-semibold"
		>
			{#snippet children({ data })}
				{format(data.value, 'integer')}
			{/snippet}
		</Tooltip.Root>
	{/snippet}
</Chart>
