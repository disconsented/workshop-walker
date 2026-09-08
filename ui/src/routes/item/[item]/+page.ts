export const prerender = false;
export const load = async ({ fetch, params }) => {
	const item = await fetch(`/api/item/${params.item}`).then((res) => res.json());

	return {
		data: item,
		// The parent app crumb is streamed so its name does not hold up the page.
		breadcrumbs: fetch(`/api/app/${item.app}`)
			.then((res) => (res.ok ? res.json() : undefined))
			.catch(() => undefined)
			.then((parent) => [
				{ title: parent?.name ?? String(item.app), href: `/app/${item.app}` },
				{ title: item.title, href: `/item/${params.item}` }
			])
	};
};
