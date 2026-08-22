use std::convert::Infallible;

use axum::{
    extract::{rejection::JsonRejection, State},
    response::sse::{Event, KeepAlive, Sse},
    Json,
};
use tokio_stream::iter;

use super::{
    error::ApiError,
    models::{CompletionResponse, GenerateRequest, GenerateResponse, TokenResponse},
};
use crate::{LocalGeneration, LocalNode};

pub(super) async fn generate(
    State(node): State<LocalNode>,
    request: Result<Json<GenerateRequest>, JsonRejection>,
) -> Result<Json<GenerateResponse>, ApiError> {
    let request = request
        .map_err(|_| ApiError::invalid_request())?
        .0
        .into_domain()?;
    let generation = execute(node, request).await?;
    Ok(Json(generation.into()))
}

pub(super) async fn stream(
    State(node): State<LocalNode>,
    request: Result<Json<GenerateRequest>, JsonRejection>,
) -> Result<Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>>, ApiError> {
    let request = request
        .map_err(|_| ApiError::invalid_request())?
        .0
        .into_domain()?;
    let generation = execute(node, request).await?;
    let events = stream_events(generation)?;
    Ok(Sse::new(iter(events.into_iter().map(Ok))).keep_alive(KeepAlive::default()))
}

async fn execute(
    node: LocalNode,
    request: synapseflow_domain::GenerationRequest,
) -> Result<LocalGeneration, ApiError> {
    tokio::task::spawn_blocking(move || node.execute(request))
        .await
        .map_err(|_| ApiError::execution_join_failed())?
        .map_err(ApiError::from)
}

fn stream_events(generation: LocalGeneration) -> Result<Vec<Event>, ApiError> {
    let session_id = generation.session_id.to_string();
    let mut events = generation
        .output
        .tokens
        .into_iter()
        .map(TokenResponse::from)
        .map(|token| {
            Event::default()
                .event("token")
                .json_data(token)
                .map_err(|_| ApiError::execution_join_failed())
        })
        .collect::<Result<Vec<_>, _>>()?;
    events.push(
        Event::default()
            .event("complete")
            .json_data(CompletionResponse::new(session_id))
            .map_err(|_| ApiError::execution_join_failed())?,
    );
    Ok(events)
}
