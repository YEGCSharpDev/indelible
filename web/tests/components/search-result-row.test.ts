import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import SearchResultRow from '$lib/components/search/SearchResultRow.svelte';
import type { SearchResultResponse } from '$lib/api/generated/types.gen';

function baseResult(overrides: Partial<SearchResultResponse> = {}): SearchResultResponse {
	return {
		document_id: 'doc_test',
		result_kind: 'document',
		title: 'Test Newsletter',
		snippet: 'Body snippet',
		score: 0.5,
		content_type: 'email',
		saved_at: new Date('2026-05-18T10:00:00Z').toISOString(),
		updated_at: new Date('2026-05-18T10:00:00Z').toISOString(),
		...overrides
	};
}

describe('SearchResultRow', () => {
	it('renders correctly', () => {
		render(SearchResultRow, {
			props: {
				result: baseResult(),
				selected: false,
				onSelect: () => {},
				onOpen: () => {}
			}
		});
		const title = screen.getByText('Test Newsletter');
		expect(title).toBeDefined();
	});
});
