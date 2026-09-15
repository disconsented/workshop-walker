import type { PageLoad } from './$types';
import { parseSafeJSON } from '$lib/parser';

export const prerender = false;

export const load: PageLoad = async ({ fetch }) => {
	// Both lists are streamed, so a slow query on one tab does not hold up the
	// other two.
	const get = async (path: string) => {
		const res = await fetch(path);
		if (!res.ok) {
			throw new Error(`${path} returned ${res.status} ${res.statusText}`);
		}
		return res.text().then(parseSafeJSON);
	};

	return {
		breadcrumbs: [{ title: 'Admin', href: '/admin' }],
		users: get('/api/admin/users'),
		properties: get('/api/admin/properties')
	};
};
