use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::features::models::{User, UserRole, UserStatus};

#[derive(Deserialize, Debug)]
pub struct VerifyQuery {
    pub token: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RedirectResponse {
    pub redirect_to: String,
}

#[derive(Deserialize, Debug)]
pub struct OAuthCallback {
    pub(crate) code: String,
}

#[derive(Serialize, Debug)]
pub struct AuthResponse {
    pub user: User,
    pub tokens: Tokens,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
}

#[derive(Deserialize, Validate, Debug)]
pub struct ContinueWithEmailSchema {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,
    #[validate(length(
        min = 8,
        max = 32,
        message = "Password should be long between 8 and 32"
    ))]
    pub password: String,
}

#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct GoogleOAuthUser {
    pub sub: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub family_name: Option<String>,
    pub given_name: Option<String>,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub phone_number: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct GithubOAuthUser {
    pub id: i64,
    pub login: String,
    pub avatar_url: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize, Default, Debug)]
#[serde(default, rename_all = "camelCase")]
pub struct OAuthUserSchema {
    pub username: Option<String>,
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub phone_number: Option<String>,
    pub password: Option<String>,
    pub picture: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOut {
    pub id: Uuid,
    pub full_name: String,
    pub email: String,
    pub phone_number: String,
    pub picture: Option<String>,
    pub role: UserRole,
    pub status: UserStatus,
    pub email_verified: bool,
    pub oauth_user_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
