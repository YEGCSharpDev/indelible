package app.indelible.auth.viewmodel

import app.indelible.core.i18n.UiMessage
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.auth_password_required
import indelible.composeapp.generated.resources.auth_username_required
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

class LoginStateTest {
    @Test
    fun validateReturnsUsernameError() {
        val state = LoginState(username = "", password = "validpassword")
        val validated = state.validate()
        assertNotNull(validated.usernameError)
        assertEquals(UiMessage(Res.string.auth_username_required), validated.usernameError)
        assertNull(validated.passwordError)
    }

    @Test
    fun validateReturnsPasswordError() {
        val state = LoginState(username = "user123", password = "")
        val validated = state.validate()
        assertNull(validated.usernameError)
        assertNotNull(validated.passwordError)
        assertEquals(UiMessage(Res.string.auth_password_required), validated.passwordError)
    }

    @Test
    fun validateReturnsBothErrors() {
        val state = LoginState(username = "", password = "")
        val validated = state.validate()
        assertNotNull(validated.usernameError)
        assertNotNull(validated.passwordError)
    }

    @Test
    fun isValidReturnsTrueForValidInput() {
        val state = LoginState(username = "user123", password = "password123")
        assertTrue(state.isValid)
    }

    @Test
    fun isValidReturnsFalseForEmptyUsername() {
        val state = LoginState(username = "", password = "password123")
        assertFalse(state.isValid)
    }

    @Test
    fun isValidReturnsFalseForEmptyPassword() {
        val state = LoginState(username = "user123", password = "")
        assertFalse(state.isValid)
    }

    @Test
    fun validateClearsServerError() {
        val state =
            LoginState(
                username = "user123",
                password = "password123",
                serverError = UiMessage(Res.string.auth_password_required),
            )
        val validated = state.validate()
        assertNull(validated.serverError)
    }
}
