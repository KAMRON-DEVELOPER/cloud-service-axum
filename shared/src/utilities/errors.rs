use std::{alloc, string::FromUtf8Error};

use axum::{Json, http::StatusCode, response::IntoResponse, response::Response};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    JwtError(String),
    #[error("Database url not set error")]
    DatabaseUrlNotSetError,
    #[error("Database url parsing error")]
    DatabaseParsingError,
    #[error("Database connection error")]
    DatabaseConnectionError,
    #[error("Failed to fetch {resource} with ID {id}")]
    DatabaseFetchError { resource: String, id: String },
    #[error("Failed to delete {resource} with ID {id}")]
    DatabaseDeleteError { resource: String, id: String },
    #[error("Sqlx error: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("Query error")]
    QueryError(String),
    #[error("Redis url not set error")]
    RedisUrlNotSetError,
    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),
    #[error("Missing qdrant url error")]
    MissingQdrantUrlError,
    #[error("Missing amqp url error")]
    MissingAmqpUrlError,
    #[error("Missing qdrant api key error")]
    MissingQdrantApiKeyError,
    #[error("Qdrant error: {0}")]
    QdrantError(#[from] qdrant_client::QdrantError),
    #[error("Vector search error: {0}")]
    VectorSearchError(String),
    #[error("ImageEmbedding creation error")]
    ImageEmbeddingCreationError,
    #[error("TextEmbedding creation error")]
    TextEmbeddingCreationError,
    #[error("Embedding error")]
    EmbeddingError,
    #[error("Bcrypt error: {0}")]
    BcryptError(#[from] bcrypt::BcryptError),
    #[error("Object storage error: {0}")]
    ObjectStorageError(#[from] object_store::Error),
    #[error("HTTP request error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("You're not authorized!")]
    UnauthorizedError,
    #[error("Invalid uuid format: {0}")]
    UuidParseError(#[from] uuid::Error),
    #[error("Url parse error: {0}")]
    UrlParseError(#[from] url::ParseError),
    #[error("Invalid uri error: {0}")]
    InvalidUriError(#[from] axum::http::uri::InvalidUri),
    #[error("Openidconnect discovery error: {0}")]
    OpenIdConnectDiscoveryError(
        #[from] openidconnect::DiscoveryError<oauth2::HttpClientError<reqwest::Error>>,
    ),
    #[error("Openidconnect configuration error: {0}")]
    OpenIdConnectConfigurationError(#[from] openidconnect::ConfigurationError),
    #[error("Attempted to get a non-none value but found none")]
    OptionError,
    #[error("Attempted to parse a number to an integer but errored out: {0}")]
    ParseIntError(#[from] std::num::TryFromIntError),
    #[error("Encountered an error trying to convert an infallible value: {0}")]
    FromRequestPartsError(#[from] std::convert::Infallible),
    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader { expected: String, found: String },
    #[error("Wrong credentials")]
    WrongCredentials,
    #[error("Internal error, {0}")]
    InternalError(String),
    #[error("External service error, {0}")]
    ExternalServiceError(String),
    #[error("Missing email service api key error")]
    MissingEmailServiceApiKeyError,
    #[error("Missing credentials")]
    MissingCredentials,
    #[error("Missing tls ca error")]
    MissingTlsCaError,
    #[error("Missing tls key error")]
    MissingTlsKeyError,
    #[error("Missing tls cert error")]
    MissingTlsCertError,
    #[error("Tonic error")]
    TonicError(#[from] tonic::transport::Error),
    #[error("Token creation error")]
    TokenCreationError,
    #[error("Invalid token error")]
    InvalidTokenError,
    #[error("Missing authorization token error")]
    MissingAuthorizationToken,
    #[error("Missing acces token error")]
    MissingAccessToken,
    #[error("Missing refresh token error")]
    MissingRefreshToken,
    #[error("{0} token required")]
    Unauthorized(String),
    #[error("Missing oauth id error")]
    MissingOAuthIdError,
    #[error("Missing google oauth sub error")]
    MissingGoogleOAuthSubError,
    #[error("Missing github oauth id error")]
    MissingGithubOAuthIdError,
    #[error("Invalid authorization token error")]
    InvalidAuthorizationTokenError,
    #[error("jsonwebtoken error")]
    JsonWebTokenError(#[from] jsonwebtoken::errors::Error),
    #[error("Missing session token token error")]
    MissingSessionTokenError,
    #[error("Invalid session token error")]
    InvalidSessionTokenError,
    #[error("Session not found error")]
    SessionNotFoundError,
    #[error("Expired session token error")]
    ExpiredSessionTokenError,
    #[error("OAuth user not found error")]
    OAuthUserNotFoundError,
    #[error("OAuth user id expired error")]
    OAuthUserIdExpiredError,
    #[error("Json validation error")]
    JsonValidationError,
    #[error("Invalid form data, {0}")]
    InvalidFormData(String),
    #[error("Missing pkce code verifier error")]
    MissingPkceCodeVerifierError,
    #[error("Nonce not found error")]
    NonceNotFoundError,
    #[error("Id token not found error")]
    IdTokenNotFoundError,
    #[error("Openidconnect claims verification error, {0}")]
    OpenIdConnectClaimsVerificationError(#[from] openidconnect::ClaimsVerificationError),
    #[error("Openidconnect signing error, {0}")]
    OpenIdConnectSigningError(#[from] openidconnect::SigningError),
    #[error("Openidconnect signature verification error, {0}")]
    OpenIdConnectSignatureVerificationError(#[from] openidconnect::SignatureVerificationError),
    #[error("Openidconnect http client error, {0}")]
    OpenIdConnectHttpClientError(#[from] openidconnect::HttpClientError<reqwest::Error>),
    #[error("Openidconnect user info error, {0}")]
    OpenIdConnectUserInfoError(
        #[from] openidconnect::UserInfoError<openidconnect::HttpClientError<reqwest::Error>>,
    ),
    #[error("Validation error, {0}")]
    ValidationError(String),
    #[error("Validation error, {0}")]
    ValidatorValidationError(#[from] validator::ValidationError),
    #[error("Validation errors, {0}")]
    ValidatorValidationErrors(#[from] validator::ValidationErrors),
    #[error("Oauth request token error, {0}")]
    RequestTokenError(
        #[from]
        oauth2::RequestTokenError<
            oauth2::HttpClientError<reqwest::Error>,
            oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
        >,
    ),
    #[error("{0}")]
    NotFoundError(String),
    #[error("IO error, {0}")]
    IoError(#[from] std::io::Error),
    #[error("Invalid ca cert error")]
    InvalidCaCertError(String),
    #[error("Incompatible ca cert type error")]
    IncompatibleCaCertTypeError(String),
    #[error("Invalid client cert error")]
    InvalidClientCertError(String),
    #[error("Incompatible client cert type error")]
    IncompatibleClientCertTypeError(String),
    #[error("Invalid client key error")]
    InvalidClientKeyError(String),
    #[error("Incompatible client key type error")]
    IncompatibleClientKeyTypeError(String),
    #[error("Invalid PEM error")]
    InvalidPemError(#[from] rustls::pki_types::pem::Error),
    #[error("Rustls error")]
    RustlsError(#[from] rustls::Error),
    #[error("Invalid image format error")]
    InvalidImageFormatError(String),
    #[error("Serde json error")]
    SerdejsonError(#[from] serde_json::Error),
    #[error("Kube error")]
    KubeError(#[from] kube::Error),
    #[error("Kafka error, {0}")]
    KafkaError(#[from] rdkafka::error::KafkaError),
    #[error("Lapin error, {0}")]
    LapinError(#[from] lapin::Error),
    #[error("Invalid key error, {0}")]
    InvalidKey(String),
    #[error("Encryption failed error, {0}")]
    EncryptionFailed(String),
    #[error("Decryption failed error, {0}")]
    DecryptionFailed(#[from] FromUtf8Error),
    #[error("Invalid format error")]
    InvalidFormat,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Self::IoError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::SerdejsonError(e) => (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()),
            Self::InvalidPemError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::RustlsError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::InvalidCaCertError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Invalid ca cert error, {}", e),
            ),
            Self::IncompatibleCaCertTypeError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!(" Incompatible ca cert type error, {}", e),
            ),
            Self::InvalidClientCertError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Invalid client cert error, {}", e),
            ),
            Self::IncompatibleClientCertTypeError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!(" Incompatible client cert type error, {}", e),
            ),
            Self::InvalidClientKeyError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Invalid client key error, {}", e),
            ),
            Self::IncompatibleClientKeyTypeError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!(" Incompatible client key type error, {}", e),
            ),
            Self::JwtError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e),
            Self::DatabaseUrlNotSetError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database url not set error".to_string(),
            ),
            Self::DatabaseParsingError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database url parsing error".to_string(),
            ),
            Self::DatabaseConnectionError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database connection error".to_string(),
            ),
            Self::DatabaseFetchError { resource, id } => (
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("Failed to fetch {resource} with ID {id}"),
            ),
            Self::DatabaseDeleteError { resource, id } => (
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("Failed to delete {resource} with ID {id}"),
            ),
            Self::SqlxError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::QueryError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e),
            Self::RedisUrlNotSetError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Redis url not set error".to_string(),
            ),
            Self::RedisError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::MissingQdrantUrlError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Missing qdrant url error".to_string(),
            ),
            Self::MissingAmqpUrlError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Missing amqp url error".to_string(),
            ),
            Self::MissingQdrantApiKeyError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Missing qdrant api key error".to_string(),
            ),
            Self::QdrantError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::VectorSearchError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Vector search error, {}", e),
            ),
            Self::ImageEmbeddingCreationError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "ImageEmbedding creation error".to_string(),
            ),
            Self::TextEmbeddingCreationError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "TextEmbedding creation error".to_string(),
            ),
            Self::EmbeddingError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Embedding error".to_string(),
            ),
            Self::BcryptError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::ObjectStorageError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::Request(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::UnauthorizedError => (StatusCode::UNAUTHORIZED, "Unauthorized!".to_string()),
            Self::UuidParseError(e) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("Invalid uuid format, {}", e),
            ),
            Self::UrlParseError(e) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("Url parse error, {}", e),
            ),
            Self::InvalidUriError(e) => (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()),
            Self::OpenIdConnectDiscoveryError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Openidconnect discovery error, {}", e),
            ),
            Self::OpenIdConnectClaimsVerificationError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Openidconnect claims verification error, {}", e),
            ),
            Self::OpenIdConnectSigningError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Openidconnect signing error, {}", e),
            ),
            Self::OpenIdConnectConfigurationError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Openidconnect configuration error, {}", e),
            ),
            Self::OpenIdConnectSignatureVerificationError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Openidconnect signature verification error, {}", e),
            ),
            Self::OpenIdConnectHttpClientError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Openidconnect http client error, {}", e),
            ),
            Self::OpenIdConnectUserInfoError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Openidconnect client info error, {}", e),
            ),
            Self::OptionError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Attempted to get a non-none value but found none".to_string(),
            ),
            Self::ParseIntError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::FromRequestPartsError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::InvalidHeader { expected, found } => (
                StatusCode::BAD_REQUEST,
                format!("invalid header (expected {expected:?}, found {found:?})"),
            ),
            AppError::WrongCredentials => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Wrong credentials".to_string(),
            ),
            AppError::MissingCredentials => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Missing credentials".to_string(),
            ),
            AppError::MissingTlsCaError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Missing tls ca error".to_string(),
            ),
            AppError::MissingTlsKeyError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Missing tls key error".to_string(),
            ),
            AppError::MissingTlsCertError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Missing tls cert error".to_string(),
            ),
            AppError::TonicError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::InternalError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e),
            AppError::ExternalServiceError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e),
            AppError::MissingEmailServiceApiKeyError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Missing email service api key".to_string(),
            ),
            AppError::TokenCreationError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Token creation error".to_string(),
            ),
            AppError::InvalidTokenError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Invalid token".to_string(),
            ),
            Self::MissingAuthorizationToken => (
                StatusCode::UNAUTHORIZED,
                "Missing authorization token".to_string(),
            ),
            Self::MissingAccessToken => {
                (StatusCode::UNAUTHORIZED, "Missing access token".to_string())
            }
            Self::MissingRefreshToken => (
                StatusCode::UNAUTHORIZED,
                "Missing refresh token".to_string(),
            ),
            Self::Unauthorized(e) => (StatusCode::UNAUTHORIZED, e),
            Self::MissingOAuthIdError => (
                StatusCode::UNAUTHORIZED,
                "Missing oauth id error".to_string(),
            ),
            Self::MissingGoogleOAuthSubError => (
                StatusCode::UNAUTHORIZED,
                "Missing google oauth sub error".to_string(),
            ),
            Self::MissingGithubOAuthIdError => (
                StatusCode::UNAUTHORIZED,
                "Missing github oauth id error".to_string(),
            ),
            Self::MissingSessionTokenError => (
                StatusCode::UNAUTHORIZED,
                "Missing session token".to_string(),
            ),
            Self::InvalidSessionTokenError => (
                StatusCode::UNAUTHORIZED,
                "Invalid session token".to_string(),
            ),
            Self::SessionNotFoundError => {
                (StatusCode::UNAUTHORIZED, "Session not found".to_string())
            }
            Self::ExpiredSessionTokenError => (
                StatusCode::UNAUTHORIZED,
                "Expired session token".to_string(),
            ),
            Self::OAuthUserNotFoundError => (
                StatusCode::UNAUTHORIZED,
                "OAuth user not found error".to_string(),
            ),
            Self::OAuthUserIdExpiredError => (
                StatusCode::UNAUTHORIZED,
                "OAuth user id expired error".to_string(),
            ),
            Self::InvalidAuthorizationTokenError => (
                StatusCode::UNAUTHORIZED,
                "Invalid authorization token".to_string(),
            ),
            Self::JsonWebTokenError(e) => (StatusCode::UNAUTHORIZED, e.to_string()),
            Self::JsonValidationError => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "Json validation error".to_string(),
            ),
            Self::InvalidFormData(e) => (StatusCode::UNPROCESSABLE_ENTITY, e),
            Self::MissingPkceCodeVerifierError => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "Missing pkce code verifier error".to_string(),
            ),
            Self::NonceNotFoundError => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "Nonce not found error".to_string(),
            ),
            Self::IdTokenNotFoundError => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "Id token not found error".to_string(),
            ),
            Self::ValidationError(e) => (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()),
            Self::ValidatorValidationError(e) => (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()),
            Self::ValidatorValidationErrors(e) => (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()),
            Self::RequestTokenError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::NotFoundError(e) => (StatusCode::NOT_FOUND, e),
            Self::InvalidImageFormatError(e) => (StatusCode::UNPROCESSABLE_ENTITY, e),
            Self::KubeError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::KafkaError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::LapinError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::InvalidKey(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::EncryptionFailed(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::DecryptionFailed(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::InvalidFormat => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Invalid format error".to_string(),
            ),
        };

        let body = Json(json!({"error": error_message}));

        (status, body).into_response()
    }
}
