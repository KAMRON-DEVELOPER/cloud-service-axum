use crate::utilities::{config::Config, errors::AppError};
use kube::Client;

#[derive(Clone)]
pub struct Kubernetes {
    pub client: Client,
}

impl Kubernetes {
    pub async fn new(config: &Config) -> Result<Self, AppError> {
        // let mut options = config
        //     .database_url
        //     .as_ref()
        //     .unwrap()
        //     .parse()
        //     .map_err(|_| AppError::DatabaseParsingError)?;

        let client = Client::try_default().await?;
        Ok(Kubernetes { client })
    }
}
