<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { createDocumentEntry } from '$lib/api';
	import StepLayout from '$lib/components/onboarding/StepLayout.svelte';
	import { getOnboarding } from '$lib/stores/onboarding.svelte';
	import { t } from '$lib/i18n';

	const onboarding = getOnboarding();

	let articleUrl = $state('');
	let submitting = $state(false);
	let saveMessage = $state('');
	let saving = $state(false);

	const isValidUrl = $derived.by(() => {
		try {
			const url = new URL(articleUrl.trim());
			if (url.protocol !== 'https:' && url.protocol !== 'http:') return false;
			const parts = url.hostname.split('.').filter((part) => part.length > 0);
			const tld = parts[parts.length - 1];
			return parts.length >= 2 && tld !== undefined && tld.length >= 2;
		} catch {
			return false;
		}
	});

	async function saveArticle() {
		if (!isValidUrl) return;
		saving = true;
		saveMessage = '';
		try {
			const { data, error } = await createDocumentEntry({ body: { url: articleUrl.trim() } });
			saveMessage =
				data && !error ? $t('onboarding_content_saved') : $t('onboarding_content_save_failed');
			if (data && !error) articleUrl = '';
		} catch {
			saveMessage = $t('onboarding_content_save_failed');
		} finally {
			saving = false;
		}
	}

	async function handleContinue() {
		submitting = true;
		try {
			if (await onboarding.completeStep(2)) goto(resolve('/onboarding/feeds'));
		} finally {
			submitting = false;
		}
	}

	async function handleSkip() {
		submitting = true;
		try {
			if (await onboarding.completeStep(2)) goto(resolve('/onboarding/feeds'));
		} finally {
			submitting = false;
		}
	}
</script>

<StepLayout
	title={$t('onboarding_content_title')}
	description={$t('onboarding_content_description')}
	currentStep={2}
	showSkip
	{submitting}
	onContinue={handleContinue}
	onSkip={handleSkip}
>
	<div class="content">
		<div class="url-row">
			<input
				type="url"
				class="url-input"
				bind:value={articleUrl}
				placeholder="https://example.com/article"
				aria-label={$t('onboarding_article_url')}
			/>
			<button type="button" class="save-btn" disabled={!isValidUrl || saving} onclick={saveArticle}>
				{saving ? $t('common_saving') : $t('common_save')}
			</button>
		</div>
		{#if saveMessage}<p class="status-message">{saveMessage}</p>{/if}

		{#if onboarding.error}<p class="status-message error">{onboarding.error}</p>{/if}
	</div>
</StepLayout>

<style>
	.content {
		display: flex;
		flex-direction: column;
		width: 100%;
	}

	.url-row {
		display: flex;
		margin-bottom: 8px;
		border: 1px solid var(--border-secondary);
		border-radius: 8px;
		overflow: hidden;
	}

	.url-input {
		flex: 1;
		height: 40px;
		padding: 0 14px;
		border: none;
		outline: none;
		background: var(--bg-elevated);
		color: var(--text-primary);
		font: inherit;
		font-size: 15px;
	}

	.url-input::placeholder {
		color: var(--text-secondary);
	}

	.save-btn {
		height: 40px;
		padding: 0 16px;
		border: none;
		background: var(--accent);
		color: var(--text-on-color);
		font: inherit;
		font-size: 15px;
		font-weight: 500;
		cursor: pointer;
	}

	.save-btn:hover:not(:disabled) {
		background: var(--accent-hover);
	}

	.save-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.status-message {
		margin: 0;
		color: var(--success);
		font-size: 12px;
		line-height: 1.4;
	}

	.status-message.error {
		margin-top: 10px;
		color: var(--destructive);
	}
</style>
