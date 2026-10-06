import { parseSafeJSON, reviver } from '$lib/parser';
import slugify from 'slugify';

export const prerender = false;
export const load = async ({ fetch, params }) => {
	const item = await fetch(`/api/item/${params.item}`)
		.then((res) => res.text())
		.then(parseSafeJSON);

	const app = await fetch(`/api/app/${item.app}`)
		.then((res) => res.text())
		.then(parseSafeJSON);

	return {
		data: item,
		app: app,
		breadcrumbs: [
			{ title: app?.name ?? String(item.app), href: `/app/${item.app}/${slugify(app?.name)}` },
			{ title: item.title, href: `/item/${params.item}/${slugify(item.title)}` }
		]
	};
};
