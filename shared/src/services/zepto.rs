use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::utilities::{config::Config, errors::AppError};

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct ZeptoResponseData {
    code: String,
    message: String,
    #[serde(default)]
    additional_info: Vec<serde_json::Value>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct ZeptoResponse {
    data: Vec<ZeptoResponseData>,
    message: String,
    request_id: String,
    object: String,
}

#[derive(Serialize)]
struct EmailAddress {
    name: String,
    address: String,
}

#[derive(Serialize)]
struct Recipient {
    email_address: EmailAddress,
}

#[derive(Serialize)]
struct Payload {
    template_alias: String,
    from: EmailAddress,
    to: Vec<Recipient>,
    merge_info: serde_json::Value,
}

pub struct ZeptoMail {
    api_url: String,
    client: Client,
}

impl Default for ZeptoMail {
    fn default() -> Self {
        Self::new()
    }
}

impl ZeptoMail {
    pub fn new() -> Self {
        Self {
            api_url: "https://api.zeptomail.com/v1.1/email/template".to_string(),
            client: Client::new(),
        }
    }

    pub async fn send_verification_link_email(
        &self,
        to_email: String,
        name: String,
        verification_link: String,
        config: &Config,
    ) -> Result<(), AppError> {
        let payload = Payload {
            template_alias: "pinespot-email-verification-link-key-alias".to_string(),
            from: EmailAddress {
                name: "PineSpot Verification".to_string(),
                address: "verification@kronk.uz".to_string(),
            },
            to: vec![Recipient {
                email_address: EmailAddress {
                    address: to_email.to_string(),
                    name: name.clone(),
                },
            }],
            merge_info: serde_json::json!({
                "verification_link": verification_link
            }),
        };

        debug!("Sending email to '{}' with email '{}'", name, to_email);

        let api_key = config
            .email_service_api_key
            .clone()
            .ok_or(AppError::MissingEmailServiceApiKeyError)?;

        let res = self
            .client
            .post(&self.api_url)
            .header("accept", "application/json")
            .header("content-type", "application/json")
            .header("authorization", format!("Zoho-enczapikey {}", api_key))
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                AppError::ExternalServiceError(format!("ZeptoMail request failed: {}", e))
            })?;

        let status = res.status();
        let body = res.json::<ZeptoResponse>().await.map_err(|e| {
            AppError::ExternalServiceError(format!("Failed to parse ZeptoMail response: {}", e))
        })?;

        if status.is_success() {
            tracing::info!("ZeptoMail success: {:?}", body);
            Ok(())
        } else {
            Err(AppError::ExternalServiceError(format!(
                "ZeptoMail error (status {}): {:?}",
                status, body
            )))
        }
    }
}
