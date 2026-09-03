export const prerender = false;
export const load = async ({ fetch }) => {
	const res = await fetch(`/api/apps`);
	const apps = await res.json();

	return { items: apps };
};
