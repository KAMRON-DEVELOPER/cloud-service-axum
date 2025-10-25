use axum::{extract::State, response::IntoResponse};
use shared::{
    services::{amqp::Amqp, database::Database, kafka::Kafka},
    utilities::{errors::AppError, jwt::Claims},
};

use crate::services::build_kubernetes::Kubernetes;

pub async fn get_projects(
    claims: Claims,
    State(database): State<Database>,
    State(kubernetes): State<Kubernetes>,
    State(kafka): State<Kafka>,
    State(amqp): State<Amqp>,
) -> Result<impl IntoResponse, AppError> {
    let _: uuid::Uuid = claims.sub;
    let _: rdkafka::producer::FutureProducer = kafka.producer;
    let _: std::sync::Arc<rdkafka::consumer::StreamConsumer> = kafka.consumer;
    let _: lapin::Channel = amqp.channel().await?;
    let _: sqlx::Pool<sqlx::Postgres> = database.pool;
    let _: kube::Client = kubernetes.client;

    Ok(())
}

pub async fn get_project() -> Result<impl IntoResponse, AppError> {
    Ok(())
}

pub async fn create_project() -> Result<impl IntoResponse, AppError> {
    Ok(())
}

pub async fn delete_project() -> Result<impl IntoResponse, AppError> {
    Ok(())
}

pub async fn update_project() -> Result<impl IntoResponse, AppError> {
    Ok(())
}
