<script lang="ts">
	import { Portal, SegmentedControl, ToggleGroup, Tooltip } from '@skeletonlabs/skeleton-svelte';
	import Icon from 'svelte-awesome';
	import {
		faArrowDownWideShort,
		faCalendar,
		faChevronDown,
		faChevronUp,
		faLanguage,
		faRightFromBracket,
		faRightToBracket,
		faSliders
	} from '@fortawesome/free-solid-svg-icons';
	import { language, orderBy, tags, title, updatedAfter, updatedBefore } from './store.svelte';
	import { SvelteURLSearchParams } from 'svelte/reactivity';
	import { goto } from '$app/navigation';

	interface Props {
		appTags: string[];
	}

	let { appTags }: Props = $props();
	let value = $state<string | null>('and');
	let showAdvanced = $state(true);
	let updatedDate = $state(updatedBefore.v || updatedAfter.v ? 'custom' : 'all');

	const params = new SvelteURLSearchParams();
	$effect(() => {
		params.set('language', language.v);
	});

	$effect(() => {
		params.set('order_by', orderBy.v);
	});

	$effect(() => {
		if (title.v) {
			params.set('title', title.v);
		} else {
			params.delete('title');
		}
	});

	$effect(() => {
		if (updatedBefore.v) {
			params.set('updated_before', Math.trunc(updatedBefore.v.getTime() / 1000).toString());
		} else {
			params.delete('updated_before');
		}
	});

	$effect(() => {
		if (updatedAfter.v) {
			params.set('updated_after', Math.trunc(updatedAfter.v.getTime() / 1000).toString());
		} else {
			params.delete('updated_after');
		}
	});

	$effect(() => {
		if (tags.v) {
			params.delete('tags');
			tags.v.forEach((v) => {
				params.append('tags', v);
			});
		} else {
			params.delete('tags');
		}
	});

	const loadParams = () => {
		goto(`?${params}`, {
			keepFocus: true,
			noScroll: true,
			replaceState: true
		});
	};

	function updatedSelectionChange(event: InputEvent) {
		if (event.target) {
			const date = new Date();
			switch (event.target.value) {
				case 'all':
					updatedAfter.v = undefined;
					updatedBefore.v = undefined;
					break;
				case 'today':
					date.setDate(date.getDate() - 1);
					updatedAfter.v = date;
					updatedBefore.v = undefined;
					break;
				case 'week':
					date.setDate(date.getDate() - 7);
					updatedAfter.v = date;
					updatedBefore.v = undefined;
					break;
				case 'month':
					date.setDate(date.getDate() - 30);
					updatedAfter.v = date;
					updatedBefore.v = undefined;
					break;
				case 'quarter':
					date.setDate(date.getDate() - 90);
					updatedAfter.v = date;
					updatedBefore.v = undefined;
					break;
				case 'half':
					date.setDate(date.getDate() - 180);
					updatedAfter.v = date;
					updatedBefore.v = undefined;
					break;
				case 'year':
					date.setDate(date.getDate() - 365);
					updatedAfter.v = date;
					updatedBefore.v = undefined;
					break;
			}
		}
	}

	function submitForum(event) {
		event.preventDefault();
		loadParams();
	}
</script>

<form
	class="card preset-filled-surface-100-900 border-surface-200-800 flex flex-col border-1"
	onsubmit={submitForum}
