import { language, tags } from './store.svelte';
import type { PageLoad } from '../../../../.svelte-kit/types/src/routes/app/[id]/$types';

export const load: PageLoad = async ({ fetch, params }) => {
	// const url = new URL();

	let paramList = [];
	if (language.v) {
	}
	if (tags.v) {
		tags.v.forEach((tag) => {
			paramList.push(['tags', tag]);
		});
	}

	const searchParams = new URLSearchParams(paramList);

	const res = await fetch(`/api/list?` + searchParams.toString());
	const item = await res.json();
	return { result: item };
};
