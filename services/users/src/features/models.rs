use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

#[derive(Type, Serialize, Deserialize, PartialEq, Eq, Default, Debug)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    #[default]
    Regular,
}

#[derive(Type, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Debug)]
#[sqlx(type_name = "user_status", rename_all = "lowercase")]
pub enum UserStatus {
    Active,
    Suspended,
    #[default]
    PendingVerification,
}

#[derive(Type, Serialize, Deserialize, PartialEq, Eq, Debug)]
#[sqlx(type_name = "provider", rename_all = "lowercase")]
pub enum Provider {
    Google,
    Github,
    Email,
}

#[derive(FromRow, Serialize, Deserialize, PartialEq, Eq, Default, Debug)]
#[sqlx(default)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: Uuid,
    pub full_name: String,
    pub username: String,
    pub email: String,
    pub password: Option<String>,
    pub phone_number: Option<String>,
    pub picture: Option<String>,
    pub role: UserRole,
    pub status: UserStatus,
    pub email_verified: bool,
    pub oauth_user_id: Option<String>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub last_login_ip: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(FromRow, Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OAuthUser {
    pub id: String,
    pub provider: Provider,
    pub username: Option<String>,
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub phone_number: Option<String>,
    pub password: Option<String>,
    pub picture: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}
