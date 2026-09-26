import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';

import IntegrationConnectionCard from '$lib/components/integrations/IntegrationConnectionCard.svelte';
import IntegrationStatusPill from '$lib/components/integrations/IntegrationStatusPill.svelte';

describe('IntegrationConnectionCard', () => {
	it('renders a current-design card shell with status and errors', () => {
		render(IntegrationConnectionCard, {
			props: {
				title: 'Miniflux',
				tagline: 'Sync reading state and articles.',
				statusLabel: 'Needs attention',
				statusVariant: 'attention',
				errorMessage: 'Miniflux API token has expired.'
			}
		});

		expect(screen.getByTestId('integration-connection-card')).toBeTruthy();
		expect(screen.getByText('Miniflux')).toBeTruthy();
		expect(screen.getByText('Sync reading state and articles.')).toBeTruthy();
		expect(screen.getByText('Needs attention')).toBeTruthy();
		expect(screen.getByRole('alert').textContent).toContain('Miniflux API token has expired.');
	});
});

describe('IntegrationStatusPill', () => {
	it('marks syncing status with a stable variant and pulse indicator', () => {
		render(IntegrationStatusPill, {
			props: {
				variant: 'syncing',
				label: 'Syncing',
				pulse: true
			}
		});

		const pill = screen.getByTestId('integration-status-pill');
		expect(pill.getAttribute('data-variant')).toBe('syncing');
		expect(pill.textContent).toContain('Syncing');
		expect(pill.querySelector('.pulse-dot')).toBeTruthy();
	});
});
