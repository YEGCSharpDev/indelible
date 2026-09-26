<script lang="ts">
	import type { DocumentListEntry } from '$lib/api';
	import { t } from '$lib/i18n';
	import MorphSwitcher from '$lib/components/ui/MorphSwitcher.svelte';
	import DetailInfo from './DetailInfo.svelte';
	import NotebookTab from './NotebookTab.svelte';
	import EditMetadataPanel from './EditMetadataPanel.svelte';

	interface Props {
		item: DocumentListEntry | null;
		collectionId?: string | null;
		collectionName?: string | null;
	}

	let { item, collectionId = null, collectionName = null }: Props = $props();

	const displayItem = $derived(item);

	type Tab = 'info' | 'notebook';
	type TabOption = { value: Tab; labelKey: 'common_info' | 'common_notebook' };

	let activeTab: Tab = $state('info');
	let editing = $state(false);
	const currentItemId = $derived(item?.id ?? null);
	let trackedItemId = $state<string | null>(null);

	$effect(() => {
		if (currentItemId !== trackedItemId) {
			trackedItemId = currentItemId;
			editing = false;
		}
	});

	const tabOptions: TabOption[] = [
		{ value: 'info', labelKey: 'common_info' },
		{ value: 'notebook', labelKey: 'common_notebook' }
	];

	function onTabChange(value: string) {
		activeTab = value as Tab;
	}
</script>

<aside class="detail-panel">
	<div class="detail-tabs">
		<span class="tabs-eyebrow">{$t('common_details')}</span>
		<MorphSwitcher options={tabOptions} value={activeTab} onchange={onTabChange} size="sm" />
	</div>

	{#if editing && displayItem}
		<EditMetadataPanel
			item={displayItem}
			onClose={() => {
				editing = false;
			}}
		/>
	{:else if activeTab === 'info' && displayItem}
		<DetailInfo
			item={displayItem}
			onEditMetadata={() => {
				editing = true;
			}}
		/>
	{:else if activeTab === 'notebook' && displayItem}
		<NotebookTab item={displayItem} />
	{/if}
</aside>

<style>
	.detail-panel {
		width: 300px;
		min-width: 300px;
		background: var(--vibrancy-sidebar);
		backdrop-filter: blur(60px) saturate(220%);
		-webkit-backdrop-filter: blur(60px) saturate(220%);
		border-left: 0.5px solid var(--border-primary);
		display: flex;
		flex-direction: column;
		overflow-y: auto;
	}

	/* Mirrors the list header grammar: quiet label left, switcher right.
	   Height is overridable so the row's hairline can align with whatever
	   header sits beside it (60px library list, 44px reader toolbar). */
	.detail-tabs {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 10px;
		height: var(--detail-tabs-height, 60px);
		padding: 0 16px;
		border-bottom: 0.5px solid var(--border-primary);
		flex-shrink: 0;
	}

	.tabs-eyebrow {
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--text-tertiary);
		white-space: nowrap;
	}
</style>
