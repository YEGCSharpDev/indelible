import { describe, it, expect } from 'vitest';
import { parseIntegrationCallback } from '$lib/integrations/callback';

function url(query: string): URL {
	return new URL(`https://app.example.com/preferences/integrations${query}`);
}

describe('parseIntegrationCallback', () => {
	it('returns null when no integration params are present', () => {
		expect(parseIntegrationCallback(url(''))).toBeNull();
		expect(parseIntegrationCallback(url('?foo=bar'))).toBeNull();
	});

	it('returns success with provider when ?connected=… is present', () => {
		expect(parseIntegrationCallback(url('?connected=miniflux'))).toEqual({
			kind: 'success',
			provider: 'miniflux'
		});
	});

	it('returns denied when integration_error=denied', () => {
		expect(parseIntegrationCallback(url('?integration_error=denied&provider=miniflux'))).toEqual({
			kind: 'denied',
			provider: 'miniflux'
		});
	});

	it('returns provider_error for integration_error=provider_error', () => {
		expect(
			parseIntegrationCallback(url('?integration_error=provider_error&provider=miniflux'))
		).toEqual({ kind: 'provider_error', provider: 'miniflux' });
	});

	it('returns server_error for integration_error=server', () => {
		expect(parseIntegrationCallback(url('?integration_error=server&provider=miniflux'))).toEqual({
			kind: 'server_error',
			provider: 'miniflux'
		});
	});

	it('collapses unknown integration_error kinds to server_error', () => {
		expect(parseIntegrationCallback(url('?integration_error=banana&provider=miniflux'))).toEqual({
			kind: 'server_error',
			provider: 'miniflux'
		});
	});

	it('tolerates a missing provider on errors', () => {
		expect(parseIntegrationCallback(url('?integration_error=denied'))).toEqual({
			kind: 'denied',
			provider: null
		});
	});

	it('prefers ?connected over ?integration_error when both are present', () => {
		expect(
			parseIntegrationCallback(
				url('?connected=miniflux&integration_error=denied&provider=miniflux')
			)
		).toEqual({ kind: 'success', provider: 'miniflux' });
	});
});
