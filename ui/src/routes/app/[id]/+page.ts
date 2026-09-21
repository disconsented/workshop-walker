import {
	app,
	language,
	limit,
	orderBy,
	searchProps,
	tags,
	title,
	updatedAfter,
	updatedBefore
} from './store.svelte';
import type { PageLoad } from '../../../../.svelte-kit/types/src/routes/app/[id]/$types';
import { parseSafeJSON } from '$lib/parser';

export const prerender = false;
let firstRun = true;
export const load: PageLoad = async ({ fetch, params, url }) => {
	loadParams(url.searchParams);

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

	if (title.v) {
		paramList.push(['title', title.v]);
	}

	if (updatedBefore.v) {
		paramList.push(['updated_before', updatedBefore.v / 1000]);
	}

	if (updatedAfter.v) {
		paramList.push(['updated_after', updatedAfter.v / 1000]);
	}

	if (searchProps.v) {
		searchProps.v.forEach(({ positive }, prop_string) => {
			if (positive) {
				paramList.push(['positive_props', prop_string]);
			} else {
				paramList.push(['negative_props', prop_string]);
			}
		});
	}

	paramList.push(['app', params.id]);

	const appRequest = fetch(`/api/app/${params.id}`).then(async (res) => {
		app.v = await res.text().then(parseSafeJSON);
		if (firstRun && tags.v.length == 0) {
			tags.v = app.v.tags.filter((tag) => app.v.default_tags.some((e) => e === tag));
			app.v.default_tags.forEach((tag) => {
				paramList.push(['tags', tag]);
			});
		}

		firstRun = false;
	});

	return {
		// Streamed: the app name is only known once /api/app resolves.
		breadcrumbs: appRequest.then(() => [
			{ title: app.v.name ?? params.id, href: `/app/${params.id}` }
		]),
		appRequest: appRequest,
		searchRequest: appRequest.then(() =>
			fetch(`/api/list?` + new URLSearchParams(paramList).toString()).then(async (res) => {
				if (res.ok) {
					return res.text().then(parseSafeJSON);
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

function loadParams(params: URLSearchParams) {
	console.debug('loadingParams', params);
	const paramLanguage = params.get('language');
	if (paramLanguage) {
		language.v = paramLanguage;
	} else {
		language.v = '1';
	}

	const paramOrderBy = params.get('order_by');
	if (paramOrderBy) {
		orderBy.v = paramOrderBy;
	} else {
		orderBy.v = 'LastUpdated';
	}

	const paramLimit = params.get('limit');
	if (paramLimit) {
		limit.v = Number(paramLimit);
	} else {
		limit.v = 100;
	}

	const paramTitle = params.get('title');
	if (paramTitle) {
		title.v = paramTitle;
	} else {
		title.v = undefined;
	}

	const paramUpdatedBefore = params.get('updated_before');
	if (paramUpdatedBefore) {
		updatedBefore.v = new Date(Number(paramUpdatedBefore) * 1000);
	} else {
		updatedBefore.v = undefined;
	}

	const paramUpdatedAfter = params.get('updated_after');
	if (paramUpdatedAfter) {
		updatedAfter.v = new Date(Number(paramUpdatedAfter) * 1000);
	} else {
		updatedAfter.v = undefined;
	}

	const paramTags = params.getAll('tags');
	if (paramTags.length > 0) {
		tags.v = paramTags;
	} else {
		tags.v = [];
	}

	searchProps.v.clear();
	const paramPositiveProps = params.getAll('positive_props');
	if (paramPositiveProps.length > 0) {
		paramPositiveProps.forEach((prop) => {
			const prop_split = prop.split(':');
			searchProps.v.set(prop, {
				property: { class: prop_split[0], value: prop_split[1] },
				positive: true
			});
		});
	}

	const paramNegativeProps = params.getAll('negative_props');
	if (paramNegativeProps.length > 0) {
		paramNegativeProps.forEach((prop) => {
			const prop_split = prop.split(':');
			searchProps.v.set(prop, {
				property: { class: prop_split[0], value: prop_split[1] },
				positive: false
			});
		});
	}
}
