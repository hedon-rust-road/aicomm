mod config;
mod error;
mod events;
mod extractors;
mod handlers;
mod openapi;

pub mod pb;

use anyhow::Context;
use axum::http::Method;
use axum::middleware::from_fn_with_state;
use axum::routing::post;
use axum::Router;
use chat_core::middlewares::{extract_user, set_layer, TokenVerify};
use chat_core::DecodingKey;
use clickhouse::Client;
use core::fmt;
use handlers::create_event_handler;
use openapi::OpenApiRouter as _;
use std::ops::Deref;
use std::sync::Arc;
use tokio::fs;
use tower_http::cors::{self, CorsLayer};

pub use config::AppConfig;
pub use config::*;
pub use error::*;
pub use events::*;

#[derive(Debug, Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

#[allow(unused)]
pub struct AppStateInner {
    pub(crate) config: AppConfig,
    pub(crate) dk: DecodingKey,
    pub(crate) client: Client,
}

pub async fn get_router(state: AppState) -> Result<Router, AppError> {
    let cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::PUT,
        ])
        .allow_origin(cors::Any)
        .allow_headers(cors::Any);

    let api = Router::new()
        .route("/event", post(create_event_handler))
        .layer(from_fn_with_state(state.clone(), extract_user::<AppState>))
        .layer(cors);

    let app = Router::new().openapi().nest("/api", api).with_state(state);

    Ok(set_layer(app))
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl TokenVerify for AppState {
    type Error = AppError;
    fn verify(&self, token: &str) -> Result<chat_core::User, Self::Error> {
        Ok(self.dk.verify(token)?)
    }
}

impl AppState {
    pub async fn try_new(config: AppConfig) -> Result<Self, AppError> {
        fs::create_dir_all(&config.server.base_dir)
            .await
            .context("create base_dir failed")?;

        let dk = DecodingKey::load(&config.auth.pk).context("load decoding key failed")?;
        let mut client = Client::default()
            .with_url(&config.server.db_url)
            .with_database(&config.server.db_name);
        if let Some(user) = config.server.db_user.as_ref() {
            client = client.with_user(user);
        }
        if let Some(password) = config.server.db_password.as_ref() {
            client = client.with_password(password);
        }
        Ok(Self {
            inner: Arc::new(AppStateInner { config, dk, client }),
        })
    }
}

impl fmt::Debug for AppStateInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppStateInner")
            .field("config", &self.config)
            .finish()
    }
}
