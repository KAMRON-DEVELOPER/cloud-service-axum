use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

// ============================================
// ENUMS
// ============================================

#[derive(Type, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[sqlx(type_name = "deployment_status", rename_all = "lowercase")]
pub enum DeploymentStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Terminated,
}

// ============================================
// MODELS
// ============================================

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

#[derive(FromRow, Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Deployment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub image: String,
    pub env_vars: serde_json::Value,
    pub replicas: i32,
    pub resources: serde_json::Value,
    pub labels: Option<serde_json::Value>,
    pub status: DeploymentStatus,
    pub cluster_namespace: String,
    pub cluster_deployment_name: String,
    pub node_selector: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(FromRow, Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentSecret {
    pub id: Uuid,
    pub deployment_id: Uuid,
    pub key: String,
    pub value: Vec<u8>,
    pub created_at: DateTime<Utc>,
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

// ============================================
// HELPER STRUCTS FOR JSONB FIELDS
// ============================================

/// Resource specification stored in the `resources` JSONB field
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResourceSpec {
    pub cpu_request_millicores: i32,
    pub cpu_limit_millicores: i32,
    pub memory_request_mb: i32,
    pub memory_limit_mb: i32,
}

impl Default for ResourceSpec {
    fn default() -> Self {
        Self {
            cpu_request_millicores: 250,
            cpu_limit_millicores: 500,
            memory_request_mb: 256,
            memory_limit_mb: 512,
        }
    }
}
