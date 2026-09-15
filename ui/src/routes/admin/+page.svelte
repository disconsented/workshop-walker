<script lang="ts">
	import { Tabs } from '@skeletonlabs/skeleton-svelte';
	import type { PageData } from './$types';
	import AdminApps from './AdminApps.svelte';
	import AdminProperties from './AdminProperties.svelte';
	import AdminUsers from './AdminUsers.svelte';

	let { data }: { data: PageData } = $props();

	let group = $state('properties');
</script>

<div class="mx-auto my-8 max-w-6xl">
	<h1 class="mb-6 text-2xl font-bold">Property Management System</h1>

	<Tabs value={group} onValueChange={(event) => (group = event.value)}>
		<Tabs.List>
			<Tabs.Trigger value="properties">Properties</Tabs.Trigger>
			<Tabs.Trigger value="users">Users</Tabs.Trigger>
			<Tabs.Trigger value="apps">Apps</Tabs.Trigger>
			<Tabs.Indicator />
		</Tabs.List>

		<Tabs.Content value="properties">
			{#await data.properties}
				{@render loading()}
			{:then properties}
				<AdminProperties {properties} />
			{:catch error}
				{@render failed(error)}
			{/await}
		</Tabs.Content>

		<Tabs.Content value="users">
			{#await data.users}
				{@render loading()}
			{:then users}
				<AdminUsers {users} />
			{:catch error}
				{@render failed(error)}
			{/await}
		</Tabs.Content>

		<Tabs.Content value="apps">
			<AdminApps />
		</Tabs.Content>
	</Tabs>
</div>

{#snippet loading()}
	<p class="text-surface-500 text-sm">Loading…</p>
{/snippet}

{#snippet failed(error: Error)}
	<p class="text-error-500">{error.message}</p>
{/snippet}
