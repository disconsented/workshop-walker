/** One crumb in the navbar trail. The last one is the current page. */
export interface Segment {
	title: string;
	href: string;
}

/**
 * A page publishes its trail by returning `breadcrumbs` from its `load`.
 * The value may be a promise so a page can stream a crumb whose label is only
 * known after a fetch; the navbar renders the resolved crumbs when they land.
 */
export type Breadcrumbs = Segment[] | Promise<Segment[]>;
