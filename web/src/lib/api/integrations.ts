import {
	authorizeIntegration,
	connectMiniflux,
	deleteIntegration,
	listIntegrations,
	syncIntegration,
	type AuthorizeIntegrationResponse,
	type IntegrationConnectionDto,
	type IntegrationListResponse,
	type SyncIntegrationResponse
} from '$lib/api';

type ApiProblem = {
	detail?: string;
	error?: string;
	message?: string;
};

export type ApiResult<T> = { success: true; data: T } | { success: false; error: string };

function extractMessage(problem: unknown, fallback: string): string {
	if (!problem || typeof problem !== 'object') {
		return fallback;
	}
	const candidate = problem as ApiProblem;
	return candidate.detail ?? candidate.message ?? candidate.error ?? fallback;
}

function failure<T>(err: unknown, action: string): ApiResult<T> {
	console.error(`[integrations api] ${action}`, err);
	if (err && typeof err === 'object' && 'message' in err && typeof err.message === 'string') {
		return { success: false, error: err.message };
	}
	return { success: false, error: `An unexpected error occurred while ${action}.` };
}

export async function loadIntegrationConnections(): Promise<ApiResult<IntegrationListResponse>> {
	try {
		const { data, error } = await listIntegrations();
		if (data) {
			return { success: true, data };
		}
		return { success: false, error: extractMessage(error, 'Failed to load integrations') };
	} catch (err) {
		return failure(err, 'loading integrations');
	}
}

export async function startIntegrationAuthorization(
	provider: string,
	redirectAfter?: string
): Promise<ApiResult<AuthorizeIntegrationResponse>> {
	try {
		const { data, error } = await authorizeIntegration({
			path: { provider },
			body: { redirect_after: redirectAfter ?? null }
		});
		if (data) {
			return { success: true, data };
		}
		return {
			success: false,
			error: extractMessage(error, `Failed to start ${provider} authorization`)
		};
	} catch (err) {
		return failure(err, `starting ${provider} authorization`);
	}
}

export async function dispatchIntegrationSync(
	connectionId: string
): Promise<ApiResult<SyncIntegrationResponse>> {
	try {
		const { data, error } = await syncIntegration({ path: { id: connectionId } });
		if (data) {
			return { success: true, data };
		}
		return { success: false, error: extractMessage(error, 'Failed to start sync') };
	} catch (err) {
		return failure(err, 'starting integration sync');
	}
}

export async function disconnectIntegration(connectionId: string): Promise<ApiResult<void>> {
	try {
		const { error, response } = await deleteIntegration({ path: { id: connectionId } });
		if (response?.ok) {
			return { success: true, data: undefined };
		}
		return { success: false, error: extractMessage(error, 'Failed to disconnect integration') };
	} catch (err) {
		return failure(err, 'disconnecting integration');
	}
}

export async function setupMinifluxConnection(
	url: string,
	apiKey: string
): Promise<ApiResult<IntegrationConnectionDto>> {
	try {
		const { data, error } = await connectMiniflux({
			body: { url, api_key: apiKey }
		});
		if (data) {
			return { success: true, data };
		}
		return { success: false, error: extractMessage(error, 'Failed to connect Miniflux') };
	} catch (err) {
		return failure(err, 'connecting Miniflux');
	}
}
