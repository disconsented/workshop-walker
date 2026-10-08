import { SvelteMap } from 'svelte/reactivity';

export const language = $state({ v: '1' });
export const tags: { v: string[] } = $state({ v: [] });
export const orderBy = $state({ v: 'TrendWeek' });
export const limit = $state({ v: 50 });
export const page = $state({ v: 0 });
export const title = $state({ v: undefined });
export const lastUpdatedPeriod = $state({ v: undefined });
export const lastUpdatedGte: { v: Date | undefined } = $state({ v: undefined });
export const lastUpdatedLte: { v: Date | undefined } = $state({ v: undefined });
export const createdPeriod = $state({ v: undefined });
export const createdGte: { v: Date | undefined } = $state({ v: undefined });
export const createdLte: { v: Date | undefined } = $state({ v: undefined });
export const app = $state({ v: {} });
export const searchProps: {
	v: SvelteMap<string, { property: { class: string; value: string }; positive: boolean }>;
} = $state({ v: new SvelteMap() });
