<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { t } from '$lib/i18n';
	import SettingsHero from '$lib/components/settings/SettingsHero.svelte';
	import type { IntegrationConnectionDto } from '$lib/api';
	import {
		loadIntegrationConnections,
		dispatchIntegrationSync,
		setupMinifluxConnection
	} from '$lib/api/integrations';
	import { onMount } from 'svelte';
	import { relativeTime } from '$lib/utils/relative-time';

	let loading = $state(true);
	let connections = $state<IntegrationConnectionDto[]>([]);
	let error = $state<string | null>(null);

	let url = $state('');
	let apiKey = $state('');
	let connecting = $state(false);

	let syncState = $state<'idle' | 'pending' | 'success' | 'error'>('idle');
	let syncError = $state<string | null>(null);

	const connection = $derived(connections.find((c) => c.provider === 'miniflux'));

	const statusLabel = $derived.by(() => {
		if (!connection) return $t('settings_miniflux_status_disconnected');
		if (syncState === 'pending') return $t('settings_miniflux_status_syncing');
		if (connection.status === 'error') return $t('settings_miniflux_status_error');
		return $t('settings_miniflux_status_connected');
	});

	const heroState = $derived.by(() => {
		if (!connection) return 'disconnected';
		if (syncState === 'pending') return 'syncing';
		if (connection.status === 'error') return 'error';
		return 'connected';
	});

	const lastSyncLabel = $derived.by(() => {
		return relativeTime(connection?.last_sync_at) ?? $t('settings_miniflux_never');
	});

	onMount(async () => {
		await refresh();
	});

	async function refresh() {
		loading = true;
		const result = await loadIntegrationConnections();
		if (result.success) {
			connections = result.data.connections;
		}
		loading = false;
	}

	async function connect() {
		if (!url || !apiKey) return;
		connecting = true;
		error = null;
		try {
			const result = await setupMinifluxConnection(url, apiKey);
			if (!result.success) {
				error = result.error;
			} else {
				await refresh();
			}
		} finally {
			connecting = false;
		}
	}

	async function handleSync() {
		if (!connection) return;
		syncState = 'pending';
		syncError = null;
		const result = await dispatchIntegrationSync(connection.id);
		if (result.success) {
			syncState = 'success';
			await refresh();
		} else {
			syncState = 'error';
			syncError = result.error;
		}
	}

	function disconnect() {
		// Just go back to hub for disconnect dialog
		void goto(resolve('/preferences/integrations'));
	}
</script>

