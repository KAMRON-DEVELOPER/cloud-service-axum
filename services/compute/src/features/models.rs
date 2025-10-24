use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

#[derive(Type, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[sqlx(type_name = "deployment_status", rename_all = "lowercase")]
pub enum DeploymentStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Terminated,
}

#[derive(FromRow, Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResourceSpec {
    /// CPU request in millicores (e.g. 250 = 0.25 CPU)
    pub cpu_request_millicores: i32,
    /// CPU limit in millicores
    pub cpu_limit_millicores: i32,
    /// Memory request in MB
    pub memory_request_mb: i32,
    /// Memory limit in MB
    pub memory_limit_mb: i32,
}

#[derive(FromRow, Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Deployment {
    pub id: Uuid,
    /// Owner of the deployment (user)
    pub user_id: Uuid,
    /// Project grouping
    pub project_id: Uuid,
    /// Logical name of this deployment (unique per project)
    pub name: String,
    /// OCI image reference
    pub image: String,
    /// Arbitrary environment variables / config for the container; stored as JSONB
    pub env_vars: serde_json::Value,
    /// Desired replicas
    pub replicas: i32,
    /// Resource specification (stored as JSONB in DB)
    pub resources: serde_json::Value,
    /// Optional labels and annotations (k8s-like) as JSONB
    pub labels: Option<serde_json::Value>,
    /// Status enum
    pub status: DeploymentStatus,
    /// k8s namespace used in cluster
    pub cluster_namespace: String,
    /// Name of the k8s deployment resource in the cluster
    pub cluster_deployment_name: String,
    /// optional node selector (JSON map or string representation)
    pub node_selector: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(FromRow, Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentEvent {
    pub id: Uuid,
    pub deployment_id: Uuid,
    pub event_type: String,
    pub message: Option<String>,
    pub created_at: DateTime<Utc>,
}
