import { orderBy, language, tags, limit, title, lastUpdated } from './store.svelte';
import type { PageLoad } from '../../../../.svelte-kit/types/src/routes/app/[id]/$types';

export const prerender = false;
export const load: PageLoad = async ({ fetch, params }) => {
	let paramList = [];
	if (language.v) {
		paramList.push(['languages', language.v]);
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

	if (title.v) {
		paramList.push(['title', title.v]);
	}

	if (lastUpdated.v) {
		paramList.push(['last_updated', Date.parse(lastUpdated.v) / 1000]);
	}

	paramList.push(['app', params.id]);
	const searchParams = new URLSearchParams(paramList);

	return {
		appRequest: await fetch(`/api/app/${params.id}`).then(res => res.json()),
		searchRequest: fetch(`/api/list?` + searchParams.toString()).then((res) => {
			console.log('api/list Result', res);
			if (res.ok) {
				return res.json();
			}
			const status = res.status;
			const statusText = res.statusText;
			return res.text().then((text) => {
				return {
					statusText: statusText,
					status: status,
					body: text
				};
			});
		}),
		id: params.id
	};
};
