use kube::Client;
use shared::utilities::{config::Config, errors::AppError};

#[derive(Clone)]
pub struct Kubernetes {
    pub client: Client,
}

impl Kubernetes {
    pub async fn new(config: &Config) -> Result<Self, AppError> {
        let mut _options = config.database_url.as_ref().unwrap();

        let client = Client::try_default().await?;
        Ok(Kubernetes { client })
    }
}
