import { describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/svelte';
import DeleteAccountDialog from '../../src/routes/(app)/preferences/account/components/DeleteAccountDialog.svelte';

function renderDialog(overrides = {}) {
	const onClose = vi.fn();
	const onConfirmUsernameChange = vi.fn();
	const onDelete = vi.fn();
	render(DeleteAccountDialog, {
		props: {
			username: 'testuser',
			confirmUsername: '',
			deleteUsernameMatches: false,
			deleting: false,
			error: '',
			onClose,
			onConfirmUsernameChange,
			onDelete,
			...overrides
		}
	});
	return { onClose, onConfirmUsernameChange, onDelete };
}

describe('DeleteAccountDialog', () => {
	it('renders destructive account deletion copy and disables delete until confirmed', () => {
		renderDialog();
		expect(screen.getByRole('dialog', { name: 'Delete your account' })).toBeTruthy();
		expect(screen.getByText(/tied to testuser\. This cannot be recovered\./)).toBeTruthy();
		expect(screen.getByRole<HTMLButtonElement>('button', { name: 'Delete forever' }).disabled).toBe(
			true
		);
	});

	it('emits typed username changes and delete/cancel callbacks', async () => {
		const handlers = renderDialog({ confirmUsername: 'testuser', deleteUsernameMatches: true });
		await fireEvent.input(screen.getByLabelText('Type your username to confirm'), {
			target: { value: 'testuser' }
		});
		await fireEvent.click(screen.getByRole('button', { name: 'Delete forever' }));
		await fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));

		expect(handlers.onConfirmUsernameChange).toHaveBeenCalledWith('testuser');
		expect(handlers.onDelete).toHaveBeenCalledOnce();
		expect(handlers.onClose).toHaveBeenCalledOnce();
	});

	it('shows errors and busy copy', () => {
		renderDialog({
			confirmUsername: 'testuser',
			deleteUsernameMatches: true,
			deleting: true,
			error: 'Account deletion failed'
		});
		expect(screen.getByText('Account deletion failed')).toBeTruthy();
		expect(screen.getByRole<HTMLButtonElement>('button', { name: 'Deleting…' }).disabled).toBe(
			true
		);
	});
});
