use std::sync::Arc;

use lapin::{
    Channel, Connection, ConnectionProperties,
    tcp::{OwnedIdentity, OwnedTLSConfig},
};
use tracing::info;

use crate::utilities::{config::Config, errors::AppError};

#[derive(Clone)]
pub struct Amqp {
    connection: Arc<Connection>,
}

impl Amqp {
    pub async fn new(config: &Config) -> Result<Self, AppError> {
        let uri = config
            .amqp_addr
            .clone()
            .ok_or_else(|| AppError::MissingAmqpUrlError)?;

        let options = ConnectionProperties::default();

        if config.client_cert.is_some() && config.client_key.is_some() {
            let connection = Connection::connect(&uri, options).await?;
            info!("✅ RabbitMQ connection established.");

            return Ok(Self {
                connection: Arc::new(connection),
            });
        }

        let mut tlsconfig = OwnedTLSConfig::default();

        if let (Some(ca), Some(client_cert), Some(client_key)) =
            (&config.ca, &config.client_cert, &config.client_key)
        {
            info!("🔐 AMQP SSL/TLS enabled");
            tlsconfig.cert_chain = Some(ca.to_string());
            tlsconfig.identity = Some(OwnedIdentity::PKCS8 {
                pem: client_cert.clone().into_bytes(),
                key: client_key.clone().into_bytes(),
            });
        }

        let connection = Connection::connect_with_config(&uri, options, tlsconfig).await?;
        info!("✅ RabbitMQ connection established.");

        Ok(Self {
            connection: Arc::new(connection),
        })
    }

    pub async fn channel(&self) -> Result<Channel, AppError> {
        Ok(self.connection.create_channel().await?)
    }
}
