use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::features::models::DeploymentStatus;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateDeploymentRequest {
    pub name: String,
    pub image: String,
    pub replicas: i32,
    pub port: i32,
    pub env_vars: Option<serde_json::Value>,
    pub cpu_limit: Option<String>,
    pub memory_limit: Option<String>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentResponse {
    pub id: Uuid,
    pub name: String,
    pub image: String,
    pub status: DeploymentStatus,
    pub replicas: i32,
    pub ready_replicas: i32,
    pub external_url: Option<String>,
    pub created_at: DateTime<Utc>,
}
