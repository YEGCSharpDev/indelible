import type {
	DocumentReaderAssetResponse,
	LibraryEntryResponse,
	SearchEmbeddedSenderResponse
} from './generated/types.gen';
import type { LibraryQueryBody as GeneratedLibraryQueryBody } from './generated/types.gen';

export type DocumentListEntry = Omit<LibraryEntryResponse, 'library_entry_id'> & {
	id: string;
	library_entry_id: string | null;
	item_type: string;
	available_assets?: string[];
	chapter_locator?: string | null;
	chapter_offset?: number | null;
	deleted_at?: string | null;
	last_read_at?: string | null;
	max_progress_percent?: number | null;
	pipeline_error?: string | null;
	pipeline_status?: string | null;
	progress_percent?: number | null;
	readable_ready?: boolean;
	saved?: boolean;
	sender?: SearchEmbeddedSenderResponse | null;
	summary?: string | null;
	video_duration_seconds?: number | null;
};

export type LibraryQueryBody = GeneratedLibraryQueryBody;

export type LibraryTriageRequest = {
	state: string;
};

export type DocumentUpdateBody = Partial<
	Pick<
		LibraryEntryResponse,
		| 'author'
		| 'canonical_url'
		| 'excerpt'
		| 'language'
		| 'lead_image_url'
		| 'published_at'
		| 'thumbnail_url'
		| 'title'
		| 'url'
	>
>;

export type DocumentAssetListResponse = {
	data: DocumentReaderAssetResponse[];
};

export type TestMilaConfigBodyWritable = {
	chat_api_base?: string;
	chat_api_key?: string;
	chat_model?: string;
	embedding_api_base?: string;
	embedding_api_key?: string;
	embedding_dim?: number;
	embedding_model?: string;
};

export type TestConfigResponse = {
	chat_error?: string | null;
	chat_model_ok: boolean;
	embedding_dim?: number | null;
	embedding_error?: string | null;
	embedding_model_ok: boolean;
	error?: string | null;
	success: boolean;
};
