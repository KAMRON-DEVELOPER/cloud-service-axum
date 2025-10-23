use crate::utilities::errors::AppError;
use axum::{
    RequestPartsExt,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::utilities::config::Config;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
    Access,
    Refresh,
    EmailVerification,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Claims {
    pub sub: Uuid,
    pub typ: TokenType,
    pub exp: i64,
    pub iat: i64,
}

pub fn create_token(config: &Config, user_id: Uuid, typ: TokenType) -> Result<String, AppError> {
    let now = Utc::now();

    let exp = now
        + match typ {
            TokenType::Access => Duration::minutes(config.access_token_expire_in_minute.unwrap()),
            TokenType::Refresh => Duration::days(config.refresh_token_expire_in_days.unwrap()),
            TokenType::EmailVerification => {
                Duration::hours(config.email_verification_token_expire_in_hours.unwrap())
            }
        };

    let claims = Claims {
        sub: user_id,
        typ,
        iat: now.timestamp(),
        exp: exp.timestamp(),
    };

    let encoding_key = EncodingKey::from_secret(config.secret_key.as_ref().unwrap().as_bytes());
    let encoded_token = encode(&Header::new(Algorithm::HS256), &claims, &encoding_key)?;
    Ok(encoded_token)
}

pub fn verify_token(config: &Config, token: &str) -> Result<Claims, AppError> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.secret_key.as_ref().unwrap().as_bytes()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}

impl<S> FromRequestParts<S> for Claims
where
    Config: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AppError::MissingAccessToken)?;

        let config = Config::from_ref(state);

        let claims = verify_token(&config, bearer.token())?;

        if claims.typ != TokenType::Access {
            return Err(AppError::Unauthorized("Access token required".into()));
        }

        Ok(claims)
    }
}
