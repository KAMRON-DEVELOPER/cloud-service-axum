use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

// ============================================================================
// K3s-Specific Resource Models
// ============================================================================

#[derive(Type, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[sqlx(type_name = "deployment_status", rename_all = "lowercase")]
pub enum DeploymentStatus {
    Pending,
    Running,
    Failed,
    Stopped,
    Deleted,
}

/// User deployments (containers/pods running on K3s)
#[derive(FromRow, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Deployment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub organization_id: Option<Uuid>,

    // K3s identifiers
    pub name: String,
    pub namespace: String, // Each user gets their own namespace
    pub k8s_deployment_name: String,

    // Container config
    pub image: String, // e.g., "nginx:latest", "user/custom-app:v1"
    pub replicas: i32,
    pub port: i32,
    pub env_vars: Option<serde_json::Value>, // JSON of env variables

    // Resource limits
    pub cpu_limit: Option<String>,    // e.g., "500m"
    pub memory_limit: Option<String>, // e.g., "512Mi"
    pub cpu_request: Option<String>,
    pub memory_request: Option<String>,

    // Status
    pub status: DeploymentStatus,
    pub status_message: Option<String>,
    pub ready_replicas: i32,

    // Networking
    pub internal_url: Option<String>, // e.g., "service.namespace.svc.cluster.local"
    pub external_url: Option<String>, // If exposed via ingress

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Resource quotas per user/org
#[derive(FromRow, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResourceQuota {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub organization_id: Option<Uuid>,

    // Limits
    pub max_deployments: i32,
    pub max_cpu: String,     // e.g., "4000m" (4 CPUs)
    pub max_memory: String,  // e.g., "8Gi"
    pub max_storage: String, // e.g., "50Gi"

    // Current usage (updated periodically)
    pub used_cpu: Option<String>,
    pub used_memory: Option<String>,
    pub used_storage: Option<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
