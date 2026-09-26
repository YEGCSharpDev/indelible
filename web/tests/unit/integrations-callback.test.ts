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
		expect(parseIntegrationCallback(url('?connected=obsidian'))).toEqual({
			kind: 'success',
			provider: 'obsidian'
		});
	});

	it('returns denied when integration_error=denied', () => {
		expect(parseIntegrationCallback(url('?integration_error=denied&provider=obsidian'))).toEqual({
			kind: 'denied',
			provider: 'obsidian'
		});
	});

	it('returns provider_error for integration_error=provider_error', () => {
		expect(
			parseIntegrationCallback(url('?integration_error=provider_error&provider=obsidian'))
		).toEqual({ kind: 'provider_error', provider: 'obsidian' });
	});

	it('returns server_error for integration_error=server', () => {
		expect(parseIntegrationCallback(url('?integration_error=server&provider=obsidian'))).toEqual({
			kind: 'server_error',
			provider: 'obsidian'
		});
	});

	it('collapses unknown integration_error kinds to server_error', () => {
		expect(parseIntegrationCallback(url('?integration_error=banana&provider=obsidian'))).toEqual({
			kind: 'server_error',
			provider: 'obsidian'
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
			parseIntegrationCallback(url('?connected=obsidian&integration_error=denied&provider=obsidian'))
		).toEqual({ kind: 'success', provider: 'obsidian' });
	});
});
