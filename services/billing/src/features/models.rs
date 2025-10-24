use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

#[derive(Type, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[sqlx(type_name = "transaction_type", rename_all = "snake_case")]
pub enum TransactionType {
    InitialCredit,
    UsageCharge,
    TopUp,
    Refund,
}

#[derive(FromRow, Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserWallet {
    pub id: Uuid,
    pub user_id: Uuid,
    pub credit_balance: BigDecimal,
    pub currency: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(FromRow, Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WalletTransaction {
    pub id: Uuid,
    pub wallet_id: Uuid,
    pub amount: BigDecimal,
    #[sqlx(rename = "type")]
    pub transaction_type: TransactionType,
    pub details: Option<String>,
    pub billing_record_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(FromRow, Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BillingRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub deployment_id: Option<Uuid>,
    pub cpu_millicores: i32,
    pub memory_mb: i32,
    pub cost_per_hour: BigDecimal,
    pub hours_used: BigDecimal,
    pub total_cost: BigDecimal,
    pub charged_at: DateTime<Utc>,
}
