mod auth;
mod db;
mod models;
mod repository;
mod seed;

use axum::http::HeaderMap;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use db::PgGuideRepository;
use models::{
    ArchivedGuideEntry, Area, CalibrationReviewStatus, MediaAsset, NearbyRoute, OfflinePack, Route,
    RouteArOverlay, RouteCalibrationCapture, Wall,
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
    admin_token: String,
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

    let (_, _, admin_token) = auth::configured_admin_credentials();
    let state = AppState {
        repository: configure_repository().await,
        admin_token,
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/areas", get(list_areas))
        .route("/api/v1/areas/:area_id", get(get_area))
        .route("/api/v1/walls/:wall_id", get(get_wall))
        .route("/api/v1/routes/:route_id", get(get_route))
        .route("/api/v1/search", get(search_routes))
        .route("/api/v1/nearby/routes", get(nearby_routes))
        .route("/api/v1/offline-packs/areas/:area_id", get(get_area_pack))
        .route("/api/v1/admin/auth/login", post(admin_login))
        .route(
            "/api/v1/admin/offline-packs/areas/:area_id/publish",
            post(publish_area_pack),
        )
        .route("/api/v1/admin/areas", post(create_area))
        .route("/api/v1/admin/walls", post(create_wall))
        .route("/api/v1/admin/routes", post(create_route))
        .route("/api/v1/admin/ar-overlays", post(create_ar_overlay))
        .route("/api/v1/admin/routes/:route_id", put(update_route))
        .route("/api/v1/admin/areas/:area_id", put(update_area))
        .route("/api/v1/admin/walls/:wall_id", put(update_wall))
        .route("/api/v1/admin/areas/:area_id/archive", post(archive_area))
        .route("/api/v1/admin/walls/:wall_id/archive", post(archive_wall))
        .route(
            "/api/v1/admin/routes/:route_id/archive",
            post(archive_route),
        )
        .route(
            "/api/v1/admin/ar-overlays/:overlay_id",
            put(update_ar_overlay),
        )
        .route("/api/v1/admin/media/:media_id", put(update_media))
        .route("/api/v1/admin/routes/:route_id/media", post(create_media))
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
        .route("/api/v1/admin/archived", get(list_archived))
        .route(
            "/api/v1/admin/archived/:entity_id/restore",
            post(restore_entity),
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

    for area in seed.areas_seed() {
        repository
            .publish_offline_pack(area.id)
            .await
            .expect("publish seed pack");
    }

    tracing::info!("seed data imported");
}

async fn health() -> impl IntoResponse {
    Json(health_payload())
}

fn health_payload() -> serde_json::Value {
    serde_json::json!({
        "status": "ok",
        "service": "climbar-backend",
        "version": env!("CARGO_PKG_VERSION"),
    })
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

#[derive(Deserialize)]
struct NearbyQuery {
    latitude: f64,
    longitude: f64,
    radius_meters: Option<f64>,
}

async fn nearby_routes(
    State(state): State<AppState>,
    Query(query): Query<NearbyQuery>,
) -> Result<Json<Vec<NearbyRoute>>, StatusCode> {
    state
        .repository
        .nearby_routes(
            query.latitude,
            query.longitude,
            query.radius_meters.unwrap_or(2_000.0).clamp(1.0, 50_000.0),
        )
        .await
        .map(Json)
        .map_err(status_from_repository_error)
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
struct AdminLoginRequest {
    email: String,
    password: String,
}

async fn admin_login(
    Json(login): Json<AdminLoginRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let (email, password, token) = auth::configured_admin_credentials();
    if login.email != email || login.password != password {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(Json(serde_json::json!({ "token": token, "email": email })))
}

async fn publish_area_pack(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(area_id): Path<Uuid>,
) -> Result<Json<OfflinePack>, StatusCode> {
    auth::authorize(&headers, &state)?;
    state
        .repository
        .publish_offline_pack(area_id)
        .await
        .map_err(status_from_repository_error)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn create_area(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(area): Json<Area>,
) -> Result<(StatusCode, Json<Area>), StatusCode> {
    auth::authorize(&headers, &state)?;
    state
        .repository
        .create_area(area)
        .await
        .map(|area| (StatusCode::CREATED, Json(area)))
        .map_err(status_from_repository_error)
}

async fn create_wall(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(wall): Json<Wall>,
) -> Result<(StatusCode, Json<Wall>), StatusCode> {
    auth::authorize(&headers, &state)?;
    state
        .repository
        .create_wall(wall)
        .await
        .map_err(status_from_repository_error)?
        .map(|wall| (StatusCode::CREATED, Json(wall)))
        .ok_or(StatusCode::NOT_FOUND)
}

async fn update_area(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(area_id): Path<Uuid>,
    Json(area): Json<Area>,
) -> Result<Json<Area>, StatusCode> {
    auth::authorize(&headers, &state)?;
    state
        .repository
        .update_area(area_id, area)
        .await
        .map_err(status_from_repository_error)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn update_wall(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(wall_id): Path<Uuid>,
    Json(wall): Json<Wall>,
) -> Result<Json<Wall>, StatusCode> {
    auth::authorize(&headers, &state)?;
    state
        .repository
        .update_wall(wall_id, wall)
        .await
        .map_err(status_from_repository_error)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn archive_area(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(area_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    auth::authorize(&headers, &state)?;
    if state
        .repository
        .archive_area(area_id)
        .await
        .map_err(status_from_repository_error)?
    {
        state
            .repository
            .publish_offline_pack(area_id)
            .await
            .map_err(status_from_repository_error)?;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn archive_wall(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(wall_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    auth::authorize(&headers, &state)?;
    let area_id = state
        .repository
        .wall(wall_id)
        .await
        .map_err(status_from_repository_error)?
        .map(|wall| wall.area_id);
    if state
        .repository
        .archive_wall(wall_id)
        .await
        .map_err(status_from_repository_error)?
    {
        if let Some(area_id) = area_id {
            state
                .repository
                .publish_offline_pack(area_id)
                .await
                .map_err(status_from_repository_error)?;
        }
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn archive_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(route_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    auth::authorize(&headers, &state)?;
    let area_id = if let Some(route) = state
        .repository
        .route(route_id)
        .await
        .map_err(status_from_repository_error)?
    {
        state
            .repository
            .wall(route.wall_id)
            .await
            .map_err(status_from_repository_error)?
            .map(|wall| wall.area_id)
    } else {
        None
    };
    if state
        .repository
        .archive_route(route_id)
        .await
        .map_err(status_from_repository_error)?
    {
        if let Some(area_id) = area_id {
            state
                .repository
                .publish_offline_pack(area_id)
                .await
                .map_err(status_from_repository_error)?;
        }
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn list_archived(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<ArchivedGuideEntry>>, StatusCode> {
    auth::authorize(&headers, &state)?;
    state
        .repository
        .archived_entities()
        .await
        .map(Json)
        .map_err(status_from_repository_error)
}

async fn restore_entity(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(entity_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    auth::authorize(&headers, &state)?;
    let entity_type = state
        .repository
        .archived_entities()
        .await
        .map_err(status_from_repository_error)?
        .into_iter()
        .find(|entry| entry.id == entity_id)
        .map(|entry| entry.entity_type);
    if state
        .repository
        .restore_entity(entity_id)
        .await
        .map_err(status_from_repository_error)?
    {
        let area_id = match entity_type.as_deref() {
            Some("area") => Some(entity_id),
            Some("wall") => state
                .repository
                .wall(entity_id)
                .await
                .map_err(status_from_repository_error)?
                .map(|wall| wall.area_id),
            Some("route") => {
                if let Some(route) = state
                    .repository
                    .route(entity_id)
                    .await
                    .map_err(status_from_repository_error)?
                {
                    state
                        .repository
                        .wall(route.wall_id)
                        .await
                        .map_err(status_from_repository_error)?
                        .map(|wall| wall.area_id)
                } else {
                    None
                }
            }
            _ => None,
        };
        if let Some(area_id) = area_id {
            state
                .repository
                .publish_offline_pack(area_id)
                .await
                .map_err(status_from_repository_error)?;
        }
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn create_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(route): Json<Route>,
) -> Result<(StatusCode, Json<Route>), StatusCode> {
    auth::authorize(&headers, &state)?;
    state
        .repository
        .create_route(route)
        .await
        .map_err(status_from_repository_error)?
        .map(|route| (StatusCode::CREATED, Json(route)))
        .ok_or(StatusCode::NOT_FOUND)
}

async fn create_ar_overlay(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(overlay): Json<RouteArOverlay>,
) -> Result<(StatusCode, Json<RouteArOverlay>), StatusCode> {
    auth::authorize(&headers, &state)?;
    state
        .repository
        .create_ar_overlay(overlay)
        .await
        .map_err(status_from_repository_error)?
        .map(|overlay| (StatusCode::CREATED, Json(overlay)))
        .ok_or(StatusCode::NOT_FOUND)
}

async fn update_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(route_id): Path<Uuid>,
    Json(route): Json<Route>,
) -> Result<Json<Route>, StatusCode> {
    auth::authorize(&headers, &state)?;
    state
        .repository
        .update_route(route_id, route)
        .await
        .map_err(status_from_repository_error)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn update_ar_overlay(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(overlay_id): Path<Uuid>,
    Json(overlay): Json<RouteArOverlay>,
) -> Result<Json<RouteArOverlay>, StatusCode> {
    auth::authorize(&headers, &state)?;
    state
        .repository
        .update_ar_overlay(overlay_id, overlay)
        .await
        .map_err(status_from_repository_error)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn update_media(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(media_id): Path<Uuid>,
    Json(media): Json<MediaAsset>,
) -> Result<Json<MediaAsset>, StatusCode> {
    auth::authorize(&headers, &state)?;
    state
        .repository
        .update_media(media_id, media)
        .await
        .map_err(status_from_repository_error)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn create_media(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(route_id): Path<Uuid>,
    Json(media): Json<MediaAsset>,
) -> Result<(StatusCode, Json<MediaAsset>), StatusCode> {
    auth::authorize(&headers, &state)?;
    state
        .repository
        .create_media(route_id, media)
        .await
        .map_err(status_from_repository_error)?
        .map(|media| (StatusCode::CREATED, Json(media)))
        .ok_or(StatusCode::NOT_FOUND)
}

#[derive(Deserialize)]
struct CalibrationCaptureQuery {
    route_id: Option<Uuid>,
    overlay_id: Option<Uuid>,
}

async fn list_calibration_captures(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<CalibrationCaptureQuery>,
) -> Result<Json<Vec<RouteCalibrationCapture>>, StatusCode> {
    auth::authorize(&headers, &state)?;
    state
        .repository
        .calibration_captures(query.route_id, query.overlay_id)
        .await
        .map(Json)
        .map_err(status_from_repository_error)
}

async fn create_calibration_capture(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(capture): Json<RouteCalibrationCapture>,
) -> Result<(StatusCode, Json<RouteCalibrationCapture>), StatusCode> {
    auth::authorize(&headers, &state)?;
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
    headers: HeaderMap,
    Path(capture_id): Path<Uuid>,
    Json(review): Json<ReviewCalibrationCaptureRequest>,
) -> Result<Json<RouteCalibrationCapture>, StatusCode> {
    auth::authorize(&headers, &state)?;
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
    headers: HeaderMap,
    Path((overlay_id, capture_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<RouteArOverlay>, StatusCode> {
    auth::authorize(&headers, &state)?;
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
    use super::{health_payload, server_addr};

    #[test]
    fn health_payload_identifies_the_running_service() {
        let payload = health_payload();

        assert_eq!(payload["status"], "ok");
        assert_eq!(payload["service"], "climbar-backend");
        assert_eq!(payload["version"], env!("CARGO_PKG_VERSION"));
    }

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
