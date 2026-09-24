import { SvelteMap } from 'svelte/reactivity';

export const language = $state({ v: '1' });
export const tags: { v: string[] } = $state({ v: [] });
export const orderBy = $state({ v: 'TrendMonth' });
export const limit = $state({ v: 50 });
export const title = $state({ v: undefined });
export const lastUpdatedGte: { v: Date | undefined } = $state({ v: undefined });
export const lastUpdatedLte: { v: Date | undefined } = $state({ v: undefined });
export const app = $state({ v: {} });
export const searchProps: {
	v: SvelteMap<string, { property: { class: string; value: string }; positive: boolean }>;
} = $state({ v: new SvelteMap() });
