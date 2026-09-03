export const prerender = false;
export const load = async ({ fetch, params }) => {
	return {
		data: await fetch(`/api/item/${params.item}`).then((res) => res.json())
	};
};
