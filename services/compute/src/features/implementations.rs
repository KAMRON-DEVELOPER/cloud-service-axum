use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use shared::{
    services::database::Database,
    utilities::{config::Config, errors::AppError, jwt::Claims},
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    features::{
        repository::{DeploymentRepository, ProjectRepository},
        schemas::{
            CreateDeploymentRequest, CreateProjectRequest, DeploymentResponse, ListResponse,
            MessageResponse, ProjectResponse, ScaleDeploymentRequest,
        },
    },
    services::{build_kubernetes::Kubernetes, kubernetes::DeploymentService},
};

// ============================================
// PROJECT HANDLERS
// ============================================

pub async fn get_projects(
    claims: Claims,
    State(database): State<Database>,
) -> Result<impl IntoResponse, AppError> {
    let user_id: Uuid = claims.sub;

    let projects = ProjectRepository::get_all_by_user_id(&database.pool, user_id).await?;

    let response: Vec<ProjectResponse> = projects
        .into_iter()
        .map(|p| ProjectResponse {
            id: p.id,
            name: p.name,
            description: p.description,
            created_at: p.created_at,
            updated_at: p.updated_at,
        })
        .collect();

    Ok(Json(ListResponse {
        total: response.len(),
        data: response,
    }))
}

pub async fn get_project(
    claims: Claims,
    Path(project_id): Path<Uuid>,
    State(database): State<Database>,
) -> Result<impl IntoResponse, AppError> {
    let user_id: Uuid = claims.sub;

    let project = ProjectRepository::get_by_id(&database.pool, project_id, user_id).await?;

    Ok(Json(ProjectResponse {
        id: project.id,
        name: project.name,
        description: project.description,
        created_at: project.created_at,
        updated_at: project.updated_at,
    }))
}

pub async fn create_project(
    claims: Claims,
    State(database): State<Database>,
    Json(req): Json<CreateProjectRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.validate()?;

    let user_id: Uuid = claims.sub;

    let project = ProjectRepository::create(
        &database.pool,
        user_id,
        &req.name,
        req.description.as_deref(),
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(ProjectResponse {
            id: project.id,
            name: project.name,
            description: project.description,
            created_at: project.created_at,
            updated_at: project.updated_at,
        }),
    ))
}

pub async fn update_project(
    claims: Claims,
    Path(project_id): Path<Uuid>,
    State(database): State<Database>,
    Json(req): Json<CreateProjectRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.validate()?;

    let user_id: Uuid = claims.sub;

    let project = ProjectRepository::update(
        &database.pool,
        project_id,
        user_id,
        Some(req.name.as_str()),
        req.description.as_deref(),
    )
    .await?;

    Ok(Json(ProjectResponse {
        id: project.id,
        name: project.name,
        description: project.description,
        created_at: project.created_at,
        updated_at: project.updated_at,
    }))
}

pub async fn delete_project(
    claims: Claims,
    Path(project_id): Path<Uuid>,
    State(database): State<Database>,
) -> Result<impl IntoResponse, AppError> {
    let user_id: Uuid = claims.sub;

    ProjectRepository::delete(&database.pool, project_id, user_id).await?;

    Ok((
        StatusCode::OK,
        Json(MessageResponse::new("Project deleted successfully")),
    ))
}

// ============================================
// DEPLOYMENT HANDLERS
// ============================================

pub async fn get_deployments(
    claims: Claims,
    Path(project_id): Path<Uuid>,
    State(database): State<Database>,
) -> Result<impl IntoResponse, AppError> {
    let user_id: Uuid = claims.sub;

    let deployments =
        DeploymentRepository::get_all_by_project(&database.pool, project_id, user_id).await?;

    let response: Vec<DeploymentResponse> = deployments
        .into_iter()
        .map(|d| {
            let resources = serde_json::from_value(d.resources).unwrap_or_default();
            DeploymentResponse {
                id: d.id,
                project_id: d.project_id,
                name: d.name,
                image: d.image,
                status: d.status,
                replicas: d.replicas,
                resources,
                external_url: None,
                created_at: d.created_at,
                updated_at: d.updated_at,
            }
        })
        .collect();

    Ok(Json(ListResponse {
        total: response.len(),
        data: response,
    }))
}

pub async fn get_deployment(
    claims: Claims,
    Path(deployment_id): Path<Uuid>,
    State(database): State<Database>,
) -> Result<impl IntoResponse, AppError> {
    let user_id: Uuid = claims.sub;

    let detail = DeploymentService::get_detail(&database.pool, deployment_id, user_id).await?;

    Ok(Json(detail))
}

pub async fn create_deployment(
    claims: Claims,
    Path(project_id): Path<Uuid>,
    State(database): State<Database>,
    State(kubernetes): State<Kubernetes>,
    State(config): State<Config>,
    Json(req): Json<CreateDeploymentRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.validate()?;

    let user_id: Uuid = claims.sub;

    // Verify project ownership
    ProjectRepository::get_by_id(&database.pool, project_id, user_id).await?;

    let deployment = DeploymentService::create(
        &database.pool,
        &kubernetes.client,
        &config.k8s_encryption_key,
        user_id,
        project_id,
        &config.base_domain,
        req,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(deployment)))
}

pub async fn scale_deployment(
    claims: Claims,
    Path(deployment_id): Path<Uuid>,
    State(database): State<Database>,
    State(kubernetes): State<Kubernetes>,
    Json(req): Json<ScaleDeploymentRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.validate()?;

    let user_id: Uuid = claims.sub;

    let deployment = DeploymentService::scale(
        &database.pool,
        &kubernetes.client,
        deployment_id,
        user_id,
        req.replicas,
    )
    .await?;

    Ok(Json(deployment))
}

pub async fn delete_deployment(
    claims: Claims,
    Path(deployment_id): Path<Uuid>,
    State(database): State<Database>,
    State(kubernetes): State<Kubernetes>,
) -> Result<impl IntoResponse, AppError> {
    let user_id: Uuid = claims.sub;

    DeploymentService::delete(&database.pool, &kubernetes.client, deployment_id, user_id).await?;

    Ok((
        StatusCode::OK,
        Json(MessageResponse::new("Deployment deleted successfully")),
    ))
}
