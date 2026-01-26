export const prerender = false;
export const load = async ({ fetch, params }) => {
	let request = {
		data: await fetch(`/api/item/${params.item}`).then((res) => res.json())
		// app: await fetch(`/api/item/${params.item}/app`).then((res) => res.json())
	};
	return request;
};
