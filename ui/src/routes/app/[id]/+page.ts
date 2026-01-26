import { orderBy, language, tags, limit, title, lastUpdated, app } from './store.svelte';
import type { PageLoad } from '../../../../.svelte-kit/types/src/routes/app/[id]/$types';

export const prerender = false;
let firstRun = true;
export const load: PageLoad = async ({ fetch, params }) => {
	let paramList = [];
	if (language.v) {
		paramList.push(['languages', language.v]);
	}
	if (tags.v) {
		tags.v.forEach((tag) => {
			paramList.push(['tags', tag.id]);
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

	const appRequest = fetch(`/api/app/${params.id}`).then(async (res) => {
		firstRun = !!app.v;
		app.v = await res.json();
		if (firstRun) {
			tags.v = app.v.tags.filter((tag) => app.v.default_tags.some((e) => e.id === tag.id));
			app.v.default_tags.forEach((tag) => {
				paramList.push(['tags', tag.id]);
			});
		}
	});

	return {
		appRequest: appRequest,
		searchRequest: appRequest.then(() =>
			fetch(`/api/list?` + new URLSearchParams(paramList).toString()).then(async (res) => {
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
			})
		),
		id: params.id
	};
};
