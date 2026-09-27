use chrono::{DateTime, Utc};
use ind_auth::UserProfile;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Multipart body for POST /api/v1/me/avatar. The `file` part must be
/// image/jpeg, image/png, or image/webp, at most 2 MiB.
#[derive(Debug, ToSchema)]
pub struct AvatarUploadSchema {
    #[expect(
        dead_code,
        reason = "schema-only field; utoipa derives the multipart spec from it"
    )]
    #[schema(value_type = String, format = Binary)]
    pub file: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProfileResponse {
    pub id: String,
    pub object: &'static str,
    pub username: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[schema(value_type = Option<String>, required = true, nullable)]
    pub locale: Option<String>,
    pub timezone: String,
    pub theme: String,
    pub onboarding_completed: bool,
    pub has_password: bool,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct UpdateProfileRequest {
    #[validate(custom(function = "crate::validation::optional_trimmed_non_blank"))]
    #[validate(custom(function = "crate::validation::optional_trimmed_max_display_name_length"))]
    pub display_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable")]
    #[schema(value_type = Option<String>, nullable)]
    #[validate(custom(function = "crate::validation::optional_avatar_reference"))]
    pub avatar_url: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable")]
    #[schema(value_type = Option<String>, nullable)]
    #[validate(custom(function = "crate::validation::optional_locale"))]
    pub locale: Option<Option<String>>,
    #[validate(custom(function = "crate::validation::optional_timezone"))]
    pub timezone: Option<String>,
    #[validate(custom(function = "crate::validation::optional_theme"))]
    pub theme: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    #[validate(custom(function = "crate::validation::password_length"))]
    pub new_password: String,
}

#[derive(Debug, Deserialize, ToSchema, validator::Validate)]
pub struct DeleteAccountRequest {
    #[validate(length(min = 1, message = "must not be empty"))]
    pub confirmation: String,
}

pub fn deserialize_optional_nullable<'de, D>(
    deserializer: D,
) -> Result<Option<Option<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Visitor;

    struct NullableStringVisitor;

    impl<'de> Visitor<'de> for NullableStringVisitor {
        type Value = Option<Option<String>>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("null or a string")
        }

        fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(Some(None))
        }

        fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(Some(None))
        }

        fn visit_some<D2: serde::Deserializer<'de>>(
            self,
            deserializer: D2,
        ) -> Result<Self::Value, D2::Error> {
            Ok(Some(Some(String::deserialize(deserializer)?)))
        }
    }

    deserializer.deserialize_option(NullableStringVisitor)
}

impl ProfileResponse {
    pub fn from_user_profile(profile: UserProfile, theme: String) -> Self {
        Self {
            id: profile.id.to_string(),
            object: "user",
            username: profile.username,
            display_name: profile.display_name,
            avatar_url: profile.avatar_url,
            locale: profile.locale,
            timezone: profile.timezone,
            theme,
            onboarding_completed: profile.onboarding_completed,
            has_password: profile.has_password,
            created_at: profile.created_at,
            updated_at: profile.updated_at,
        }
    }
}
