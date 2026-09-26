import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';

import type { DocumentListEntry } from '$lib/api';
import BookDetailPanel from '$lib/components/reader/book/BookDetailPanel.svelte';

function item(): DocumentListEntry {
	return {
		id: 'doc_pdf',
		document_id: 'doc_pdf',
		title: 'Scanned field notes',
		document_type: 'pdf',
		item_type: 'pdf',
		object: 'library_entry',
		source: 'manual',
		created_at: '2026-08-12T00:00:00Z',
		updated_at: '2026-08-12T00:00:00Z',
		saved_at: '2026-08-12T00:00:00Z',
		triage_state: 'inbox',
		is_favorite: false,
		is_shortlisted: false
	} as DocumentListEntry;
}

const bookMetadata = {
	title: 'Scanned field notes',
	author: 'Indelible',
	totalChapters: 4
};

describe('BookDetailPanel tabs', () => {
	it('renders Info and Notebook tabs and does not include Chat', () => {
		render(BookDetailPanel, {
			props: { item: item(), bookMetadata, progress: 10 }
		});

		expect(screen.getByRole('tab', { name: 'Info' })).toBeTruthy();
		expect(screen.getByRole('tab', { name: 'Notebook' })).toBeTruthy();
		expect(screen.queryByRole('tab', { name: 'Chat' })).toBeNull();
	});
});