<div class="obs-page" data-page-state={heroState}>
	<SettingsHero variant="miniflux">
		<div class="hero-watermark" aria-hidden="true">
			<svg
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<path d="M4 11a9 9 0 0 1 9 9" />
				<path d="M4 4a16 16 0 0 1 16 16" />
				<circle cx="5" cy="19" r="1" />
			</svg>
		</div>

		<div class="hero-text">
			<div class="hero-eyebrow">
				<svg viewBox="0 0 24 24" aria-hidden="true">
					<path d="M9 7h-3a3 3 0 1 0 0 6h3" />
					<path d="M15 17h3a3 3 0 1 0 0-6h-3" />
					<path d="M9 12h6" />
				</svg>
				{$t('settings_integrations')}
				<span class="eyebrow-sep">/</span>
				<svg
					viewBox="0 0 24 24"
					aria-hidden="true"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<path d="M4 11a9 9 0 0 1 9 9" />
					<path d="M4 4a16 16 0 0 1 16 16" />
					<circle cx="5" cy="19" r="1" />
				</svg>
				{$t('settings_miniflux')}
			</div>
			<h1 class="hero-headline">{$t('settings_miniflux_title')}</h1>
			<p class="hero-sub">
				{$t('settings_miniflux_description')}
			</p>
			<div class="hero-ops">
				<span class="hero-status-pill" data-status={heroState}>
					<span class="pulse"></span>{statusLabel}
				</span>
				{#if heroState !== 'disconnected'}
					<span class="ops-sep" aria-hidden="true"></span>
					<span>{$t('settings_miniflux_last_synced')} <strong>{lastSyncLabel}</strong></span>
				{/if}
			</div>
		</div>

		<div class="hero-gem" aria-hidden="true">
			<svg viewBox="0 0 240 280">
				<circle class="gem-glow" cx="120" cy="140" r="118" />
				<circle cx="120" cy="140" r="60" stroke-width="12" />
				<circle cx="120" cy="140" r="90" stroke-width="12" stroke-dasharray="10 20" />
			</svg>
		</div>
	</SettingsHero>

	<div class="body-area">
		<div class="settings-body">
			{#if loading}
				<p class="muted">{$t('common_loading')}</p>
			{:else if connection}
				<div class="card card-stack">
					<div class="row">
						<div>
							<p class="row-title">{$t('settings_miniflux_connection')}</p>
							<p class="row-sub">
								{$t('settings_miniflux_status_label', { values: { status: statusLabel } })}
							</p>
						</div>
						<div class="row-ops">
							<button
								class="btn btn-primary"
								onclick={handleSync}
								disabled={syncState === 'pending'}
							>
								{syncState === 'pending'
									? $t('settings_miniflux_syncing')
									: $t('settings_miniflux_force_resync')}
							</button>
							<button class="btn btn-secondary" onclick={disconnect}
								>{$t('settings_miniflux_disconnect')}</button
							>
						</div>
					</div>

					{#if syncError || connection.last_error}
						<div class="alert-block">
							<div class="alert">
								<strong>{$t('settings_miniflux_sync_failed')}</strong>
								<p>{$t('settings_miniflux_sync_error_details')}</p>
								<span>{syncError || connection.last_error}</span>
							</div>
						</div>
					{/if}
				</div>
			{:else}
				<div class="card" style="padding: 24px;">
					<form
						class="connect-form"
						onsubmit={(e) => {
							e.preventDefault();
							connect();
						}}
					>
						<div class="field">
							<label for="url">{$t('settings_miniflux_url')}</label>
							<input
								id="url"
								type="url"
								bind:value={url}
								placeholder={$t('settings_miniflux_url_placeholder')}
								required
								class="input"
							/>
						</div>
						<div class="field">
							<label for="api_key">{$t('settings_miniflux_api_key')}</label>
							<input id="api_key" type="password" bind:value={apiKey} required class="input" />
						</div>
						{#if error}
							<p class="error">{error}</p>
						{/if}
						<div style="margin-top: 8px;">
							<button type="submit" class="btn btn-primary" disabled={connecting}>
								{connecting ? $t('settings_miniflux_connecting') : $t('settings_miniflux_connect')}
							</button>
						</div>
					</form>
				</div>
			{/if}
		</div>
	</div>
</div>

<style>
	.obs-page {
		display: flex;
		flex-direction: column;
		min-height: 100%;
	}
	.obs-page :global(.hero-inner) {
		max-width: none;
		width: 100%;
	}
	.body-area {
		flex: 1;
		display: flex;
		flex-direction: column;
		background: var(--bg-secondary);
		position: relative;
	}
	.settings-body {
		flex: 1;
		padding: 28px 36px 80px;
		max-width: 940px;
		margin: 0 auto;
		width: 100%;
		display: flex;
		flex-direction: column;
		min-height: 0;
	}

	/* Hero Watermark & Gem */
	.hero-watermark {
		position: absolute;
		right: 200px;
		top: 50%;
		transform: translateY(-50%) rotate(15deg);
		width: 320px;
		height: 320px;
		pointer-events: none;
		opacity: 0.07;
		color: var(--accent);
		z-index: 0;
	}
	.hero-watermark svg,
	.hero-gem svg {
		width: 100%;
		height: 100%;
		stroke: currentColor;
		fill: none;
		stroke-linecap: round;
		stroke-linejoin: round;
	}
	.hero-gem {
		flex-shrink: 0;
		width: 220px;
		height: 240px;
		color: var(--accent);
		position: relative;
		z-index: 1;
	}
	.gem-glow {
		fill: color-mix(in oklab, var(--accent) 14%, transparent);
		stroke: none;
	}

	/* Hero Text */
	.hero-text {
		flex: 1 1 0;
		min-width: 0;
		position: relative;
		z-index: 1;
	}
	.hero-eyebrow {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		font-size: 11px;
		font-weight: 700;
		letter-spacing: 0;
		text-transform: uppercase;
		color: var(--hero-miniflux-eyebrow);
		margin-bottom: 10px;
	}
	.hero-eyebrow svg {
		width: 12px;
		height: 12px;
		stroke: currentColor;
		fill: none;
		stroke-width: 2.2;
	}
	.eyebrow-sep {
		color: var(--hero-miniflux-edge);
		font-weight: 400;
	}
	.hero-headline {
		font-size: clamp(26px, 3.2vw, 38px);
		font-weight: 600;
		letter-spacing: 0;
		line-height: 1.06;
		color: var(--hero-miniflux-headline);
		margin: 0 0 8px;
	}
	.hero-sub {
		font-size: 13.5px;
		color: var(--hero-miniflux-eyebrow);
		line-height: 1.52;
		max-width: 460px;
		margin: 0 0 16px;
	}
	.hero-ops {
		display: flex;
		align-items: center;
		flex-wrap: wrap;
		gap: 12px 16px;
		font-size: 12px;
		color: var(--hero-miniflux-eyebrow);
	}
	.hero-ops > * {
		display: inline-flex;
		align-items: center;
		gap: 6px;
	}
	.hero-ops strong {
		color: var(--hero-miniflux-headline);
		font-weight: 600;
	}
	.ops-sep {
		width: 1px;
		height: 11px;
		background: var(--hero-miniflux-edge);
	}

	/* Pills */
	.hero-status-pill {
		padding: 3px 11px;
		border-radius: 980px;
		font-size: 11.5px;
		font-weight: 600;
		background: var(--bg-primary);
		color: var(--text-primary);
	}
	.pulse {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: currentColor;
	}
	.hero-status-pill[data-status='connected'] {
		background: rgba(52, 199, 89, 0.15);
		color: var(--success);
	}
	.hero-status-pill[data-status='syncing'] {
		background: rgba(0, 113, 227, 0.15);
		color: var(--accent);
	}
	.hero-status-pill[data-status='error'] {
		background: rgba(255, 59, 48, 0.15);
		color: var(--destructive);
	}
	.hero-status-pill[data-status='disconnected'] {
		background: var(--fill-secondary);
		color: var(--text-secondary);
	}

	/* Form Elements */
	.connect-form {
		display: flex;
		flex-direction: column;
		gap: 16px;
		max-width: 400px;
	}
	.field {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.field label {
		font-family: var(--font-sans);
		font-size: 14px;
		font-weight: 500;
		color: var(--text-primary);
	}
	.input {
		padding: 10px 12px;
		border: 1px solid var(--input-border);
		border-radius: 8px;
		background: var(--input-bg);
		color: var(--text-primary);
		font-family: var(--font-sans);
		font-size: 15px;
	}
	.btn {
		padding: 7px 12px;
		border: 1px solid var(--border-hairline);
		background: var(--bg-elevated);
		color: var(--text-primary);
		border-radius: 8px;
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
	}
	.btn-primary {
		background: var(--accent);
		color: var(--text-on-color);
		border-color: transparent;
	}
	.btn-primary:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}
	.error {
		color: var(--destructive);
		font-family: var(--font-sans);
		font-size: 14px;
		margin: 0;
	}
	.muted {
		color: var(--text-tertiary);
		font-size: 14px;
	}

	/* Cards */
	.card {
		background: var(--bg-elevated);
		border: 1px solid var(--border-hairline);
		border-radius: 14px;
		box-shadow: var(--shadow-1);
		overflow: hidden;
	}
	.card-stack > * + * {
		border-top: 1px solid var(--border-hairline);
	}
	.row {
		display: grid;
		grid-template-columns: 1fr auto;
		gap: 24px;
		align-items: center;
		padding: 14px 22px;
	}
	.row-ops {
		display: flex;
		gap: 8px;
		align-items: center;
	}
	.row-title {
		font-size: 14px;
		font-weight: 500;
		margin: 0;
		color: var(--text-primary);
	}
	.row-sub {
		font-size: 12.5px;
		color: var(--text-tertiary);
		margin: 4px 0 0;
		line-height: 1.45;
		max-width: 64ch;
	}

	/* Alerts */
	.alert-block {
		padding: 16px 22px 18px;
	}
	.alert {
		padding: 14px;
		border-radius: 12px;
		background: rgba(255, 59, 48, 0.1);
		border: 1px solid rgba(255, 59, 48, 0.2);
		color: var(--text-secondary);
	}
	.alert strong {
		color: var(--text-primary);
	}
	.alert p {
		margin: 4px 0 10px;
		font-size: 13px;
	}
	.alert span {
		font-family: var(--font-mono, ui-monospace, monospace);
		font-size: 11.5px;
		color: var(--destructive);
		word-break: break-word;
	}

	@media (max-width: 599px) {
		.settings-body {
			padding: 20px 16px 56px;
		}
	}
	@container hero (max-width: 679px) {
		.hero-gem {
			display: none;
		}
		.hero-watermark {
			right: -40px;
		}
	}
</style>
