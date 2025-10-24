use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(sqlx::Type, Serialize, Deserialize, Debug, Clone, Copy)]
#[sqlx(type_name = "deployment_status", rename_all = "lowercase")]
pub enum DeploymentStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Terminated,
}

#[derive(FromRow, Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(FromRow, Serialize, Deserialize, Debug, Clone)]
pub struct Job {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub image: String,
    pub command: Option<String>,
    pub status: DeploymentStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct Deployment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub image: String,
    pub env_vars: serde_json::Value,
    pub replicas: i32,
    pub cpu_limit_millicores: i32,
    pub memory_limit_mb: i32,
    pub status: String,
    pub cluster_namespace: String,
    pub cluster_deployment_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct DeploymentEvent {
    pub id: Uuid,
    pub deployment_id: Uuid,
    pub event_type: String,
    pub message: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct BillingRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub deployment_id: Option<Uuid>,
    pub cpu_millicores: i32,
    pub memory_mb: i32,
    pub cost_per_hour: f64,
    pub hours_used: f64,
    pub total_cost: f64,
    pub charged_at: DateTime<Utc>,
}
