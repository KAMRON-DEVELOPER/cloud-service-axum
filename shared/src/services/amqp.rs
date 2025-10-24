use std::sync::Arc;

use lapin::{Channel, Connection, ConnectionProperties};
use tracing::info;

use crate::utilities::{config::Config, errors::AppError};

#[derive(Clone)]
pub struct Amqp {
    connection: Arc<Connection>,
}

impl Amqp {
    pub async fn new(config: &Config) -> Result<Self, AppError> {
        let connection = Connection::connect(
            &config.amqp_url.clone().unwrap(),
            ConnectionProperties::default(),
        )
        .await?;

        info!("✅ RabbitMQ connection established.");

        Ok(Self {
            connection: Arc::new(connection),
        })
    }

    pub async fn channel(&self) -> Result<Channel, AppError> {
        Ok(self.connection.create_channel().await?)
    }
}
