mod db;
mod models;
mod repository;
mod seed;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use db::PgGuideRepository;
use models::{
    Area, CalibrationReviewStatus, OfflinePack, Route, RouteArOverlay, RouteCalibrationCapture,
    Wall,
};
use repository::{GuideRepository, RepositoryError};
use seed::SeedStore;
use serde::Deserialize;
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
        .route("/api/v1/walls/:wall_id", get(get_wall))
        .route("/api/v1/routes/:route_id", get(get_route))
        .route("/api/v1/search", get(search_routes))
        .route("/api/v1/offline-packs/areas/:area_id", get(get_area_pack))
        .route(
            "/api/v1/admin/ar-calibration-captures",
            get(list_calibration_captures).post(create_calibration_capture),
        )
        .route(
            "/api/v1/admin/ar-calibration-captures/:capture_id/review",
            axum::routing::post(review_calibration_capture),
        )
        .route(
            "/api/v1/admin/ar-overlays/:overlay_id/apply-calibration/:capture_id",
            axum::routing::post(apply_calibration_capture),
        )
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = server_addr(
        std::env::var("CLIMBAR_HOST").ok(),
        std::env::var("CLIMBAR_PORT").ok(),
    );
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

async fn get_wall(
    State(state): State<AppState>,
    Path(wall_id): Path<Uuid>,
) -> Result<Json<Wall>, StatusCode> {
    state
        .repository
        .wall(wall_id)
        .await
        .map_err(status_from_repository_error)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn get_route(
    State(state): State<AppState>,
    Path(route_id): Path<Uuid>,
) -> Result<Json<Route>, StatusCode> {
    state
        .repository
        .route(route_id)
        .await
        .map_err(status_from_repository_error)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
}

async fn search_routes(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<Route>>, StatusCode> {
    state
        .repository
        .search(&query.q)
        .await
        .map(Json)
        .map_err(status_from_repository_error)
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

#[derive(Deserialize)]
struct CalibrationCaptureQuery {
    route_id: Option<Uuid>,
    overlay_id: Option<Uuid>,
}

async fn list_calibration_captures(
    State(state): State<AppState>,
    Query(query): Query<CalibrationCaptureQuery>,
) -> Result<Json<Vec<RouteCalibrationCapture>>, StatusCode> {
    state
        .repository
        .calibration_captures(query.route_id, query.overlay_id)
        .await
        .map(Json)
        .map_err(status_from_repository_error)
}

async fn create_calibration_capture(
    State(state): State<AppState>,
    Json(capture): Json<RouteCalibrationCapture>,
) -> Result<(StatusCode, Json<RouteCalibrationCapture>), StatusCode> {
    state
        .repository
        .create_calibration_capture(capture)
        .await
        .map(|capture| (StatusCode::CREATED, Json(capture)))
        .map_err(status_from_repository_error)
}

#[derive(Deserialize)]
struct ReviewCalibrationCaptureRequest {
    review_status: CalibrationReviewStatus,
    reviewer_notes: Option<String>,
}

async fn review_calibration_capture(
    State(state): State<AppState>,
    Path(capture_id): Path<Uuid>,
    Json(review): Json<ReviewCalibrationCaptureRequest>,
) -> Result<Json<RouteCalibrationCapture>, StatusCode> {
    state
        .repository
        .review_calibration_capture(capture_id, review.review_status, review.reviewer_notes)
        .await
        .map_err(status_from_repository_error)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn apply_calibration_capture(
    State(state): State<AppState>,
    Path((overlay_id, capture_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<RouteArOverlay>, StatusCode> {
    state
        .repository
        .apply_calibration_capture_to_overlay(overlay_id, capture_id)
        .await
        .map_err(status_from_repository_error)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

fn status_from_repository_error(error: RepositoryError) -> StatusCode {
    tracing::error!(?error, "repository error");
    StatusCode::INTERNAL_SERVER_ERROR
}

fn server_addr(host: Option<String>, port: Option<String>) -> SocketAddr {
    let host = host.unwrap_or_else(|| "127.0.0.1".to_string());
    let port = port
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8080);

    format!("{host}:{port}")
        .parse()
        .expect("CLIMBAR_HOST and CLIMBAR_PORT must form a valid socket address")
}

#[cfg(test)]
mod tests {
    use super::server_addr;

    #[test]
    fn server_addr_defaults_to_localhost() {
        assert_eq!(server_addr(None, None).to_string(), "127.0.0.1:8080");
    }

    #[test]
    fn server_addr_supports_network_binding() {
        assert_eq!(
            server_addr(Some("0.0.0.0".to_string()), Some("8081".to_string())).to_string(),
            "0.0.0.0:8081"
        );
    }
}
