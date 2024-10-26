use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::{AppError, AppState, CreateAgent, UpdateAgent};

/// List all agents in the chat
#[utoipa::path(
    get,
    path = "/api/chats/{id}/agents",
    params(
        ("id" = u64, Path, description = "Chat ID")
    ),
    responses(
        (status = 200, description = "List of agents", body = Vec<ChatAgent>)
    ),
    security(
        ("token" = [])
    )
)]
pub(crate) async fn list_agent_handler(
    Path(id): Path<u64>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let agents = state.list_agents(id as _).await?;
    Ok((StatusCode::OK, Json(agents)))
}

/// Create a new agent in the chat
#[utoipa::path(
    post,
    path = "/api/chats/{id}/agents",
    params(
        ("id" = u64, Path, description = "Chat ID")
    ),
    responses(
        (status = 201, description = "Agent created", body = ChatAgent)
    ),
    security(
        ("token" = [])
    )
)]
pub(crate) async fn create_agent_handler(
    Path(id): Path<u64>,
    State(state): State<AppState>,
    Json(agent): Json<CreateAgent>,
) -> Result<impl IntoResponse, AppError> {
    let agent = state.create_agent(agent, id).await?;
    Ok((StatusCode::CREATED, Json(agent)))
}

/// Update an agent in the chat
#[utoipa::path(
    patch,
    path = "/api/chats/{id}/agents/{agent_id}",
    params(
        ("id" = u64, Path, description = "Chat ID"),
        ("agent_id" = u64, Path, description = "Agent ID")
    ),
    responses(
        (status = 200, description = "Agent updated", body = ChatAgent),
        (status = 404, description = "Agent not found", body = ErrorOutput)
    ),
    security(
        ("token" = [])
    )
)]
pub(crate) async fn update_agent_handler(
    Path(id): Path<u64>,
    State(state): State<AppState>,
    Json(agent): Json<UpdateAgent>,
) -> Result<impl IntoResponse, AppError> {
    let agent = state.update_agent(agent, id as _).await?;
    Ok((StatusCode::OK, Json(agent)))
}
