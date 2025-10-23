use crate::{
    features::kubernetes::models::Deployment, services::build_kubernetes::Kubernetes,
    utilities::errors::AppError,
};
// use futures::{StreamExt, TryStreamExt};
use k8s_openapi::api::core::v1::Pod;
use kube::{
    Client,
    api::{Api, ListParams, PostParams, ResourceExt},
};
use sqlx::PgPool;

pub async fn get_many_listings(pool: &PgPool, kubernetes: &Kubernetes) -> Result<(), AppError> {
    let client = kubernetes.client.clone();

    // Read pods in the configured namespace into the typed interface from k8s-openapi
    let pods: Api<Pod> = Api::default_namespaced(client);
    for p in pods.list(&ListParams::default()).await? {
        println!("found pod {}", p.name_any());
    }

    Ok(())
}

pub async fn create_k8s_deployment(
    namespace: &str,
    deployment: &Deployment,
) -> Result<(), AppError> {
    let client = Client::try_default().await?;
    let deployments: Api<K8sDeployment> = Api::namespaced(client, namespace);

    Ok(())
}
