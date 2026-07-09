mod db;
mod models;
mod repository;
mod seed;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use db::PgGuideRepository;
use models::{Area, OfflinePack};
use repository::{GuideRepository, RepositoryError};
use seed::SeedStore;
use std::{net::SocketAddr, sync::Arc};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    repository: Arc<dyn GuideRepository>,
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

    let args = std::env::args().collect::<Vec<_>>();

    if args.get(1).is_some_and(|arg| arg == "import-seed") {
        import_seed().await;
        return;
    }

    let state = AppState {
        repository: configure_repository().await,
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/areas", get(list_areas))
        .route("/api/v1/areas/:area_id", get(get_area))
        .route("/api/v1/offline-packs/areas/:area_id", get(get_area_pack))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let port = std::env::var("CLIMBAR_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8080);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::info!("listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind backend listener");

    axum::serve(listener, app)
        .await
        .expect("run backend server");
}

async fn configure_repository() -> Arc<dyn GuideRepository> {
    match std::env::var("CLIMBAR_DATABASE_URL") {
        Ok(database_url) => {
            let repository = PgGuideRepository::connect(&database_url)
                .await
                .expect("connect to Postgres database");
            tracing::info!("using Postgres guide repository");
            Arc::new(repository)
        }
        Err(_) => {
            tracing::warn!("CLIMBAR_DATABASE_URL is not set; using in-memory seed data");
            Arc::new(SeedStore::new())
        }
    }
}

async fn import_seed() {
    let database_url = std::env::var("CLIMBAR_DATABASE_URL")
        .expect("CLIMBAR_DATABASE_URL must be set to import seed data");
    let repository = PgGuideRepository::connect(&database_url)
        .await
        .expect("connect to Postgres database");
    let seed = SeedStore::new();

    repository
        .import_seed(&seed.areas_seed())
        .await
        .expect("import seed data");

    tracing::info!("seed data imported");
}

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn list_areas(State(state): State<AppState>) -> Result<Json<Vec<Area>>, StatusCode> {
    state
        .repository
        .areas()
        .await
        .map(Json)
        .map_err(status_from_repository_error)
}

async fn get_area(
    State(state): State<AppState>,
    Path(area_id): Path<Uuid>,
) -> Result<Json<Area>, StatusCode> {
    state
        .repository
        .area(area_id)
        .await
        .map_err(status_from_repository_error)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn get_area_pack(
    State(state): State<AppState>,
    Path(area_id): Path<Uuid>,
) -> Result<Json<OfflinePack>, StatusCode> {
    state
        .repository
        .offline_pack(area_id)
        .await
        .map_err(status_from_repository_error)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

fn status_from_repository_error(error: RepositoryError) -> StatusCode {
    tracing::error!(?error, "repository error");
    StatusCode::INTERNAL_SERVER_ERROR
}
