use serde::{Deserialize, Serialize};

use crate::utilities::errors::AppError;

#[derive(Serialize, Debug)]
pub struct ListResponse<T> {
    pub data: Vec<T>,
    pub total: i64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Pagination {
    #[serde(default = "default_offset")]
    pub offset: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_offset() -> i64 {
    0
}

fn default_limit() -> i64 {
    20
}

impl Pagination {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.offset < 0 {
            return Err(AppError::ValidationError(
                "Offset must be positive".to_string(),
            ));
        }

        if self.limit < 0 {
            return Err(AppError::ValidationError("Limit must positive".to_string()));
        } else if self.limit == 0 {
            return Err(AppError::ValidationError(
                "Limit must not be zero!".to_string(),
            ));
        }

        if self.limit > 100 {
            return Err(AppError::ValidationError(
                "Limit cannot exceed 100".to_string(),
            ));
        }

        Ok(())
    }
}
