use chrono::{Duration, Utc};
pub use ind_application::ports::{RegisterRequest, RegisterResponse};
use ind_domain::{
    ClientType, RefreshToken, RefreshTokenId, User, UserId, UserStatus, validate_password,
};

use crate::crypto::{generate_refresh_token, hash_password, hash_token};
use crate::error::AuthError;
use crate::jwt;
use crate::service::AuthService;

const IDLE_TIMEOUT_DAYS: i64 = 30;
const ABSOLUTE_LIFETIME_DAYS: i64 = 90;

impl AuthService {
    pub async fn register(
        &self,
        req: RegisterRequest,
        client_type: ClientType,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> Result<RegisterResponse, AuthError> {
        let username = req.username.clone();

        if validate_password(&req.password).is_err() {
            return Err(AuthError::PasswordTooWeak);
        }

        if !self.allow_signups && self.user_repo.has_any_users().await? {
            return Err(AuthError::SignupsDisabled);
        }

        if self.user_repo.find_by_username(&username).await?.is_some() {
            return Err(AuthError::EmailAlreadyExists);
        }

        let password_clone = req.password.clone();
        let password_hash = tokio::task::spawn_blocking(move || hash_password(&password_clone))
            .await
            .map_err(|_| AuthError::HashError("hashing task failed".into()))??;
        let now = Utc::now();

        let user = User {
            id: UserId::new(),
            username,
            password_hash: Some(password_hash),
            display_name: req.display_name,
            avatar_url: None,
            locale: None,
            timezone: "UTC".to_string(),
            theme: Default::default(),
            onboarding_completed: false,
            onboarding_step: 0,
            status: UserStatus::Active,
            created_at: now,
            updated_at: now,
        };

        let user = if self.allow_signups {
            self.user_repo.create(user).await?
        } else {
            self.user_repo
                .create_first_user(user)
                .await?
                .ok_or(AuthError::SignupsDisabled)?
        };

        let family_id = uuid::Uuid::now_v7();
        let raw_refresh = generate_refresh_token();
        let token_hash = hash_token(&raw_refresh);

        let refresh_token = RefreshToken {
            id: RefreshTokenId::new(),
            family_id,
            user_id: user.id,
            token_hash,
            client_type,
            ip_address: ip,
            user_agent,
            replaced_by: None,
            revoked_at: None,
            expires_at: now + Duration::days(IDLE_TIMEOUT_DAYS),
            absolute_expires_at: now + Duration::days(ABSOLUTE_LIFETIME_DAYS),
            last_used_at: now,
            created_at: now,
        };
        let refresh_token = self.refresh_token_repo.create(refresh_token).await?;

        let (access_token, expires_at) =
            jwt::sign_access_token(user.id, client_type, &self.jwt_secret)?;

        Ok(RegisterResponse {
            user,
            access_token,
            expires_at,
            raw_refresh_token: raw_refresh,
            refresh_token,
        })
    }
}
