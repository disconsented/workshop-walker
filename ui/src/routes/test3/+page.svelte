<script lang="ts">
	import CommandSearch from './CommandSearch.svelte';
	import FacetMatrix from './FacetMatrix.svelte';
	import HybridSearch from './HybridSearch.svelte';
	import VisualQuery from './VisualQuery.svelte';

	let currentInput = () => {
	};
	let handleKeyPress = () => {
	};
	let removeToken = () => {
	};
	let addOperator = () => {
	};
	let addFilter = () => {
	};
	let addGroup = () => {
	};

	// Example tokens array for SearchBar.svelte
	const tokens = [
		{
			id: 1,
			type: 'condition',
			field: 'tag',
			value: 'mod',
			negate: false
		},
		{
			id: 2,
			type: 'operator',
			value: 'AND'
		},
		{
			id: 3,
			type: 'group',
			value: '('
		},
		{
			id: 4,
			type: 'condition',
			field: 'genre',
			value: 'Science Fiction',
			negate: false
		},
		{
			id: 5,
			type: 'operator',
			value: 'OR'
		},
		{
			id: 6,
			type: 'condition',
			field: 'theme',
			value: 'Cyberpunk',
			negate: false
		},
		{
			id: 7,
			type: 'group',
			value: ')'
		},
		{
			id: 8,
			type: 'operator',
			value: 'NOT'
		},
		{
			id: 9,
			type: 'condition',
			field: 'language',
			value: 'Japanese',
			negate: true
		}
	];

	// Simpler examples:
	const singleToken = [
		{ id: 1, type: 'condition', field: 'author', value: 'CyberSmith', negate: false }
	];

	const mixedTokens = [
		{ id: 1, type: 'condition', field: 'tag', value: 'weapons', negate: false },
		{ id: 2, type: 'operator', value: 'AND' },
		{ id: 3, type: 'condition', field: 'score', value: '>7.5', negate: false }
	];
	// Example filters array for FacetedSearch.svelte
	const filters = [
		{
			id: 1,
			field: 'tag',
			value: 'mod',
			operator: 'AND',  // First item usually ignores operator
			negate: false
		},
		{
			id: 2,
			field: 'tag',
			value: 'translation',
			operator: 'NOT',
			negate: true
		},
		{
			id: 3,
			field: 'genre',
			value: 'Science Fiction',
			operator: 'OR',
			negate: false
		},
		{
			id: 4,
			field: 'language',
			value: 'Chinese',
			operator: 'AND',
			negate: false
		}
	];

	// Simpler examples:
	const singleFilter = [
		{ id: 1, field: 'author', value: 'NovaDesign', operator: 'AND', negate: false }
	];

	const rangeFilter = [
		{ id: 1, field: 'last_updated', value: '>2024-01-01', operator: 'AND', negate: false },
		{ id: 2, field: 'score', value: '5..8', operator: 'AND', negate: false }
	];
</script>
<div class="border border-amber-500 p-1">
	<span class="label">Search Bar</span>
	{@render searchBar()}
</div>

<div class="border border-amber-500 p-1">
	<span class="label">Facet Search</span>
	{@render facetSearch()}
</div>

<div class="border border-amber-500 p-1">
	<span class="label">Command Search</span>
	<CommandSearch></CommandSearch>
</div>

<div class="border border-amber-500 p-1">
	<span class="label">Facet Matrix</span>
	<FacetMatrix></FacetMatrix>
</div>

<div class="border border-amber-500 p-1">
	<span class="label">Hybrid Search</span>
	<HybridSearch></HybridSearch>
</div>

<div class="border border-amber-500 p-1">
	<span class="label">Visual Query</span>
	<VisualQuery></VisualQuery>
</div>


{#snippet searchBar()}
	<!-- SearchBar.svelte -->
	<div class="flex flex-wrap items-center gap-2 p-2 border rounded">
		{#each tokens as token (token.id)}
    <span class="flex items-center gap-1 px-2 py-1 rounded-full">
      {#if token.negate}<span class="text-red-500">!</span>{/if}
			<span class="font-semibold">{token.field}:</span>
      <span>"{token.value}"</span>
      <button onclick={() => removeToken(token.id)}>×</button>
    </span>
		{/each}
		<input
			bind:value={currentInput}
			onkeydown={handleKeyPress}
			placeholder="tag:value"
			class="flex-grow min-w-[120px] p-2 outline-none input"
		>
	</div>
{/snippet}

{#snippet facetSearch()}

	<!-- Operator Buttons -->
	<div class="flex gap-2 mt-2">
		<button onclick={addOperator('AND')} class="px-3 py-1 border">AND</button>
		<button onclick={addOperator('OR')} class="px-3 py-1 border">OR</button>
		<button onclick={addOperator('NOT')} class="px-3 py-1 border">NOT</button>
		<button onclick={addGroup} class="px-3 py-1 border">( )</button>
	</div>

	<!-- FacetedSearch.svelte -->
	{#each filters as filter}
		<div class="input-group grid-cols-[auto_1fr_auto]">
			<select class="ig-select" bind:value={filter.field}>
				<option value="tag">Tag</option>
				<option value="genre">Genre</option>
				<option value="language">Language</option>
			</select>

			<input class="ig-input" bind:value={filter.value} placeholder="Value">

			<select class="ig-select" bind:value={filter.operator}>
				<option>AND</option>
				<option>OR</option>
				<option>NOT</option>
			</select>
		</div>
	{/each}
	<button class="btn" onclick={addFilter}>+ Add Filter</button>
	<button class="btn" onclick={addGroup}>+ Add Group</button>
{/snippet}