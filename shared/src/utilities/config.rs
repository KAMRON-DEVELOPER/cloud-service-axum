#![allow(unused)]
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use sqlx::postgres::PgSslMode;
use tokio::fs;
use tracing::{Level, warn};

use crate::utilities::errors::AppError;

#[derive(Clone, Debug)]
pub struct Config {
    pub server_addres: String,
    pub frontend_endpoint: String,

    pub base_domain: String,

    // KUBERNETES
    pub k8s_in_cluster: bool,
    pub k8s_config_path: Option<String>,
    pub k8s_encryption_key: String,

    pub base_dir: PathBuf,
    pub tracing_level: Level,

    // DATABASE
    pub pg_ssl_mode: PgSslMode,
    pub database_url: String,

    // REDIS
    pub redis_url: String,
    pub redis_host: String,
    pub redis_port: u16,
    pub redis_username: Option<String>,
    pub redis_password: Option<String>,

    // RABBITMQ
    pub amqp_addr: String,

    // KAFKA BROKERS
    pub kafka_bootstrap_servers: String,

    // GCP
    pub gcs_bucket_name: Option<String>,
    pub gcp_service_account: Option<String>,
    pub gcp_service_account_path: Option<PathBuf>,

    pub google_oauth_client_id: Option<String>,
    pub google_oauth_client_secret: Option<String>,
    pub google_oauth_redirect_url: Option<String>,

    pub github_oauth_client_id: Option<String>,
    pub github_oauth_client_secret: Option<String>,
    pub github_oauth_redirect_url: Option<String>,

    pub cookie_key: String,

    // S3
    pub s3_access_key_id: Option<String>,
    pub s3_secret_key: Option<String>,
    pub s3_endpoint: Option<String>,
    pub s3_region: Option<String>,
    pub s3_bucket_name: Option<String>,

    // JWT
    pub jwt_secret_key: String,
    pub access_token_expire_in_minute: i64,
    pub refresh_token_expire_in_days: i64,
    pub email_verification_token_expire_in_hours: i64,
    pub refresh_token_renewal_threshold_days: i64,
    pub cookie_secure: bool,

    // EMAIL
    pub email_service_api_key: String,

    // SSL/TLS
    pub ca: Option<String>,
    pub ca_path: Option<PathBuf>,
    pub client_cert: Option<String>,
    pub client_cert_path: Option<PathBuf>,
    pub client_key: Option<String>,
    pub client_key_path: Option<PathBuf>,
}

