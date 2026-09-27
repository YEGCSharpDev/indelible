import type { MessageKey } from '$lib/i18n';

export type LibraryFilterFieldDef = {
	key: string;
	labelKey: MessageKey;
	section: 'content' | 'attributes' | 'dates';
	ops: { value: string; labelKey: MessageKey }[];
	valueType: 'select' | 'multi-select' | 'text' | 'number' | 'date' | 'boolean';
	options?: { value: string; labelKey: MessageKey }[];
	booleanLabelKeys?: { true: MessageKey; false: MessageKey };
};

const LIBRARY_FILTER_FIELDS: LibraryFilterFieldDef[] = [
	{
		key: 'tag',
		labelKey: 'library_filter_field_tag',
		section: 'content',
		ops: [
			{ value: 'contains', labelKey: 'library_filter_operator_contains' },
			{ value: 'eq', labelKey: 'library_filter_operator_is_exactly' },
			{ value: 'neq', labelKey: 'library_filter_operator_is_not' }
		],
		valueType: 'text'
	},
	{
		key: 'item_type',
		labelKey: 'library_filter_field_content_type',
		section: 'content',
		ops: [
			{ value: 'eq', labelKey: 'library_filter_operator_is' },
			{ value: 'neq', labelKey: 'library_filter_operator_is_not' },
			{ value: 'in', labelKey: 'library_filter_operator_is_any_of' }
		],
		valueType: 'select',
		options: [
			{ value: 'article', labelKey: 'library_filter_value_article' },
			{ value: 'book', labelKey: 'library_filter_value_book' },
			{ value: 'pdf', labelKey: 'library_filter_value_pdf' }
		]
	},
	{
		key: 'domain',
		labelKey: 'library_filter_field_domain',
		section: 'content',
		ops: [
			{ value: 'eq', labelKey: 'library_filter_operator_is' },
			{ value: 'neq', labelKey: 'library_filter_operator_is_not' },
			{ value: 'in', labelKey: 'library_filter_operator_is_any_of' }
		],
		valueType: 'text'
	},
	{
		key: 'collection',
		labelKey: 'library_filter_field_collection',
		section: 'content',
		ops: [
			{ value: 'eq', labelKey: 'library_filter_operator_is' },
			{ value: 'contains', labelKey: 'library_filter_operator_contains' }
		],
		valueType: 'text'
	},
	{
		key: 'is_favorite',
		labelKey: 'library_filter_field_favorited',
		section: 'attributes',
		ops: [],
		valueType: 'boolean',
		booleanLabelKeys: {
			true: 'library_filter_favorited_true',
			false: 'library_filter_favorited_false'
		}
	},
	{
		key: 'triage_state',
		labelKey: 'library_filter_field_read_status',
		section: 'attributes',
		ops: [
			{ value: 'eq', labelKey: 'library_filter_operator_is' },
			{ value: 'neq', labelKey: 'library_filter_operator_is_not' },
			{ value: 'in', labelKey: 'library_filter_operator_is_any_of' }
		],
		valueType: 'select',
		options: [
			{ value: 'inbox', labelKey: 'library_filter_value_inbox' },
			{ value: 'later', labelKey: 'library_filter_value_later' },
			{ value: 'archive', labelKey: 'library_filter_value_archive' }
		]
	},
	{
		key: 'saved_at',
		labelKey: 'library_filter_field_saved_date',
		section: 'dates',
		ops: [
			{ value: 'gt', labelKey: 'library_filter_operator_after' },
			{ value: 'lt', labelKey: 'library_filter_operator_before' },
			{ value: 'eq', labelKey: 'library_filter_operator_on' },
			{ value: 'gte', labelKey: 'library_filter_operator_on_or_after' },
			{ value: 'lte', labelKey: 'library_filter_operator_on_or_before' }
		],
		valueType: 'date'
	},
	{
		key: 'published_at',
		labelKey: 'library_filter_field_published_date',
		section: 'dates',
		ops: [
			{ value: 'gt', labelKey: 'library_filter_operator_after' },
			{ value: 'lt', labelKey: 'library_filter_operator_before' },
			{ value: 'eq', labelKey: 'library_filter_operator_on' },
			{ value: 'gte', labelKey: 'library_filter_operator_on_or_after' },
			{ value: 'lte', labelKey: 'library_filter_operator_on_or_before' }
		],
		valueType: 'date'
	}
];

export function getLibraryFilterFieldDef(key: string): LibraryFilterFieldDef {
	return LIBRARY_FILTER_FIELDS.find((field) => field.key === key) ?? LIBRARY_FILTER_FIELDS[0]!;
}

export function getVisibleLibraryFilterFields(
	_activeType?: string | null
): LibraryFilterFieldDef[] {
	void _activeType;
	return LIBRARY_FILTER_FIELDS;
}
