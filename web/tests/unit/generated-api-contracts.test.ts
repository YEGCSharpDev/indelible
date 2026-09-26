import { describe, expect, it } from 'vitest';
import type { StreamDocumentAssetResponses } from '$lib/api/generated';

describe('generated API contracts', () => {
	it('types document asset downloads as binary blobs', () => {
		const asset: StreamDocumentAssetResponses[200] = new Blob(['asset']);
		expect(asset).toBeInstanceOf(Blob);
	});
});
