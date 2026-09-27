export interface AccountSnapshotInput {
	displayName: string;
	hasAvatar: boolean;
	hasPendingAvatar: boolean;
}

export function createAccountSnapshot(input: AccountSnapshotInput): string {
	return JSON.stringify({
		displayName: input.displayName,
		hasAvatar: input.hasAvatar,
		hasPendingAvatar: input.hasPendingAvatar
	});
}

export function getAccountUsername(username: string | null | undefined): string {
	return username ? `@${username}` : '';
}

export function getAccountAvatarInitial({
	displayName,
	username
}: {
	displayName?: string | null | undefined;
	username?: string | null | undefined;
}): string {
	return (displayName?.[0] ?? username?.[0] ?? 'U').toUpperCase();
}

export function formatMemberSince(iso: string | null | undefined): string {
	if (!iso) return '';
	const parsed = new Date(iso);
	if (Number.isNaN(parsed.getTime())) return '';
	return get(date)(parsed, { month: 'short', year: 'numeric' });
}

export function isDeleteUsernameConfirmed(
	confirmUsername: string,
	accountUsername: string | null | undefined
): boolean {
	return (
		confirmUsername.trim().toLowerCase() === (accountUsername ?? '').toLowerCase() &&
		confirmUsername.length > 0
	);
}
import { date } from '$lib/i18n';
import { get } from 'svelte/store';