impl Config {
    pub async fn init() -> Result<Self, AppError> {
        let k8s_encryption_key = std::env::var("K8S_ENCRYPTION_KEY")
            .expect("K8S_ENCRYPTION_KEY must be set - generate with: openssl rand -base64 32");

        let k8s_config_path =
            get_config_value("K8S_KUBECONFIG", Some("K8S_KUBECONFIG"), None, None).await?;
        let k8s_in_cluster =
            get_config_value("K8S_IN_CLUSTER", Some("K8S_IN_CLUSTER"), None, Some(false))
                .await?
                .ok_or_else(|| {
                    AppError::EnvironmentVariableNotSetError("K8S_IN_CLUSTER".to_string())
                })?;

        let base_domain =
            std::env::var("BASE_DOMAIN").unwrap_or_else(|_| "app.pinespot.uz".to_string());

        let server_addres = get_config_value(
            "SERVER_ADDRES",
            Some("SERVER_ADDRES"),
            None,
            Some("0.0.0.0:8001".to_string()),
        )
        .await?
        .unwrap();

        let frontend_endpoint = get_config_value(
            "FRONTEND_ENDPOINT",
            Some("FRONTEND_ENDPOINT"),
            None,
            Some("http://localhost:5173".to_string()),
        )
        .await?
        .unwrap();

        let base_dir = find_project_root().unwrap_or_else(|| PathBuf::from("."));

        let debug = get_config_value("DEBUG", Some("DEBUG"), None, Some(false))
            .await?
            .unwrap();

        let tracing_level = get_config_value(
            "TRACING_LEVEL",
            Some("TRACING_LEVEL"),
            None,
            Some(Level::DEBUG),
        )
        .await?
        .unwrap();

        let database_url = get_config_value(
            "DATABASE_URL",
            Some("DATABASE_URL"),
            None,
            Some("postgresql://postgres:password@localhost:5432/pinespot_db".to_string()),
        )
        .await?
        .ok_or_else(|| AppError::EnvironmentVariableNotSetError("DATABASE_URL".to_string()))?;

        let redis_url = get_config_value(
            "REDIS_URL",
            Some("REDIS_URL"),
            None,
            Some("redis://localhost:6379/0".to_string()),
        )
        .await?
        .ok_or_else(|| AppError::EnvironmentVariableNotSetError("REDIS_URL".to_string()))?;
        let redis_host = get_config_value(
            "REDIS_HOST",
            Some("REDIS_HOST"),
            None,
            Some("localhost".to_string()),
        )
        .await?
        .ok_or_else(|| AppError::EnvironmentVariableNotSetError("REDIS_HOST".to_string()))?;
        let redis_port = get_config_value("REDIS_PORT", None, None, Some(6379))
            .await?
            .ok_or_else(|| AppError::EnvironmentVariableNotSetError("REDIS_PORT".to_string()))?;
        let redis_username =
            get_config_value("REDIS_USERNAME", Some("REDIS_USERNAME"), None, None).await?;
        let redis_password =
            get_config_value("REDIS_PASSWORD", Some("REDIS_PASSWORD"), None, None).await?;

        let amqp_addr = get_config_value(
            "AMQP_ADDR",
            Some("AMQP_ADDR"),
            None,
            Some("amqp://localhost:5672".to_string()),
        )
        .await?
        .ok_or_else(|| AppError::EnvironmentVariableNotSetError("AMQP_ADDR".to_string()))?;

        let kafka_bootstrap_servers = get_config_value(
            "KAFKA_BOOTSTRAP_SERVERS",
            Some("KAFKA_BOOTSTRAP_SERVERS"),
            None,
            Some("localhost:9092".to_string()),
        )
        .await?
        .ok_or_else(|| {
            AppError::EnvironmentVariableNotSetError("KAFKA_BOOTSTRAP_SERVERS".to_string())
        })?;

        let gcs_bucket_name =
            get_config_value("GCS_BUCKET_NAME", Some("GCS_BUCKET_NAME"), None, None).await?;
        let gcp_service_account_path = base_dir.join("certs/service-account.json");
        let gcp_service_account = get_config_value(
            "service_account.json",
            Some("SERVICE_ACCOUNT"),
            Some(&gcp_service_account_path),
            None,
        )
        .await?;

        let google_oauth_client_id = get_config_value(
            "GOOGLE_OAUTH_CLIENT_ID",
            Some("GOOGLE_OAUTH_CLIENT_ID"),
            None,
            None,
        )
        .await?;
        let google_oauth_client_secret = get_config_value(
            "GOOGLE_OAUTH_CLIENT_SECRET",
            Some("GOOGLE_OAUTH_CLIENT_SECRET"),
            None,
            None,
        )
        .await?;
        let google_oauth_redirect_url = get_config_value(
            "GOOGLE_OAUTH_REDIRECT_URL",
            Some("GOOGLE_OAUTH_REDIRECT_URL"),
            None,
            None,
        )
        .await?;

        let github_oauth_client_id = get_config_value(
            "GITHUB_OAUTH_CLIENT_ID",
            Some("GITHUB_OAUTH_CLIENT_ID"),
            None,
            None,
        )
        .await?;
        let github_oauth_client_secret = get_config_value(
            "GITHUB_OAUTH_CLIENT_SECRET",
            Some("GITHUB_OAUTH_CLIENT_SECRET"),
            None,
            None,
        )
        .await?;
        let github_oauth_redirect_url = get_config_value(
            "GITHUB_OAUTH_REDIRECT_URL",
            Some("GITHUB_OAUTH_REDIRECT_URL"),
            None,
            None,
        )
        .await?;

        let cookie_key = get_config_value("KEY", Some("KEY"), None, None)
            .await?
            .ok_or_else(|| AppError::EnvironmentVariableNotSetError("COOKIE_KEY".to_string()))?;

        let s3_access_key_id =
            get_config_value("S3_ACCESS_KEY_ID", Some("S3_ACCESS_KEY_ID"), None, None).await?;
        let s3_secret_key =
            get_config_value("S3_SECRET_KEY", Some("S3_SECRET_KEY"), None, None).await?;
        let s3_endpoint = get_config_value("S3_ENDPOINT", Some("S3_ENDPOINT"), None, None).await?;
        let s3_region = get_config_value("S3_REGION", Some("S3_REGION"), None, None).await?;
        let s3_bucket_name =
            get_config_value("S3_BUCKET_NAME", Some("S3_BUCKET_NAME"), None, None).await?;
        let jwt_secret_key = get_config_value("SECRET_KEY", Some("SECRET_KEY"), None, None)
            .await?
            .ok_or_else(|| {
                AppError::EnvironmentVariableNotSetError("JWT_SECRET_KEY".to_string())
            })?;
        let access_token_expire_in_minute = get_config_value(
            "ACCESS_TOKEN_EXPIRE_IN_MINUTE",
            Some("ACCESS_TOKEN_EXPIRE_IN_MINUTE"),
            None,
            Some(15),
        )
        .await?
        .ok_or_else(|| {
            AppError::EnvironmentVariableNotSetError("ACCESS_TOKEN_EXPIRE_IN_MINUTE".to_string())
        })?;
        let refresh_token_expire_in_days = get_config_value(
            "REFRESH_TOKEN_EXPIRE_IN_DAYS",
            Some("REFRESH_TOKEN_EXPIRE_IN_DAYS"),
            None,
            Some(90),
        )
        .await?
        .ok_or_else(|| {
            AppError::EnvironmentVariableNotSetError("REFRESH_TOKEN_EXPIRE_IN_DAYS".to_string())
        })?;
        let email_verification_token_expire_in_hours = get_config_value(
            "EMAIL_VERIFICATION_TOKEN_EXPIRE_IN_HOURS",
            Some("EMAIL_VERIFICATION_TOKEN_EXPIRE_IN_HOURS"),
            None,
            Some(24),
        )
        .await?
        .ok_or_else(|| {
            AppError::EnvironmentVariableNotSetError(
                "EMAIL_VERIFICATION_TOKEN_EXPIRE_IN_HOURS".to_string(),
            )
        })?;
        let refresh_token_renewal_threshold_days = get_config_value(
            "REFRESH_TOKEN_RENEWAL_THRESHOLD_DAYS",
            Some("REFRESH_TOKEN_RENEWAL_THRESHOLD_DAYS"),
            None,
            Some(7),
        )
        .await?
        .ok_or_else(|| {
            AppError::EnvironmentVariableNotSetError(
                "REFRESH_TOKEN_RENEWAL_THRESHOLD_DAYS".to_string(),
            )
        })?;
        let cookie_secure =
            get_config_value("COOKIE_SECURE", Some("COOKIE_SECURE"), None, Some(false))
                .await?
                .ok_or_else(|| {
                    AppError::EnvironmentVariableNotSetError("COOKIE_SECURE".to_string())
                })?;

        let email_service_api_key = get_config_value(
            "EMAIL_SERVICE_API_KEY",
            Some("EMAIL_SERVICE_API_KEY"),
            None,
            None,
        )
        .await?
        .ok_or_else(|| {
            AppError::EnvironmentVariableNotSetError("EMAIL_SERVICE_API_KEY".to_string())
        })?;

        // TLS certs: Docker secrets → fallback path
        let ca_path = base_dir.join("certs/ca/ca.pem");
        let ca = get_config_value("ca.pem", Some("CA"), Some(&ca_path), None).await?;
        let client_cert_path = base_dir.join("certs/client/client-cert.pem");
        let client_cert = get_config_value(
            "client-cert.pem",
            Some("CLIENT_CERT"),
            Some(&client_cert_path),
            None,
        )
        .await?;
        let client_key_path = base_dir.join("certs/client/client-key.pem");
        let client_key = get_config_value(
            "client-key.pem",
            Some("CLIENT_KEY"),
            Some(&client_key_path),
            None,
        )
        .await?;

        let pg_ssl_mode =
            get_config_value("ssl_mode", Some("SSL_MODE"), None, Some(PgSslMode::Disable))
                .await?
                .ok_or_else(|| {
                    AppError::EnvironmentVariableNotSetError("PG_SSL_MODE".to_string())
                })?;

        let config = Config {
            k8s_in_cluster,
            k8s_config_path,
            k8s_encryption_key,
            base_domain,
            server_addres,
            frontend_endpoint,
            base_dir,
            tracing_level,
            database_url,
            redis_url,
            redis_host,
            redis_port,
            redis_username,
            redis_password,
            amqp_addr,
            kafka_bootstrap_servers,
            gcs_bucket_name,
            gcp_service_account,
            gcp_service_account_path: Some(gcp_service_account_path),
            google_oauth_client_id,
            google_oauth_client_secret,
            google_oauth_redirect_url,
            github_oauth_client_id,
            github_oauth_client_secret,
            github_oauth_redirect_url,
            cookie_key,
            s3_access_key_id,
            s3_secret_key,
            s3_endpoint,
            s3_region,
            s3_bucket_name,
            jwt_secret_key,
            access_token_expire_in_minute,
            refresh_token_expire_in_days,
            email_verification_token_expire_in_hours,
            refresh_token_renewal_threshold_days,
            cookie_secure,
            email_service_api_key,
            ca_path: Some(ca_path),
            ca,
            client_cert_path: Some(client_cert_path),
            client_cert,
            client_key_path: Some(client_key_path),
            client_key,
            pg_ssl_mode,
        };

        Ok(config)
    }
}

