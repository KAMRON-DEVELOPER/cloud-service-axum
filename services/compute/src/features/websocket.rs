use axum::{
    extract::{
        Path, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use k8s_openapi::api::apps::v1::Deployment as K8sDeployment;
use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{Api, ListParams},
    runtime::{WatchStreamExt, watcher},
};
use serde::{Deserialize, Serialize};
use shared::{
    services::database::Database,
    utilities::{errors::AppError, jwt::Claims},
};
use std::time::Duration;
use tokio::time::interval;
use uuid::Uuid;

use crate::{features::repository::DeploymentRepository, services::build_kubernetes::Kubernetes};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum DeploymentMessage {
    Status {
        replicas: i32,
        ready_replicas: i32,
        available_replicas: i32,
        conditions: Vec<DeploymentCondition>,
    },
    PodStatus {
        pods: Vec<PodInfo>,
    },
    Logs {
        pod_name: String,
        logs: String,
    },
    Error {
        message: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentCondition {
    pub condition_type: String,
    pub status: String,
    pub reason: Option<String>,
    pub message: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PodInfo {
    pub name: String,
    pub phase: String,
    pub ready: bool,
    pub restarts: i32,
    pub node: Option<String>,
}

pub async fn watch_deployment(
    ws: WebSocketUpgrade,
    Path(deployment_id): Path<Uuid>,
    claims: Claims,
    State(database): State<Database>,
    State(kubernetes): State<Kubernetes>,
) -> Result<impl IntoResponse, AppError> {
    let user_id: Uuid = claims.sub;

    // Verify deployment ownership
    let deployment =
        DeploymentRepository::get_by_id(&database.pool, deployment_id, user_id).await?;

    Ok(ws.on_upgrade(move |socket| {
        handle_socket(
            socket,
            deployment.cluster_namespace,
            deployment.cluster_deployment_name,
            kubernetes,
        )
    }))
}

async fn handle_socket(
    socket: WebSocket,
    namespace: String,
    deployment_name: String,
    kubernetes: Kubernetes,
) {
    let (mut sender, mut receiver) = socket.split();

    // Spawn task to watch deployment status
    let deployment_name_clone = deployment_name.clone();
    let namespace_clone = namespace.clone();
    let kubernetes_clone = kubernetes.clone();

    let mut send_task = tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(5));

        loop {
            ticker.tick().await;

            // Get deployment status
            match get_deployment_status(
                &kubernetes_clone.client,
                &namespace_clone,
                &deployment_name_clone,
            )
            .await
            {
                Ok(msg) => {
                    let json = serde_json::to_string(&msg).unwrap();
                    if sender.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
                Err(e) => {
                    let error_msg = DeploymentMessage::Error {
                        message: format!("Failed to get status: {}", e),
                    };
                    let json = serde_json::to_string(&error_msg).unwrap();
                    let _ = sender.send(Message::Text(json.into())).await;
                }
            }

            // Get pod status
            match get_pod_status(
                &kubernetes_clone.client,
                &namespace_clone,
                &deployment_name_clone,
            )
            .await
            {
                Ok(msg) => {
                    let json = serde_json::to_string(&msg).unwrap();
                    if sender.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
                Err(e) => {
                    let error_msg = DeploymentMessage::Error {
                        message: format!("Failed to get pods: {}", e),
                    };
                    let json = serde_json::to_string(&error_msg).unwrap();
                    let _ = sender.send(Message::Text(json.into())).await;
                }
            }
        }
    });

    // Receive messages from client (e.g., request logs)
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(_text) = msg {
                // Handle client requests (e.g., stream logs for specific pod)
                // For now, just echo back
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }
}

async fn get_deployment_status(
    client: &kube::Client,
    namespace: &str,
    deployment_name: &str,
) -> Result<DeploymentMessage, Box<dyn std::error::Error>> {
    let deployments: Api<K8sDeployment> = Api::namespaced(client.clone(), namespace);
    let deployment = deployments.get(deployment_name).await?;

    let status = deployment.status.unwrap_or_default();
    let replicas = status.replicas.unwrap_or(0);
    let ready_replicas = status.ready_replicas.unwrap_or(0);
    let available_replicas = status.available_replicas.unwrap_or(0);

    let conditions = status
        .conditions
        .unwrap_or_default()
        .into_iter()
        .map(|c| DeploymentCondition {
            condition_type: c.type_,
            status: c.status,
            reason: c.reason,
            message: c.message,
        })
        .collect();

    Ok(DeploymentMessage::Status {
        replicas,
        ready_replicas,
        available_replicas,
        conditions,
    })
}

async fn get_pod_status(
    client: &kube::Client,
    namespace: &str,
    deployment_name: &str,
) -> Result<DeploymentMessage, Box<dyn std::error::Error>> {
    let pods: Api<Pod> = Api::namespaced(client.clone(), namespace);

    let label_selector = format!("app={}", deployment_name);
    let lp = ListParams::default().labels(&label_selector);

    let pod_list = pods.list(&lp).await?;

    let pod_infos: Vec<PodInfo> = pod_list
        .items
        .into_iter()
        .map(|pod| {
            let name = pod.metadata.name.unwrap_or_default();
            let status = pod.status.unwrap_or_default();
            let phase = status.phase.unwrap_or_else(|| "Unknown".to_string());

            let container_statuses = status.container_statuses.unwrap_or_default();
            let ready = container_statuses.iter().all(|cs| cs.ready);
            let restarts = container_statuses.iter().map(|cs| cs.restart_count).sum();

            let node = status.host_ip;

            PodInfo {
                name,
                phase,
                ready,
                restarts,
                node,
            }
        })
        .collect();

    Ok(DeploymentMessage::PodStatus { pods: pod_infos })
}
