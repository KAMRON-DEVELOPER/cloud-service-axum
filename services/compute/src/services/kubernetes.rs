use std::collections::BTreeMap;
use std::collections::HashMap;

use k8s_openapi::ByteString;
use k8s_openapi::api::apps::v1::{Deployment as K8sDeployment, DeploymentSpec};
use k8s_openapi::api::core::v1::{
    Container, ContainerPort, EnvVar, EnvVarSource, PodSpec, PodTemplateSpec, ResourceRequirements,
    Secret as K8sSecret, SecretKeySelector, Service, ServicePort, ServiceSpec,
};
use k8s_openapi::api::networking::v1::{
    HTTPIngressPath, HTTPIngressRuleValue, Ingress, IngressBackend, IngressRule,
    IngressServiceBackend, IngressSpec, IngressTLS, ServiceBackendPort,
};
use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::LabelSelector;
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use kube::api::{DeleteParams, ObjectMeta, Patch, PatchParams, PostParams};
use kube::{Api, Client};
use shared::utilities::errors::AppError;
use sqlx::PgPool;
use uuid::Uuid;

use crate::features::models::{Deployment, DeploymentStatus, ResourceSpec};
use crate::features::repository::{
    DeploymentEventRepository, DeploymentRepository, DeploymentSecretRepository,
};
use crate::features::schemas::{
    CreateDeploymentRequest, DeploymentDetailResponse, DeploymentResponse,
};
use crate::utilities::encryption::EncryptionService;

pub struct DeploymentService;

impl DeploymentService {
    /// Create a new deployment with Kubernetes resources
    pub async fn create(
        pool: &PgPool,
        k8s_client: &Client,
        encryption_key: &str,
        user_id: Uuid,
        project_id: Uuid,
        base_domain: &str,
        req: CreateDeploymentRequest,
    ) -> Result<DeploymentResponse, AppError> {
        let encryption_service = EncryptionService::new(encryption_key)?;

        // Generate cluster resource names
        let cluster_namespace = "default"; // Or use user-specific namespace
        let cluster_deployment_name = format!("{}-{}", project_id, req.name)
            .to_lowercase()
            .replace("_", "-");

        // Determine subdomain
        let subdomain = req.subdomain.unwrap_or_else(|| {
            format!("{}-{}", req.name, &user_id.to_string()[..8]).to_lowercase()
        });

        let external_url = format!("{}.{}", subdomain, base_domain);

        // Prepare env vars JSON
        let env_vars_json = serde_json::to_value(req.env_vars.clone().unwrap_or_default())?;

        // Prepare resources JSON
        let resources_json = serde_json::to_value(req.resources.unwrap_or_default())?;

        // Prepare labels
        let labels_json = req.labels.map(|l| serde_json::to_value(l).unwrap());

        // Start transaction
        let mut tx = pool.begin().await?;

        // Create deployment record
        let deployment = DeploymentRepository::create(
            &mut tx,
            user_id,
            project_id,
            &req.name,
            &req.image,
            env_vars_json.clone(),
            req.replicas,
            resources_json.clone(),
            labels_json,
            cluster_namespace,
            &cluster_deployment_name,
        )
        .await?;

        // Store encrypted secrets
        if let Some(secrets) = &req.secrets {
            for (key, value) in secrets {
                let encrypted_value = encryption_service.encrypt(&value)?;
                DeploymentSecretRepository::create(&mut tx, deployment.id, &key, encrypted_value)
                    .await?;
            }
        }

        // Commit transaction
        tx.commit().await?;

        // Create Kubernetes resources
        Self::create_k8s_resources(
            k8s_client,
            &deployment,
            req.port,
            &external_url,
            req.env_vars.unwrap_or_default(),
            req.secrets.unwrap_or_default(),
        )
        .await?;

        // Log event
        DeploymentEventRepository::create(
            pool,
            deployment.id,
            "deployment_created",
            Some("Deployment created successfully"),
        )
        .await?;

        // Update status to running
        DeploymentRepository::update_status(pool, deployment.id, DeploymentStatus::Running).await?;

        Ok(DeploymentResponse {
            id: deployment.id,
            project_id: deployment.project_id,
            name: deployment.name,
            image: deployment.image,
            status: DeploymentStatus::Running,
            replicas: deployment.replicas,
            resources: serde_json::from_value(resources_json)?,
            external_url: Some(external_url),
            created_at: deployment.created_at,
            updated_at: deployment.updated_at,
        })
    }

