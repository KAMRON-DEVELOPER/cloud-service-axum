use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(sqlx::Type, Serialize, Deserialize, Debug, Clone, Copy)]
#[sqlx(type_name = "plan_type", rename_all = "lowercase")]
pub enum PlanType {
    Free,
    Basic,
    Pro,
}

#[derive(FromRow, Serialize, Deserialize, Debug, Clone)]
pub struct BillingAccount {
    pub id: Uuid,
    pub user_id: Uuid,
    pub plan: PlanType,
    pub credits: f64,
    pub currency: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(FromRow, Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub id: Uuid,
    pub account_id: Uuid,
    pub amount: f64,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
}
