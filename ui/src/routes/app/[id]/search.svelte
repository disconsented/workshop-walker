<script lang="ts">
	import {
		Combobox,
		Portal,
		SegmentedControl,
		ToggleGroup,
		Tooltip,
		useListCollection
	} from '@skeletonlabs/skeleton-svelte';
	import Icon from 'svelte-awesome';
	import {
		faArrowDownWideShort,
		faCalendar,
		faCancel,
		faChevronDown,
		faChevronUp,
		faClose,
		faLanguage,
		faRightFromBracket,
		faRightToBracket,
		faSliders
	} from '@fortawesome/free-solid-svg-icons';
	import {
		language,
		orderBy,
		searchProps,
		tags,
		title,
		updatedAfter,
		updatedBefore
	} from './store.svelte';
	import { SvelteURLSearchParams } from 'svelte/reactivity';
	import { goto } from '$app/navigation';
	import Property from '../../item/[item]/Property.svelte';

	interface Props {
		appTags: string[];
		appID: string;
	}

	let { appTags, appID }: Props = $props();
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

	$effect(() => {
		if (searchProps.v) {
			params.delete('positive_props');
			searchProps.v.forEach(({ positive }, v) => {
				if (positive) {
					params.append('positive_props', v);
				}
			});
		} else {
			params.delete('positive_props');
		}
	});

	$effect(() => {
		if (searchProps.v) {
			params.delete('negative_props');
			searchProps.v.forEach(({ positive }, v) => {
				if (!positive) {
					params.append('negative_props', v);
				}
			});
		} else {
			params.delete('negative_props');
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

	let foundProps = $state(null);
	let searchQuery: Promise<Response> = $state(null);
	let query = $state('');

	function searchSuggestProps(term: string) {
		searchQuery = fetch('/api/properties/search', {
			method: 'post',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ app: appID, search_term: term })
		})
			.then((response) => response.json())
			.then((data) => {
				foundProps = data;
			});
	}

	const collection = $derived(
		useListCollection({
			items: foundProps ?? [],
			itemToValue: (item) => `${item.class}:${item.value}`,
			itemToString: (item) => `${item.class}:${item.value}`
		})
	);
</script>

<form
	class="card preset-filled-surface-100-900 border-surface-200-800 flex flex-col border-1"
	onsubmit={submitForum}
>
	<div
		class="border-surface-200-800 flex flex-wrap items-center justify-between gap-3 border-b-1 p-4 lg:grid lg:grid-cols-[1fr_auto_auto_auto_auto]"
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
				<!--German-->
				<option value="8">Deutsch</option>
				<!--French-->
				<option value="9">Français</option>
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
	<div
		class="bg-surface-50-950/50 flex flex-wrap gap-4 p-4 lg:grid lg:grid-cols-2"
		class:!hidden={!showAdvanced}
	>
		<div class="flex w-full flex-col gap-2">
			<div class="flex flex-col gap-2">
				Updated
				<div class="flex flex-row flex-wrap gap-2 lg:flex-nowrap">
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
					<ToggleGroup
						class="flex flex-wrap border-0"
						value={tags.v}
						onValueChange={(details) => (tags.v = details.value)}
						multiple
					>
						{#each appTags as tag}
							<ToggleGroup.Item
								value={tag}
								class="chip preset-outlined-surface-400-600 hover:preset-tonal data-[state=on]:preset-filled-primary-500"
							>
								<Tooltip positioning={{ placement: 'top' }}>
									<!--Fixes submitting the forum prematurely-->
									<Tooltip.Trigger>
										<button type="button">{tag}</button>
									</Tooltip.Trigger>
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
						{/each}
					</ToggleGroup>
				</div>
			</div>
		</div>
		<div class="flex w-full flex-col gap-2">
			<div class="flex flex-col">
				<div class="flex flex-row items-center justify-between">
					Properties
					<SegmentedControl
						value="or"
						onValueChange={(details) => (value = details.value)}
						disabled
					>
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
					<Combobox
						onInputValueChange={(e) => {
							query = e.inputValue;
							searchSuggestProps(query);
						}}
						{collection}
						selectionBehavior="clear"
						onValueChange={(details) => {
							let prop = details.value[0].split(':');
							searchProps.v.set(details.value[0], {
								property: { class: prop[0], value: prop[1] },
								positive: true
							});
						}}
						value={undefined}
					>
						<Combobox.Control>
							<Combobox.Input />
							<Combobox.Trigger />
						</Combobox.Control>
						<Portal>
							<Combobox.Positioner>
								<Combobox.Content>
									<Combobox.ItemGroup>
										{#each foundProps as item (item.class + ':' + item.value)}
											<Combobox.Item {item}>
												<Combobox.ItemText>
													<div class="flex">
														<Property loggedIn={false} property={item} hideVote={true} />
													</div>
												</Combobox.ItemText>
												<Combobox.ItemIndicator />
											</Combobox.Item>
										{/each}
									</Combobox.ItemGroup>
								</Combobox.Content>
							</Combobox.Positioner>
						</Portal>
					</Combobox>
				</div>
			</div>
			<div class="flex flex-row gap-1">
				{#each searchProps.v as item (item[0])}
					{@const property = item[1].property}
					{@const key = item[0]}
					{@const positive = item[1].positive}
					<div
						class="grid cursor-pointer place-items-center gap-1 pl-1"
						class:grid-cols-[auto_1fr_auto]={!positive}
						class:grid-cols-[1fr_auto]={positive}
						class:preset-tonal-error={!positive}
						class:preset-outlined-surface-200-800={positive}
						class:preset-outlined-error-500={!positive}
					>
						{#if !positive}
							<Icon data={faCancel} class="fa-fw" />
						{/if}
						<button
							type="button"
							class="flex cursor-pointer"
							onclick={() => {
								searchProps.v.set(key, { property: property, positive: !positive });
							}}
						>
							<Property
								loggedIn={false}
								property={{
									class: property.class,
									value: property.value,
									...{ vote_state: 0, vote_count: 0, upvote_count: 0, status: 1 }
								}}
								hideVote={true}
								subtle={true}
							/>
						</button>
						<button
							class="btn cursor-pointer"
							class:preset-outlined-surface-200-800={positive}
							class:preset-tonal-error={!positive}
							type="button"
							onclick={() => {
								searchProps.v.delete(key);
							}}
						>
							<Icon data={faClose} class="fa-fw" />
						</button>
					</div>
				{/each}
			</div>
		</div>
	</div>
</form>
