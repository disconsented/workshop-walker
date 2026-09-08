// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
import type { Breadcrumbs } from '$lib/breadcrumbs';

declare global {
	namespace App {
		// interface Error {}
		// interface Locals {}
		interface PageData {
			/** Navbar trail for this page. See $lib/breadcrumbs. */
			breadcrumbs?: Breadcrumbs;
		}
		// interface PageState {}
		// interface Platform {}
	}
}

export {};
