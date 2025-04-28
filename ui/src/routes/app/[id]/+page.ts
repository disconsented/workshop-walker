import { orderBy, language, tags, limit } from './store.svelte';
import type { PageLoad } from '../../../../.svelte-kit/types/src/routes/app/[id]/$types';
export const prerender = false;
export const load: PageLoad = async ({ fetch, params }) => {
	let paramList = [];
	if (language.v) {
		paramList.push(['language', language.v]);
	}
	if (tags.v) {
		tags.v.forEach((tag) => {
			paramList.push(['tags', tag]);
		});
	}
	if (orderBy.v) {
		paramList.push(['order_by', orderBy.v]);
	}

	if (limit.v) {
		paramList.push(['limit', limit.v]);
	}

	const searchParams = new URLSearchParams(paramList);

	const res = await fetch(`/api/list?` + searchParams.toString());
	const item = await res.json();
	return { result: item };
};
