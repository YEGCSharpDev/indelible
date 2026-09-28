package app.indelible.auth.repository

import app.indelible.api.generated.models.OAuthProvidersResponse
import app.indelible.core.model.AuthResponse
import app.indelible.core.model.AuthUser
import app.indelible.core.network.NativeOAuthTokenResponse

interface AuthRepository {
    suspend fun login(
        username: String,
        password: String,
    ): Result<AuthResponse>

    suspend fun register(
        name: String,
        username: String,
        password: String,
    ): Result<AuthResponse>


    suspend fun logout(): Result<Unit>

    suspend fun getSession(): Result<AuthUser>


    suspend fun getOAuthProviders(): Result<OAuthProvidersResponse>

    suspend fun nativeOAuthStartUrl(
        providerId: String,
        codeChallenge: String,
        appState: String,
    ): String

    suspend fun exchangeNativeOAuthCode(
        code: String,
        codeVerifier: String,
    ): Result<NativeOAuthTokenResponse>

    suspend fun updateProfile(
        displayName: String?,
        theme: String? = null,
    ): Result<AuthUser>

    suspend fun fetchAvatarBytes(avatarUrl: String): Result<ByteArray>
}
