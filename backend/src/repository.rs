use crate::models::{
    Area, CalibrationReviewStatus, OfflinePack, Route, RouteArOverlay, RouteCalibrationCapture,
    Wall,
};
use async_trait::async_trait;
use std::{error::Error, fmt};
use uuid::Uuid;

pub type RepositoryResult<T> = Result<T, RepositoryError>;

#[derive(Debug)]
pub enum RepositoryError {
    Database(sqlx::Error),
    Decode(String),
}

impl fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(error) => write!(f, "database error: {error}"),
            Self::Decode(error) => write!(f, "decode error: {error}"),
        }
    }
}

impl Error for RepositoryError {}

impl From<sqlx::Error> for RepositoryError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}

#[async_trait]
pub trait GuideRepository: Send + Sync {
    async fn areas(&self) -> RepositoryResult<Vec<Area>>;
    async fn area(&self, area_id: Uuid) -> RepositoryResult<Option<Area>>;
    async fn wall(&self, wall_id: Uuid) -> RepositoryResult<Option<Wall>>;
    async fn route(&self, route_id: Uuid) -> RepositoryResult<Option<Route>>;
    async fn search(&self, query: &str) -> RepositoryResult<Vec<Route>>;
    async fn offline_pack(&self, area_id: Uuid) -> RepositoryResult<Option<OfflinePack>>;
    async fn create_area(&self, area: Area) -> RepositoryResult<Area>;
    async fn create_wall(&self, wall: Wall) -> RepositoryResult<Option<Wall>>;
    async fn create_route(&self, route: Route) -> RepositoryResult<Option<Route>>;
    async fn create_ar_overlay(
        &self,
        overlay: RouteArOverlay,
    ) -> RepositoryResult<Option<RouteArOverlay>>;
    async fn update_route(&self, route_id: Uuid, route: Route) -> RepositoryResult<Option<Route>>;
    async fn update_ar_overlay(
        &self,
        overlay_id: Uuid,
        overlay: RouteArOverlay,
    ) -> RepositoryResult<Option<RouteArOverlay>>;
    async fn create_calibration_capture(
        &self,
        capture: RouteCalibrationCapture,
    ) -> RepositoryResult<RouteCalibrationCapture>;
    async fn calibration_captures(
        &self,
        route_id: Option<Uuid>,
        overlay_id: Option<Uuid>,
    ) -> RepositoryResult<Vec<RouteCalibrationCapture>>;
    async fn review_calibration_capture(
        &self,
        capture_id: Uuid,
        review_status: CalibrationReviewStatus,
        reviewer_notes: Option<String>,
    ) -> RepositoryResult<Option<RouteCalibrationCapture>>;
    async fn apply_calibration_capture_to_overlay(
        &self,
        overlay_id: Uuid,
        capture_id: Uuid,
    ) -> RepositoryResult<Option<RouteArOverlay>>;
}
