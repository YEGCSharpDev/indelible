package app.indelible.core.model

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class AuthUser(
    val id: String,
    @SerialName("display_name")
    val displayName: String,
    @SerialName("onboarding_completed")
    val onboardingCompleted: Boolean = false,
    @SerialName("avatar_url")
    val avatarUrl: String? = null,
)
