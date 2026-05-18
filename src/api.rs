use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde_json::{json, Value};
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::RwLock;

#[derive(Default)]
pub struct RegisterState {
    pub registers: BTreeMap<String, u16>,
    pub last_updated: Option<String>,
    pub error: Option<String>,
}

pub type SharedState = Arc<RwLock<RegisterState>>;

pub fn build_router(state: SharedState) -> Router {
    Router::new()
        .route("/registers", get(get_all_registers))
        .route("/registers/:address", get(get_register))
        .with_state(state)
}

async fn get_all_registers(State(state): State<SharedState>) -> Json<Value> {
    let s = state.read().await;
    Json(json!({
        "registers": s.registers,
        "last_updated": s.last_updated,
        "error": s.error,
    }))
}

async fn get_register(
    State(state): State<SharedState>,
    Path(address): Path<u16>,
) -> impl IntoResponse {
    let s = state.read().await;
    let key = address.to_string();
    match s.registers.get(&key) {
        Some(&value) => Json(json!({
            "address": address,
            "value": value,
            "last_updated": s.last_updated,
        }))
        .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": format!("register {address} not found") })),
        )
            .into_response(),
    }
}
