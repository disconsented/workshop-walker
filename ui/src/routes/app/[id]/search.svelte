<script lang="ts">
	import { Combobox, Portal, SegmentedControl } from '@skeletonlabs/skeleton-svelte';
	import Icon from 'svelte-awesome';
	import { faLanguage } from '@fortawesome/free-solid-svg-icons';

	let value = $state<string | null>('and');

	const items = [{ value: 'rimworld', label: 'Rimworld' }];
	const languages =  [{ value: 'english', label: 'English' }, { value: 'french', label: 'French' }];
	const onOpenChange = (): void => {}
	const onInputValueChange = (): void => {}
</script>

<form class="card preset-filled-surface-100-900 border-surface-200-800 flex flex-col border-1">
	<div
		class="border-surface-200-800 items-center justify-between border-b-1 p-4 grid grid-cols-[auto_1fr_auto_auto_auto_auto] gap-3">
		<select class="select">
			<option value="rimworld">Rimworld</option>
		</select>
		<input class="input" type="text" placeholder="Search by Title">

		<div class="grid field-group grid-cols-[auto_1fr] items-center">
			<label class="label label-text h-full w-full items-center flex px-1 rounded-l-lg">
				<Icon data={faLanguage} class="fa-fw"/>
			</label>
			<select class="select rounded-r-lg">
				<option value="english">English</option>
				<option value="english">English</option>
				<option value="english">English</option>
			</select>
		</div>


		<select class="select">
			<option value="english">Popular</option>
		</select>
		<button class="btn preset-filled">Advanced</button>
		<button class="btn preset-filled">Search</button>
	</div>
	<div class="grid grid-cols-2 p-4 gap-2">
		<div class="flex w-full flex-col">
			<div class="flex flex-col">
				Updated
				<div class="flex flex-row">
					<select class="select"></select><input class="input" type="date" /><input
					class="input"
					type="date"
				/>
				</div>
			</div>
			<div class="flex flex-col">
				<div class="flex flex-row items-center justify-between">
					Tags
					<SegmentedControl {value} onValueChange={(details) => (value = details.value)} disabled>
						<SegmentedControl.Control>
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
				<div class="flex flex-row">
					<button type="button" class="chip preset-outlined-surface-400-600 hover:preset-tonal"
					>Chip
					</button
					>
					<button type="button" class="chip preset-outlined-surface-400-600 hover:preset-tonal"
					>Chip
					</button
					>
					<button type="button" class="chip preset-outlined-surface-400-600 hover:preset-tonal"
					>Chip
					</button
					>
					<button type="button" class="chip preset-outlined-surface-400-600 hover:preset-tonal"
					>Chip
					</button
					>
					<button type="button" class="chip preset-outlined-surface-400-600 hover:preset-tonal"
					>Chip
					</button
					>
					<button type="button" class="chip preset-outlined-surface-400-600 hover:preset-tonal"
					>Chip
					</button
					>
					<button type="button" class="chip preset-outlined-surface-400-600 hover:preset-tonal"
					>Chip
					</button
					>
				</div>
			</div>
		</div>
		<div class="flex w-full flex-col">
			<div class="flex flex-col">
				<div class="flex flex-row items-center justify-between">
					Properties
					<SegmentedControl {value} onValueChange={(details) => (value = details.value)} disabled>
						<SegmentedControl.Control>
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
				<div class="flex flex-row">
					<input class="input  grow-0" type="text" placeholder="Add property"> <button class="btn preset-filled">Add</button>
				</div>
			</div>
		</div>
	</div>
</form>
