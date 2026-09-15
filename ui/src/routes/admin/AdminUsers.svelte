<script lang="ts">
	import { SvelteMap } from 'svelte/reactivity';
	import { stringifySafeJSON } from '$lib/parser';

	type User = {
		id: number | bigint;
		name?: string;
		admin: boolean;
		banned: boolean;
		last_logged_in: string;
	};

	type Patch = { admin?: boolean; banned?: boolean };

	interface Props {
		users: User[];
	}

	let { users }: Props = $props();

	// A change goes here and not into the row, so a row keeps its place until the
	// next load.
	const changed = new SvelteMap<string, Patch>();

	function view(user: User) {
		return { ...user, ...changed.get(String(user.id)) };
	}

	async function patch(user: User, change: Patch) {
		const key = String(user.id);
		const previous = changed.get(key) ?? {};
		changed.set(key, { ...previous, ...change });

		const res = await fetch('/api/admin/users', {
			method: 'PUT',
			headers: { 'Content-Type': 'application/json' },
			body: stringifySafeJSON({ id: user.id, ...change })
		});

		if (!res.ok) {
			changed.set(key, previous);
			console.error('Could not patch the user', res.status, res.statusText);
		}
	}
</script>

<div class="table-wrap">
	<table class="table">
		<thead>
			<tr>
				<th>ID</th>
				<th>Name</th>
				<th>Admin</th>
				<th>Banned</th>
				<th>Last Logged In</th>
			</tr>
		</thead>
		<tbody>
			{#each users as user (user.id)}
				{@const current = view(user)}
				<tr class="hover:preset-tonal-primary">
					<td>{user.id}</td>
					<td>{user.name ?? 'unpopulated'}</td>
					<td>
						<input
							type="checkbox"
							class="checkbox"
							checked={current.admin}
							onchange={(event) => patch(user, { admin: event.currentTarget.checked })}
						/>
					</td>
					<td>
						<input
							type="checkbox"
							class="checkbox"
							checked={current.banned}
							onchange={(event) => patch(user, { banned: event.currentTarget.checked })}
						/>
					</td>
					<td>{new Date(user.last_logged_in).toLocaleString()}</td>
				</tr>
			{:else}
				<tr><td colspan="5" class="text-surface-500">No users.</td></tr>
			{/each}
		</tbody>
	</table>
</div>
