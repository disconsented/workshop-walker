<script lang="ts">
	import { Accordion, Portal, Tabs, ToggleGroup, Tooltip } from '@skeletonlabs/skeleton-svelte';
	import { faChevronDown, faChevronUp, faFilter } from '@fortawesome/free-solid-svg-icons';
	import Icon from 'svelte-awesome';
	import { whichLang } from '$lib/lang';
	import BodyTab from './BodyTab.svelte';

	interface Props {
		item: any;
	}

	let { item }: Props = $props();
	let open = $state(false);
	let accordionFilter = $state(['open']);

	let selectedTags = $state([]);
	let selectedLangs = $state(['English']);

	function mapTags(item) {
		return item.tags.map((tag) => {
			return [tag.id, tag.display_name];
		});
	}

	function mapLanguages(item) {
		return item.languages;
	}

	const all_tagz = [
		...new Map(
			item.dependants.map(mapTags).flat().concat(item.dependencies.map(mapTags).flat())
		).entries()
	];
	const all_tags = all_tagz.map((tag) => {
		return { id: tag[0], display_name: tag[1] };
	});
	const all_langs = [
		...new Set(
			item.dependants.map(mapLanguages).flat().concat(item.dependencies.map(mapLanguages).flat())
		)
	];
	all_langs.sort();
	$inspect(all_langs, all_tags);
	let tab = $state(undefined);
</script>

<div class="flex flex-col gap-4">
	<div class="flex flex-col">
		<div>
			<button
				class="btn preset-outlined-surface-200-800"
				class:invisible={!open}
				onclick={() => (open = false)}
				>Show Less
			</button>
			<div
				class="absolute h-[6lh] w-full bg-linear-to-t from-black to-[transparent]"
				class:hidden={open}
			></div>
			<div class="overflow-hidden" class:max-h-[6lh]={!open}>
				{@html item.description}
			</div>
		</div>
		<button class="btn preset-outlined-surface-200-800 w-fit" onclick={() => (open = !open)}>
			Show {open ? 'Less' : 'More'}</button
		>
	</div>

	<hr class="hr border-b-surface-200-800" />

	<div>
		<Tabs defaultValue="dependants" value={tab}>
			<Tabs.List>
				<Tabs.Trigger value="dependencies"
					>Dependencies <sup class="text-primary-500">{item.dependencies.length}</sup>
				</Tabs.Trigger>
				<Tabs.Trigger value="dependants"
					>Required By <sup class="text-primary-500">{item.dependants.length}</sup>
				</Tabs.Trigger>
				<Tabs.Indicator />
			</Tabs.List>
			<div>
				<Accordion
					value={accordionFilter}
					onValueChange={(e) => (accordionFilter = e.value)}
					collapsible
				>
					<Accordion.Item value="open">
						<Accordion.ItemTrigger>
							<Icon data={faFilter} class="fa-fw" />
							Filter
							{#if accordionFilter[0] == 'open'}
								<Icon data={faChevronUp} class="fa-fw" />
							{:else}
								<Icon data={faChevronDown} class="fa-fw" />
							{/if}
						</Accordion.ItemTrigger>
						<Accordion.ItemContent>
							<form>
								<div class="grid grid-cols-2 gap-2">
									<div>
										<span class="label-text">Tags</span>
										<ToggleGroup
											class="flex flex-wrap border-0"
											value={selectedTags}
											onValueChange={(details) => (selectedTags = details.value)}
											multiple
										>
											{#each all_tags as tag}
												<ToggleGroup.Item
													value={tag.id}
													class="chip preset-outlined-surface-400-600 hover:preset-tonal data-[state=on]:preset-filled-primary-500"
												>
													<Tooltip positioning={{ placement: 'top' }}>
														<!--Fixes submitting the forum prematurely-->
														<Tooltip.Trigger>
															<button type="button">{tag.display_name}</button>
														</Tooltip.Trigger>
														<Portal>
															<Tooltip.Positioner>
																<Tooltip.Content class="card preset-filled-surface-950-50 p-2">
																	<span>"{tag.display_name}"</span>
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
									<div>
										<span class="label-text">Languages</span>
										<ToggleGroup
											class="flex flex-wrap border-0"
											value={selectedLangs}
											onValueChange={(details) => (selectedLangs = details.value)}
											multiple
										>
											{#each all_langs as lang}
												<ToggleGroup.Item
													value={whichLang(lang)}
													class="chip preset-outlined-surface-400-600 hover:preset-tonal data-[state=on]:preset-filled-primary-500"
												>
													{whichLang(lang)}
												</ToggleGroup.Item>
											{/each}
										</ToggleGroup>
									</div>
								</div>
							</form>
						</Accordion.ItemContent>
					</Accordion.Item>
				</Accordion>
			</div>
			<Tabs.Content value="dependencies">
				<span class="opacity-50"> Items that must be installed alongside this one. </span>
				<BodyTab {selectedTags} {selectedLangs} items={item.dependencies} />
			</Tabs.Content>
			<Tabs.Content value="dependants">
				<span class="opacity-50">Items that list this one as a dependency.</span>
				<BodyTab {selectedTags} {selectedLangs} items={item.dependants} />
			</Tabs.Content>
		</Tabs>
	</div>
</div>
