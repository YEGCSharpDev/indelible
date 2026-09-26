import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';

import type { DocumentListEntry } from '$lib/api';
import DetailPanel from '$lib/components/library/DetailPanel.svelte';

function stub() {
	return {
		default: vi.fn(() => ({ c: vi.fn(), m: vi.fn(), p: vi.fn(), d: vi.fn() }))
	};
}

vi.mock('$lib/components/library/DetailInfo.svelte', () => stub());
vi.mock('$lib/components/library/NotebookTab.svelte', () => stub());
vi.mock('$lib/components/library/EditMetadataPanel.svelte', () => stub());

function item(): DocumentListEntry {
	return {
		id: 'doc_1',
		document_id: 'doc_1',
		title: 'Field Notes',
		document_type: 'article',
		item_type: 'article',
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

describe('DetailPanel tabs', () => {
	it('renders Info and Notebook tabs without Chat', () => {
		render(DetailPanel, {
			props: { item: item() }
		});

		expect(screen.getByRole('tab', { name: 'Info' })).toBeTruthy();
		expect(screen.getByRole('tab', { name: 'Notebook' })).toBeTruthy();
		expect(screen.queryByRole('tab', { name: 'Chat' })).toBeNull();
	});

	it('switches between Info and Notebook tabs', async () => {
		render(DetailPanel, {
			props: { item: item() }
		});

		const infoTab = screen.getByRole('tab', { name: 'Info' });
		const notebookTab = screen.getByRole('tab', { name: 'Notebook' });

		expect(infoTab.getAttribute('aria-selected')).toBe('true');
		expect(notebookTab.getAttribute('aria-selected')).toBe('false');

		await fireEvent.click(notebookTab);

		expect(infoTab.getAttribute('aria-selected')).toBe('false');
		expect(notebookTab.getAttribute('aria-selected')).toBe('true');
	});
});
