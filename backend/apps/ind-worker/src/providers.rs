use std::sync::Arc;

use secrecy::ExposeSecret;

use crate::config::WorkerConfig;

pub fn build_credential_cipher(config: &WorkerConfig) -> Option<Arc<ind_auth::CredentialCipher>> {
    let key = config.auth.credential_key.as_ref()?;
    match ind_auth::CredentialCipher::from_base64(key.expose_secret()) {
        Ok(cipher) => Some(Arc::new(cipher)),
        Err(e) => {
            tracing::warn!(
                error = %e,
                "auth.credential_key is set but invalid; integration OAuth token decryption disabled"
            );
            None
        }
    }
}