>
	<div
		class="border-surface-200-800 grid grid-cols-[1fr_auto_auto_auto_auto] items-center justify-between gap-3 border-b-1 p-4"
	>
		<!--		Funny little hack so pressing enter does a submit-->
		<!--		See https://stackoverflow.com/questions/27807853/html5-how-to-make-a-form-submit-after-pressing-enter-at-any-of-the-text-inputs-->
		<input type="submit" class="hidden" />
		<select class="select hidden">
			<!--			ToDo: Dynamically load-->
			<option value="rimworld">Rimworld</option>
		</select>
		<input class="input" type="text" placeholder="Search by Title" bind:value={title.v} />

		<div class="field-group grid-cols-[auto_1fr] gap-0">
			<label class="label label-text preset-tonal" for="url">
				<Icon data={faLanguage} class="fa-fw" />
			</label>
			<select class="select rounded-r-lg" bind:value={language.v}>
				<option value="1">English</option>
				<!--Russian-->
				<option value="2">Русский</option>
				<!--Chinese-->
				<option value="3">國語</option>
				<!--Japanese-->
				<option value="4">日本語</option>
				<!--Korean-->
				<option value="5">한국어</option>
				<!--Spanish-->
				<option value="6">Español</option>
				<!--Portuguese-->
				<option value="7">Português</option>
			</select>
		</div>

		<div class="field-group grid-cols-[auto_1fr] gap-0">
			<label class="label label-text preset-tonal" for="url">
				<Icon data={faArrowDownWideShort} class="fa-fw" />
			</label>
			<select class="select rounded-r-lg" bind:value={orderBy.v}>
				<option value="LastUpdated">Last Updated</option>
				<option value="PopularWeek" disabled>Popular - One Week</option>
				<option value="PopularQuarter" disabled>Popular - Three Months</option>
				<option value="PopularQuarter" disabled>Popular - All Time</option>
			</select>
		</div>
		<button class="btn preset-tonal" type="button" onclick={() => (showAdvanced = !showAdvanced)}>
			<Icon data={faSliders} class="fa-fw" />
			Advanced

			<Icon data={showAdvanced ? faChevronDown : faChevronUp} class="fa-fw" />
		</button>
		<button class="btn preset-filled" type="button" onclick={loadParams}>Search</button>
	</div>
	<div class="bg-surface-50-950/50 grid grid-cols-2 gap-4 p-4" class:hidden={!showAdvanced}>
		<div class="flex w-full flex-col gap-2">
			<div class="flex flex-col gap-2">
				Updated
				<div class="flex flex-row gap-2">
					<div class="field-group w-full grid-cols-[auto_1fr] gap-0">
						<label class="label label-text preset-tonal" for="url">
							<Icon data={faCalendar} class="fa-fw" />
						</label>
						<select
							class="select rounded-r-lg"
							bind:value={updatedDate}
							oninput={updatedSelectionChange}
						>
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
						<label class="label label-text preset-tonal gap-1" for="url">
							<Icon data={faRightToBracket} class="fa-fw" />
							After
						</label>
						<input
							class="input"
							type="date"
							disabled={updatedDate != 'custom'}
							bind:value={
								() => updatedAfter.v?.toISOString().slice(0, 10) ?? '',
								(v) => (updatedAfter.v = v ? new Date(v + 'T00:00:00Z') : undefined)
							}
						/>
					</div>

					<div class="field-group grid-cols-[auto_1fr] gap-0">
						<label class="label label-text preset-tonal gap-1" for="url">
							<Icon data={faRightFromBracket} class="fa-fw" />
							Before
						</label>
						<input
							class="input"
							type="date"
							disabled={updatedDate != 'custom'}
							bind:value={
								() => updatedBefore.v?.toISOString().slice(0, 10) ?? '',
								(v) => (updatedBefore.v = v ? new Date(v + 'T00:00:00Z') : undefined)
							}
						/>
					</div>
				</div>
			</div>
			<div class="flex flex-col gap-2">
				<div class="flex flex-row items-center justify-between">
					<div>
						Tags <span class="text-sm italic opacity-50"
							>(Steam tag, shown as written. Similar names can be different tags.)</span
						>
					</div>
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
					{#each appTags as tag}
						<ToggleGroup
							value={tags.v}
							onValueChange={(details) => (tags.v = details.value)}
							multiple
						>
							<ToggleGroup.Item
								value={tag}
								class="chip preset-outlined-surface-400-600 hover:preset-tonal data-[state=on]:preset-filled-primary-500"
							>
								<Tooltip positioning={{ placement: 'top' }}>
									<Tooltip.Trigger>{tag}</Tooltip.Trigger>
									<Portal>
										<Tooltip.Positioner>
											<Tooltip.Content class="card preset-filled-surface-950-50 p-2">
												<span>"{tag}"</span>
												<Tooltip.Arrow
													class="[--arrow-background:var(--color-surface-950-50)] [--arrow-size:--spacing(2)]"
												>
													<Tooltip.ArrowTip />
												</Tooltip.Arrow>
											</Tooltip.Content>
										</Tooltip.Positioner>
									</Portal>
								</Tooltip>
							</ToggleGroup.Item>
						</ToggleGroup>
					{/each}
				</div>
			</div>
		</div>
		<div class="flex w-full flex-col gap-2">
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
