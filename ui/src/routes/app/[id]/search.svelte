<script lang="ts">
	import { SegmentedControl, ToggleGroup } from '@skeletonlabs/skeleton-svelte';
	import Icon from 'svelte-awesome';
	import {
		faArrowDownWideShort,
		faCalendar,
		faLanguage,
		faRightFromBracket,
		faRightToBracket,
		faSliders
	} from '@fortawesome/free-solid-svg-icons';

	interface Props {
		tags: string[];
	}

	let { tags }: Props = $props();
	let value = $state<string | null>('and');

	const items = [{ value: 'rimworld', label: 'Rimworld' }];
	const languages = [
		{ value: 'english', label: 'English' },
		{ value: 'french', label: 'French' }
	];
	const onOpenChange = (): void => {};
	const onInputValueChange = (): void => {};

	let showAdvanced = $state(true);
</script>

<form class="card preset-filled-surface-100-900 border-surface-200-800 flex flex-col border-1">
	<div
		class="border-surface-200-800 grid grid-cols-[auto_1fr_auto_auto_auto_auto] items-center justify-between gap-3 border-b-1 p-4"
	>
		<select class="select">
			<option value="rimworld">Rimworld</option>
		</select>
		<input class="input" type="text" placeholder="Search by Title" />

		<div class="field-group grid-cols-[auto_1fr] gap-0">
			<label class="label label-text preset-tonal" for="url">
				<Icon data={faLanguage} class="fa-fw" />
			</label>
			<select class="select rounded-r-lg">
				<option value="1">English</option>
				<option value="2">Русский</option>
				<!--Russian-->
				<option value="3">國語</option>
				<!--Chinese-->
				<option value="4">日本語</option>
				<!--Japanese-->
				<option value="5">한국어</option>
				<!--Korean-->
				<option value="6">Español</option>
				<!--Spanish-->
				<option value="7">Português</option>
				<!--Portuguese-->
			</select>
		</div>

		<div class="field-group grid-cols-[auto_1fr] gap-0">
			<label class="label label-text preset-tonal" for="url">
				<Icon data={faArrowDownWideShort} class="fa-fw" />
			</label>
			<select class="select rounded-r-lg">
				<option value="update">Last Updated</option>
				<option value="popularWeek" disabled>Popular - One Week</option>
				<option value="popularQuarter" disabled>Popular - Three Months</option>
				<option value="popularQuarter" disabled>Popular - All Time</option>
			</select>
		</div>
		<button class="btn preset-tonal" onclick={() => (showAdvanced = !showAdvanced)}
			><Icon data={faSliders} class="fa-fw" /> Advanced</button
		>
		<button class="btn preset-filled">Search</button>
	</div>
	<div class="bg-surface-50-950/50 grid grid-cols-2 gap-4 p-4" class:hidden={!showAdvanced}>
		<div class="flex w-full flex-col">
			<div class="flex flex-col">
				Updated
				<div class="flex flex-row gap-2">
					<div class="field-group w-full grid-cols-[auto_1fr] gap-0">
						<label class="label label-text preset-tonal" for="url">
							<Icon data={faCalendar} class="fa-fw" />
						</label>
						<select class="select rounded-r-lg">
							<option value="all" selected>All time</option>
							<option value="today">Today</option>
							<option value="week">Past week</option>
							<option value="month">Past month</option>
							<option value="quarter">Past 3 months</option>
							<option value="half">Past 6 months</option>
							<option value="year">Past year</option>
							<option value="custom">Custom range...</option>
						</select>
					</div>

					<div class="field-group grid-cols-[auto_1fr] gap-0">
						<label class="label label-text preset-tonal" for="url">
							<Icon data={faRightToBracket} class="fa-fw" />
						</label>
						<input class="input" type="date" />
					</div>

					<div class="field-group grid-cols-[auto_1fr] gap-0">
						<label class="label label-text preset-tonal" for="url">
							<Icon data={faRightFromBracket} class="fa-fw" />
						</label>
						<input class="input" type="date" />
					</div>
				</div>
			</div>
			<div class="flex flex-col">
				<div class="flex flex-row items-center justify-between">
					Tags
					<SegmentedControl {value} onValueChange={(details) => (value = details.value)} disabled>
						<SegmentedControl.Control class="gap-0 p-0">
							<SegmentedControl.Indicator />
							<SegmentedControl.Item value="and">
								<SegmentedControl.ItemText>AND</SegmentedControl.ItemText>
								<SegmentedControl.ItemHiddenInput />
							</SegmentedControl.Item>
							<SegmentedControl.Item value="or">
								<SegmentedControl.ItemText>OR</SegmentedControl.ItemText>
								<SegmentedControl.ItemHiddenInput />
							</SegmentedControl.Item>
						</SegmentedControl.Control>
					</SegmentedControl>
				</div>
				<div class="flex flex-row flex-wrap gap-1">
					{#each tags as tag}
						<ToggleGroup>
							<ToggleGroup.Item
								value={tag}
								class="chip preset-outlined-surface-400-600 hover:preset-tonal data-[state=on]:preset-filled-primary-500"
								>{tag}</ToggleGroup.Item
							>
						</ToggleGroup>
					{/each}
				</div>
			</div>
		</div>
		<div class="flex w-full flex-col">
			<div class="flex flex-col">
				<div class="flex flex-row items-center justify-between">
					Properties
					<SegmentedControl {value} onValueChange={(details) => (value = details.value)} disabled>
						<SegmentedControl.Control class="gap-0 p-0">
							<SegmentedControl.Indicator />
							<SegmentedControl.Item value="and">
								<SegmentedControl.ItemText>AND</SegmentedControl.ItemText>
								<SegmentedControl.ItemHiddenInput />
							</SegmentedControl.Item>
							<SegmentedControl.Item value="or">
								<SegmentedControl.ItemText>OR</SegmentedControl.ItemText>
								<SegmentedControl.ItemHiddenInput />
							</SegmentedControl.Item>
						</SegmentedControl.Control>
					</SegmentedControl>
				</div>
				<div class="field-group w-2xs grid-cols-[1fr_auto] gap-0">
					<input class="input grow-0" type="text" placeholder="Add property" disabled />
					<button class="btn preset-filled" disabled>Add</button>
				</div>
			</div>
		</div>
	</div>
</form>
