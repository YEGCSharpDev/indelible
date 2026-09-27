import { describe, expect, it } from 'vitest';

import {
	getLibraryFilterFieldDef,
	getVisibleLibraryFilterFields
} from '../../src/lib/utils/library-filter-fields';

const fieldKeys = (activeType?: string) =>
	getVisibleLibraryFilterFields(activeType).map((field) => field.key);

describe('getVisibleLibraryFilterFields', () => {
	it('keeps common fields visible in every library section', () => {
		expect(fieldKeys('articles')).toEqual(
			expect.arrayContaining(['tag', 'item_type', 'domain', 'collection'])
		);
		expect(fieldKeys()).toEqual(
			expect.arrayContaining(['tag', 'item_type', 'domain', 'collection'])
		);
	});

	it('does not offer podcasts or emails as a launch content-type filter', () => {
		const options = getLibraryFilterFieldDef('item_type').options ?? [];
		const values = options.map((option) => option.value);

		expect(values).not.toContain('podcast');
		expect(values).not.toContain('email');
	});
});
