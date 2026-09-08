export const prerender = false;
export const load: PageLoad = async ({ fetch, params }) => {
	return {
		breadcrumbs: [{ title: 'Admin', href: '/admin' }],
		users: fetch(`/api/admin/users`).then((res) => {
			if (res.ok) {
				return res.json();
			}
		}),
		properties: fetch(`/api/admin/properties`).then((res) => {
			if (res.ok) {
				return res.json();
			}
		})
	};
};
