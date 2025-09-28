<!-- VisualQuery.svelte -->
<script lang="ts">
	// Types
	type Condition = {
		id: number;
		field: string;
		operator: string;
		value: string;
		negate: boolean;
	};

	type ConditionGroup = {
		id: number;
		type: 'AND' | 'OR';
		conditions: (Condition | ConditionGroup)[];
		negate: boolean;
	};

	// Example data
	const operators = {
		default: ['=', '!=', 'contains'],
		score: ['=', '!=', '>', '<', 'between'],
		date: ['before', 'after', 'on']
	};

	let query: ConditionGroup = {
		id: 1,
		type: 'AND',
		negate: false,
		conditions: [
			{
				id: 2,
				field: 'tag',
				operator: '=',
				value: 'mod',
				negate: false
			},
			{
				id: 3,
				type: 'OR',
				negate: false,
				conditions: [
					{
						id: 4,
						field: 'genre',
						operator: '=',
						value: 'Science Fiction',
						negate: false
					},
					{
						id: 5,
						field: 'theme',
						operator: '=',
						value: 'Cyberpunk',
						negate: false
					}
				]
			},
			{
				id: 6,
				field: 'language',
				operator: '!=',
				value: 'Japanese',
				negate: false
			}
		]
	};

	// UI helpers
	const getOperators = (field: string) => {
		if (field === 'score') return operators.score;
		if (field === 'last_updated') return operators.date;
		return operators.default;
	};
</script>

<div class="space-y-4 p-4  rounded-lg">
	<div class="flex items-center gap-2">
		<select bind:value={query.type} class="px-3 py-1 border rounded select">
			<option value="AND">AND</option>
			<option value="OR">OR</option>
		</select>
		<label class="flex items-center gap-1 label">
			<input type="checkbox" bind:checked={query.negate} class="checkbox"/>
			<span>NOT</span>
		</label>
	</div>

	<div class="pl-6 space-y-3 border-l-2 ">
		{#each query.conditions as condition (condition.id)}
			{#if 'field' in condition}
				<!-- Condition Row -->
				<div class="input-group grid-cols-[auto_auto_auto_1fr_auto]">
					<label class="ig-cell gap-1">
						<input type="checkbox" bind:checked={condition.negate} class="checkbox" />
						<span>NOT</span>
					</label>

					<select bind:value={condition.field} class="px-2 py-1 border rounded ig-select">
						<option value="tag">Tag</option>
						<option value="genre">Genre</option>
						<option value="theme">Theme</option>
						<option value="language">Language</option>
						<option value="score">Score</option>
					</select>

					<select bind:value={condition.operator} class="px-2 py-1 border rounded ig-select">
						{#each getOperators(condition.field) as op}
							<option value={op}>{op}</option>
						{/each}
					</select>

					<input bind:value={condition.value}
								 class="px-2 py-1 border rounded ig-input"
								 placeholder="Value" />

					<button class="btn preset-tonal-error px-2">×</button>
				</div>
			{:else}
				<!-- Nested Group -->
				<div class="p-3 rounded border border-blue-200">
					NESTED
				</div>
			{/if}
		{/each}

		<div class="flex gap-2">
			<button class="btn px-3 py-1 bg-blue-500 text-white rounded">
				+ Condition
			</button>
			<button class="btn px-3 py-1 bg-purple-500 text-white rounded">
				+ Group
			</button>
		</div>
	</div>
</div>

<!-- Recursive component definition -->
<script lang="ts" context="module">
	export let query: ConditionGroup;
</script>