use crate::{
    services::{database::Database, redis::Redis},
    utilities::config::Config,
};
use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use object_store::{aws::AmazonS3, gcp::GoogleCloudStorage};
use qdrant_client::Qdrant;
use reqwest::Client;
use rustls::ClientConfig;

#[derive(Clone)]
pub struct AppState {
    pub rustls_config: ClientConfig,
    pub kubernetes: Kubernetes,
    pub database: Database,
    pub redis: Redis,
    pub qdrant: Qdrant,
    pub config: Config,
    pub key: Key,
    pub google_oauth_client: GoogleOAuthClient,
    pub github_oauth_client: GithubOAuthClient,
    pub http_client: Client,
    pub s3: AmazonS3,
    pub gcs: GoogleCloudStorage,
}

impl FromRef<AppState> for Kubernetes {
    fn from_ref(state: &AppState) -> Self {
        state.kubernetes.clone()
    }
}

impl FromRef<AppState> for Database {
    fn from_ref(state: &AppState) -> Self {
        state.database.clone()
    }
}

impl FromRef<AppState> for Redis {
    fn from_ref(state: &AppState) -> Self {
        state.redis.clone()
    }
}

impl FromRef<AppState> for Qdrant {
    fn from_ref(state: &AppState) -> Self {
        state.qdrant.clone()
    }
}

impl FromRef<AppState> for Config {
    fn from_ref(state: &AppState) -> Self {
        state.config.clone()
    }
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.key.clone()
    }
}

impl FromRef<AppState> for GoogleOAuthClient {
    fn from_ref(state: &AppState) -> Self {
        state.google_oauth_client.clone()
    }
}

impl FromRef<AppState> for GithubOAuthClient {
    fn from_ref(state: &AppState) -> Self {
        state.github_oauth_client.clone()
    }
}

impl FromRef<AppState> for Client {
    fn from_ref(state: &AppState) -> Self {
        state.http_client.clone()
    }
}

impl FromRef<AppState> for AmazonS3 {
    fn from_ref(state: &AppState) -> Self {
        state.s3.clone()
    }
}

impl FromRef<AppState> for GoogleCloudStorage {
    fn from_ref(state: &AppState) -> Self {
        state.gcs.clone()
    }
}
