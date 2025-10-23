use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::utilities::{config::Config, errors::AppError};

#[derive(Clone)]
pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub async fn new(config: &Config) -> Result<Self, AppError> {
        let mut options: PgConnectOptions = config
            .database_url
            .as_ref()
            .unwrap()
            .parse()
            .map_err(|_| AppError::DatabaseParsingError)?;

        if let Some(ssl_mode) = config.ssl_mode {
            options = options.ssl_mode(ssl_mode);
        }

        // if let Some(ca_path) = &config.ca_path {
        //     if ca_path.exists() {
        //         options = options.ssl_root_cert(ca_path);
        //     }
        // }
        if let Some(ca) = &config.ca {
            options = options.ssl_root_cert_from_pem(ca.as_bytes().to_owned());
        }
        // if let Some(client_cert_path) = &config.client_cert_path {
        //     if client_cert_path.exists() {
        //         options = options.ssl_client_cert(client_cert_path);
        //     }
        // }
        if let Some(client_cert) = &config.client_cert {
            options = options.ssl_client_cert_from_pem(client_cert.as_bytes());
        }
        // if let Some(client_key_path) = &config.client_key_path {
        //     if client_key_path.exists() {
        //         options = options.ssl_client_key(client_key_path);
        //     }
        // }
        if let Some(client_key) = &config.client_key {
            options = options.ssl_client_key_from_pem(client_key.as_bytes());
        }

        let pool = PgPoolOptions::new()
            .max_connections(100)
            .connect_with(options)
            .await?;

        Ok(Self { pool })
    }
}
