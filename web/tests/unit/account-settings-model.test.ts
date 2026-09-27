import { describe, expect, it } from 'vitest';
import {
	createAccountSnapshot,
	formatMemberSince,
	getAccountAvatarInitial,
	getAccountUsername,
	isDeleteUsernameConfirmed
} from '../../src/routes/(app)/preferences/account/account-model';

describe('account settings model', () => {
	it('derives the public username handle', () => {
		expect(getAccountUsername('sam')).toBe('@sam');
		expect(getAccountUsername(null)).toBe('');
	});

	it('uses display name before username for the avatar initial', () => {
		expect(getAccountAvatarInitial({ displayName: 'Mila Stone', username: 'mila' })).toBe('M');
		expect(getAccountAvatarInitial({ displayName: '', username: 'reader' })).toBe('R');
		expect(getAccountAvatarInitial({ displayName: '', username: null })).toBe('U');
	});

	it('formats member-since labels defensively', () => {
		expect(formatMemberSince('2026-04-10T12:00:00.000Z')).toBe('Apr 2026');
		expect(formatMemberSince(null)).toBe('');
		expect(formatMemberSince('not-a-date')).toBe('');
	});

	it('keeps the dirty snapshot stable for profile edits', () => {
		expect(
			createAccountSnapshot({
				displayName: 'Sam',
				hasAvatar: true,
				hasPendingAvatar: false
			})
		).toBe(
			JSON.stringify({
				displayName: 'Sam',
				hasAvatar: true,
				hasPendingAvatar: false
			})
		);
	});

	it('requires an exact username confirmation ignoring case and surrounding whitespace', () => {
		expect(isDeleteUsernameConfirmed(' TESTUSER ', 'testuser')).toBe(true);
		expect(isDeleteUsernameConfirmed('', 'testuser')).toBe(false);
		expect(isDeleteUsernameConfirmed('other', 'testuser')).toBe(false);
	});
});
