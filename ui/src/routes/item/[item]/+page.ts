import type { PageLoad } from '../../../../.svelte-kit/types/src/routes';

export const load: PageLoad = async ({ fetch, params }) => {
	const res = await fetch(`/api/item/${params.item}`);
	const item = await res.json();
	return item;
};
