<script module lang="ts">
	const data = [
		{
			date: new Date('2012-04-17T07:00:00.000Z'),
			value: 609.7
		},
		{
			date: new Date('2012-04-18T07:00:00.000Z'),
			value: 608.34
		},
		{
			date: new Date('2012-04-19T07:00:00.000Z'),
			value: 587.44
		},
		{
			date: new Date('2012-04-20T07:00:00.000Z'),
			value: 572.98
		},
		{
			date: new Date('2012-04-23T07:00:00.000Z'),
			value: 571.7
		},
		{
			date: new Date('2012-04-24T07:00:00.000Z'),
			value: 560.28
		},
		{
			date: new Date('2012-04-25T07:00:00.000Z'),
			value: 610
		},
		{
			date: new Date('2012-04-26T07:00:00.000Z'),
			value: 607.7
		},
		{
			date: new Date('2012-04-27T07:00:00.000Z'),
			value: 603
		},
		{
			date: new Date('2012-04-30T07:00:00.000Z'),
			value: 583.98
		},
		{
			date: new Date('2012-05-01T07:00:00.000Z'),
			value: 582.13
		}
	];
	import {
		Area,
		Axis,
		Chart,
		Highlight,
		Layer,
		LinearGradient,
		RectClipPath,
		Tooltip
	} from 'layerchart';
	import { format } from '@layerstack/utils';
</script>

<Chart
	{data}
	x="date"
	y="value"
	yDomain={[0, null]}
	yNice
	padding={{ top: 20, bottom: 20 }}
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
			<Axis placement="bottom" />
		</Layer>

		<Tooltip.Root
			y={24}
			xOffset={4}
			variant="none"
			class="text-primary text-sm leading-3 font-semibold"
		>
			{#snippet children({ data })}
				{format(data.value, 'currency')}
			{/snippet}
		</Tooltip.Root>

		<Tooltip.Root x={4} y={4} variant="none" class="text-sm leading-3 font-semibold">
			{#snippet children({ data })}
				{format(data.date, 'day')}
			{/snippet}
		</Tooltip.Root>

		<Tooltip.Root
			x="data"
			y={context.height + context.padding.top + 2}
			anchor="top"
			variant="none"
			class="bg-primary text-primary-content rounded-sm px-2 py-1 text-sm leading-3 font-semibold whitespace-nowrap"
		>
			{#snippet children({ data })}
				{format(data.date, 'day')}
			{/snippet}
		</Tooltip.Root>
	{/snippet}
</Chart>
