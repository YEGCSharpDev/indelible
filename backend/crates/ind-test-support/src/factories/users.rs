use super::prelude::*;

#[derive(Default)]
pub struct UserFactory {
    username: Option<String>,
    display_name: Option<String>,
}

impl UserFactory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }

    pub async fn insert(self, pool: &sqlx::PgPool) -> User {
        let timestamp = Utc::now();
        let suffix = short_unique_suffix();
        let username = self.username.unwrap_or_else(|| format!("user-{}", suffix));
        let display_name = self
            .display_name
            .unwrap_or_else(|| format!("{} {}", Name().fake::<String>(), suffix));
        PgUserRepository::new(pool.clone())
            .create(User {
                id: UserId::new(),
                username,
                password_hash: None,
                display_name,
                avatar_url: None,
                locale: None,
                timezone: "UTC".into(),
                theme: Theme::System,
                onboarding_completed: false,
                onboarding_step: 0,
                status: UserStatus::Active,
                created_at: timestamp,
                updated_at: timestamp,
            })
            .await
            .expect("UserFactory::insert failed")
    }
}
