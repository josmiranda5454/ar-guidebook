mod models;
mod seed;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use models::{Area, OfflinePack};
use seed::SeedStore;
use std::{net::SocketAddr, sync::Arc};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    store: Arc<SeedStore>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "climbar_backend=debug,tower_http=debug,axum=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = AppState {
        store: Arc::new(SeedStore::new()),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/areas", get(list_areas))
        .route("/api/v1/areas/:area_id", get(get_area))
        .route("/api/v1/offline-packs/areas/:area_id", get(get_area_pack))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::info!("listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind backend listener");

    axum::serve(listener, app)
        .await
        .expect("run backend server");
}

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn list_areas(State(state): State<AppState>) -> Json<Vec<Area>> {
    Json(state.store.areas())
}

async fn get_area(
    State(state): State<AppState>,
    Path(area_id): Path<Uuid>,
) -> Result<Json<Area>, StatusCode> {
    state
        .store
        .area(area_id)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn get_area_pack(
    State(state): State<AppState>,
    Path(area_id): Path<Uuid>,
) -> Result<Json<OfflinePack>, StatusCode> {
    state
        .store
        .offline_pack(area_id)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}
