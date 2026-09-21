import { SvelteMap } from 'svelte/reactivity';

export const language = $state({ v: '1' });
export const tags: { v: string[] } = $state({ v: [] });
export const orderBy = $state({ v: 'LastUpdated' });
export const limit = $state({ v: 50 });
export const title = $state({ v: undefined });
export const updatedBefore: { v: Date | undefined } = $state({ v: undefined });
export const updatedAfter: { v: Date | undefined } = $state({ v: undefined });
export const app = $state({ v: {} });
export const searchProps: {
	v: SvelteMap<string, { property: { class: string; value: string }; positive: boolean }>;
} = $state({ v: new SvelteMap() });