fn find_project_root() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        if dir.join("Cargo.toml").exists() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// Try to resolve config value from Docker secrets, file path, or env var.
/// - `secret_name` → filename inside `/run/secrets/`
/// - `env_name` → optional environment variable key
/// - `fallback_path` → fallback file path (checked if exists)
///
/// Returns parsed `T` if found and successfully parsed.
pub async fn get_config_value<T>(
    secret_name: &str,
    env_name: Option<&str>,
    fallback_path: Option<&PathBuf>,
    fallback: T,
) -> Result<T, AppError>
where
    T: FromStr,
{
    // 1. Docker secrets
    let docker_secret = Path::new("/run/secrets").join(secret_name);
    if docker_secret.exists() {
        match fs::read_to_string(&docker_secret).await {
            Ok(content) => {
                if let Ok(parsed) = T::from_str(content.trim()) {
                    return Ok(parsed);
                }
            }
            Err(e) => {
                return Err(AppError::FileReadError(format!(
                    "Failed to read docker secret at {0}, {e}",
                    docker_secret.display()
                )));
            }
        }
    }

    // 2. Env var
    if let Some(env_key) = env_name
        && let Ok(val) = std::env::var(env_key)
        && let Ok(parsed) = T::from_str(val.trim())
    {
        return Ok(parsed);
    }

    // 3. Fallback file path
    if let Some(path) = fallback_path
        && path.exists()
    {
        match fs::read_to_string(path).await {
            Ok(content) => {
                if let Ok(parsed) = T::from_str(content.trim()) {
                    return Ok(parsed);
                }
            }
            Err(e) => {
                return Err(AppError::FileReadError(format!(
                    "Failed to read fallback file at {}, {}",
                    path.display(),
                    e
                )));
            }
        }
    }

    // 4. Final fallback
    Ok(fallback)
}
