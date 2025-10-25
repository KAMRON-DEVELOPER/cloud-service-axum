// use kube::Client;
// use shared::utilities::{config::Config, errors::AppError};

// #[derive(Clone)]
// pub struct Kubernetes {
//     pub client: Client,
// }

// impl Kubernetes {
//     pub async fn new(config: &Config) -> Result<Self, AppError> {
//         let mut _options = config.database_url.as_ref().unwrap();

//         let client = Client::try_default().await?;
//         Ok(Kubernetes { client })
//     }
// }

use kube::{Client, Config as KubeConfig};
use shared::utilities::config::Config;

#[derive(Clone)]
pub struct Kubernetes {
    pub client: Client,
}

impl Kubernetes {
    pub async fn new(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        let client = if config.k8s_in_cluster {
            // Running inside Kubernetes cluster
            let kube_config = KubeConfig::incluster()?;
            Client::try_from(kube_config)?
        } else {
            // Running outside cluster - use kubeconfig
            let kube_config = if let Some(path) = &config.k8s_config_path {
                KubeConfig::from_kubeconfig(&kube::config::KubeConfigOptions {
                    path: Some(std::path::PathBuf::from(path)),
                    ..Default::default()
                })
                .await?
            } else {
                KubeConfig::infer().await?
            };

            Client::try_from(kube_config)?
        };

        Ok(Self { client })
    }
}
