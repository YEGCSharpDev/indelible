package app.indelible.auth.viewmodel

import app.indelible.core.i18n.UiMessage
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.auth_password_required
import indelible.composeapp.generated.resources.auth_username_required

data class LoginState(
    val username: String = "",
    val password: String = "",
    val usernameError: UiMessage? = null,
    val passwordError: UiMessage? = null,
    val serverError: UiMessage? = null,
    val isLoading: Boolean = false,
) {
    fun validate(): LoginState {
        val usernameErr = if (username.isBlank()) UiMessage(Res.string.auth_username_required) else null
        val passwordErr = if (password.isBlank()) UiMessage(Res.string.auth_password_required) else null
        return copy(usernameError = usernameErr, passwordError = passwordErr, serverError = null)
    }

    val isValid: Boolean
        get() = username.isNotBlank() && password.isNotBlank()
}
