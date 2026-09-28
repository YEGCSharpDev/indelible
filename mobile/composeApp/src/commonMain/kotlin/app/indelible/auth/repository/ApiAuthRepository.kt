package app.indelible.auth.repository

import app.indelible.api.generated.models.OAuthProvidersResponse
import app.indelible.core.model.AuthResponse
import app.indelible.core.model.AuthUser
import app.indelible.core.network.AccountApiService
import app.indelible.core.network.AuthApiService
import app.indelible.core.network.NativeOAuthTokenResponse

class ApiAuthRepository(
    private val authApiService: AuthApiService,
    private val accountApiService: AccountApiService,
) : AuthRepository {
    override suspend fun login(
        username: String,
        password: String,
    ): Result<AuthResponse> = authApiService.login(username, password)

    override suspend fun register(
        name: String,
        username: String,
        password: String,
    ): Result<AuthResponse> = authApiService.register(name, username, password)


    override suspend fun logout(): Result<Unit> = authApiService.logout()

    override suspend fun getSession(): Result<AuthUser> = accountApiService.getSession()


    override suspend fun getOAuthProviders(): Result<OAuthProvidersResponse> = authApiService.getOAuthProviders()

    override suspend fun nativeOAuthStartUrl(
        providerId: String,
        codeChallenge: String,
        appState: String,
    ): String = authApiService.nativeOAuthStartUrl(providerId, codeChallenge, appState)

    override suspend fun exchangeNativeOAuthCode(
        code: String,
        codeVerifier: String,
    ): Result<NativeOAuthTokenResponse> = authApiService.exchangeNativeOAuthCode(code, codeVerifier)

    override suspend fun updateProfile(
        displayName: String?,
        theme: String?,
    ): Result<AuthUser> = accountApiService.updateProfile(displayName, theme)

    override suspend fun fetchAvatarBytes(avatarUrl: String): Result<ByteArray> = accountApiService.fetchAvatarBytes(avatarUrl)
}