    /// Create Kubernetes Deployment, Service, Secret, and Ingress
    async fn create_k8s_resources(
        client: &Client,
        deployment: &Deployment,
        container_port: i32,
        external_url: &str,
        env_vars: HashMap<String, String>,
        secrets: HashMap<String, String>,
    ) -> Result<(), AppError> {
        let namespace = &deployment.cluster_namespace;
        let name = &deployment.cluster_deployment_name;

        // Parse resources
        let resources: ResourceSpec = serde_json::from_value(deployment.resources.clone())?;

        // Create labels
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), name.clone());
        labels.insert("deployment-id".to_string(), deployment.id.to_string());

        // 1. Create Kubernetes Secret if there are secrets
        if !secrets.is_empty() {
            let secret_name = format!("{}-secrets", name);
            let mut secret_data = BTreeMap::new();

            for (key, value) in &secrets {
                secret_data.insert(key.clone(), ByteString(value.clone().into_bytes()));
            }

            let secret = K8sSecret {
                metadata: ObjectMeta {
                    name: Some(secret_name.clone()),
                    namespace: Some(namespace.clone()),
                    labels: Some(labels.clone()),
                    ..Default::default()
                },
                data: Some(secret_data),
                ..Default::default()
            };

            let secrets_api: Api<K8sSecret> = Api::namespaced(client.clone(), namespace);
            secrets_api
                .create(&PostParams::default(), &secret)
                .await
                .map_err(|e| AppError::InternalError(format!("Failed to create secret: {}", e)))?;
        }

        // 2. Build container environment variables
        let mut container_env = vec![];

        // Regular env vars
        for (key, value) in env_vars {
            container_env.push(EnvVar {
                name: key,
                value: Some(value),
                ..Default::default()
            });
        }

        // Secret env vars
        if !secrets.is_empty() {
            let secret_name = format!("{}-secrets", name);
            for key in secrets.keys() {
                container_env.push(EnvVar {
                    name: key.clone(),
                    value_from: Some(EnvVarSource {
                        secret_key_ref: Some(SecretKeySelector {
                            name: secret_name.clone(),
                            key: key.clone(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                });
            }
        }

        // 3. Create Kubernetes Deployment
        let mut resource_requirements = BTreeMap::new();
        resource_requirements.insert(
            "cpu".to_string(),
            Quantity(format!("{}m", resources.cpu_request_millicores)),
        );
        resource_requirements.insert(
            "memory".to_string(),
            Quantity(format!("{}Mi", resources.memory_request_mb)),
        );

        let mut resource_limits = BTreeMap::new();
        resource_limits.insert(
            "cpu".to_string(),
            Quantity(format!("{}m", resources.cpu_limit_millicores)),
        );
        resource_limits.insert(
            "memory".to_string(),
            Quantity(format!("{}Mi", resources.memory_limit_mb)),
        );

        let k8s_deployment = K8sDeployment {
            metadata: ObjectMeta {
                name: Some(name.clone()),
                namespace: Some(namespace.clone()),
                labels: Some(labels.clone()),
                ..Default::default()
            },
            spec: Some(DeploymentSpec {
                replicas: Some(deployment.replicas),
                selector: LabelSelector {
                    match_labels: Some(labels.clone()),
                    ..Default::default()
                },
                template: PodTemplateSpec {
                    metadata: Some(ObjectMeta {
                        labels: Some(labels.clone()),
                        ..Default::default()
                    }),
                    spec: Some(PodSpec {
                        containers: vec![Container {
                            name: "app".to_string(),
                            image: Some(deployment.image.clone()),
                            ports: Some(vec![ContainerPort {
                                container_port,
                                ..Default::default()
                            }]),
                            env: if container_env.is_empty() {
                                None
                            } else {
                                Some(container_env)
                            },
                            resources: Some(ResourceRequirements {
                                requests: Some(resource_requirements),
                                limits: Some(resource_limits),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }),
                },
                ..Default::default()
            }),
            ..Default::default()
        };

        let deployments_api: Api<K8sDeployment> = Api::namespaced(client.clone(), namespace);
        deployments_api
            .create(&PostParams::default(), &k8s_deployment)
            .await
            .map_err(|e| {
                AppError::InternalError(format!("Failed to create k8s deployment: {}", e))
            })?;

        // 4. Create Kubernetes Service
        let service = Service {
            metadata: ObjectMeta {
                name: Some(name.clone()),
                namespace: Some(namespace.clone()),
                labels: Some(labels.clone()),
                ..Default::default()
            },
            spec: Some(ServiceSpec {
                selector: Some(labels.clone()),
                ports: Some(vec![ServicePort {
                    port: 80,
                    target_port: Some(IntOrString::Int(container_port)),
                    protocol: Some("TCP".to_string()),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            ..Default::default()
        };

        let services_api: Api<Service> = Api::namespaced(client.clone(), namespace);
        services_api
            .create(&PostParams::default(), &service)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to create service: {}", e)))?;

        // 5. Create Ingress with Traefik annotations
        let mut annotations = BTreeMap::new();
        annotations.insert(
            "kubernetes.io/ingress.class".to_string(),
            "traefik".to_string(),
        );
        annotations.insert(
            "traefik.ingress.kubernetes.io/router.entrypoints".to_string(),
            "websecure".to_string(),
        );
        annotations.insert(
            "cert-manager.io/cluster-issuer".to_string(),
            "letsencrypt-prod".to_string(), // Assuming cert-manager is installed
        );

        let ingress = Ingress {
            metadata: ObjectMeta {
                name: Some(name.clone()),
                namespace: Some(namespace.clone()),
                labels: Some(labels.clone()),
                annotations: Some(annotations),
                ..Default::default()
            },
            spec: Some(IngressSpec {
                rules: Some(vec![IngressRule {
                    host: Some(external_url.to_string()),
                    http: Some(HTTPIngressRuleValue {
                        paths: vec![HTTPIngressPath {
                            path: Some("/".to_string()),
                            path_type: "Prefix".to_string(),
                            backend: IngressBackend {
                                service: Some(IngressServiceBackend {
                                    name: name.clone(),
                                    port: Some(ServiceBackendPort {
                                        number: Some(80),
                                        ..Default::default()
                                    }),
                                }),
                                ..Default::default()
                            },
                        }],
                    }),
                }]),
                tls: Some(vec![IngressTLS {
                    hosts: Some(vec![external_url.to_string()]),
                    secret_name: Some(format!("{}-tls", name)),
                }]),
                ..Default::default()
            }),
            ..Default::default()
        };

        let ingress_api: Api<Ingress> = Api::namespaced(client.clone(), namespace);
        ingress_api
            .create(&PostParams::default(), &ingress)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to create ingress: {}", e)))?;

        Ok(())
    }

    /// Scale deployment
    pub async fn scale(
        pool: &PgPool,
        k8s_client: &Client,
        deployment_id: Uuid,
        user_id: Uuid,
        new_replicas: i32,
    ) -> Result<DeploymentResponse, AppError> {
        // Update database
        let deployment =
            DeploymentRepository::update_replicas(pool, deployment_id, user_id, new_replicas)
                .await?;

        // Update Kubernetes deployment
        let deployments_api: Api<K8sDeployment> =
            Api::namespaced(k8s_client.clone(), &deployment.cluster_namespace);

        let patch = serde_json::json!({
            "spec": {
                "replicas": new_replicas
            }
        });

        deployments_api
            .patch(
                &deployment.cluster_deployment_name,
                &PatchParams::default(),
                &Patch::Strategic(patch),
            )
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to scale deployment: {}", e)))?;

        // Log event
        DeploymentEventRepository::create(
            pool,
            deployment.id,
            "deployment_scaled",
            Some(&format!("Scaled to {} replicas", new_replicas)),
        )
        .await?;

        let resources: ResourceSpec = serde_json::from_value(deployment.resources.clone())?;

        Ok(DeploymentResponse {
            id: deployment.id,
            project_id: deployment.project_id,
            name: deployment.name,
            image: deployment.image,
            status: deployment.status,
            replicas: deployment.replicas,
            resources,
            external_url: None, // You'd need to query this from Ingress
            created_at: deployment.created_at,
            updated_at: deployment.updated_at,
        })
    }

    /// Delete deployment and cleanup Kubernetes resources
    pub async fn delete(
        pool: &PgPool,
        k8s_client: &Client,
        deployment_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        // Get deployment info
        let deployment = DeploymentRepository::get_by_id(pool, deployment_id, user_id).await?;

        let namespace = &deployment.cluster_namespace;
        let name = &deployment.cluster_deployment_name;

        // Delete Kubernetes resources
        let delete_params = DeleteParams::default();

        // Delete Ingress
        let ingress_api: Api<Ingress> = Api::namespaced(k8s_client.clone(), namespace);
        let _ = ingress_api.delete(name, &delete_params).await;

        // Delete Service
        let service_api: Api<Service> = Api::namespaced(k8s_client.clone(), namespace);
        let _ = service_api.delete(name, &delete_params).await;

        // Delete Deployment
        let deployment_api: Api<K8sDeployment> = Api::namespaced(k8s_client.clone(), namespace);
        let _ = deployment_api.delete(name, &delete_params).await;

        // Delete Secret
        let secret_name = format!("{}-secrets", name);
        let secret_api: Api<K8sSecret> = Api::namespaced(k8s_client.clone(), namespace);
        let _ = secret_api.delete(&secret_name, &delete_params).await;

        // Delete from database (cascades to secrets and events)
        DeploymentRepository::delete(pool, deployment_id, user_id).await?;

        Ok(())
    }

    /// Get deployment details with decrypted secret keys (but not values)
    pub async fn get_detail(
        pool: &PgPool,
        deployment_id: Uuid,
        user_id: Uuid,
    ) -> Result<DeploymentDetailResponse, AppError> {
        let deployment = DeploymentRepository::get_by_id(pool, deployment_id, user_id).await?;

        let secrets =
            DeploymentSecretRepository::get_all_by_deployment(pool, deployment_id).await?;
        let secret_keys: Vec<String> = secrets.into_iter().map(|s| s.key).collect();

        let env_vars: HashMap<String, String> =
            serde_json::from_value(deployment.env_vars.clone())?;
        let resources: ResourceSpec = serde_json::from_value(deployment.resources.clone())?;
        let labels: Option<HashMap<String, String>> = deployment
            .labels
            .as_ref()
            .map(|l| serde_json::from_value(l.clone()).unwrap());

        Ok(DeploymentDetailResponse {
            id: deployment.id,
            project_id: deployment.project_id,
            name: deployment.name,
            image: deployment.image,
            status: deployment.status,
            replicas: deployment.replicas,
            ready_replicas: None, // Would need to query from K8s
            resources,
            env_vars,
            secret_keys,
            labels,
            external_url: None, // Would need to query from Ingress
            cluster_namespace: deployment.cluster_namespace,
            created_at: deployment.created_at,
            updated_at: deployment.updated_at,
        })
    }
}
