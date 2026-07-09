use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeoPoint {
    pub latitude: f64,
    pub longitude: f64,
    pub elevation_meters: Option<f64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Area {
    pub id: Uuid,
    pub parent_area_id: Option<Uuid>,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub access_notes: Option<String>,
    pub location: GeoPoint,
    pub walls: Vec<Wall>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Wall {
    pub id: Uuid,
    pub area_id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub approach_notes: Option<String>,
    pub aspect: Option<String>,
    pub location: GeoPoint,
    pub routes: Vec<Route>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Route {
    pub id: Uuid,
    pub wall_id: Uuid,
    pub name: String,
    pub slug: String,
    pub grade: String,
    pub grade_system: GradeSystem,
    pub route_types: Vec<RouteType>,
    pub length_feet: Option<u16>,
    pub pitches: Option<u8>,
    pub stars_average: Option<f32>,
    pub rating_votes: u32,
    pub first_ascent: Option<String>,
    pub description: String,
    pub location_notes: String,
    pub protection_notes: Option<String>,
    pub safety_notes: Option<String>,
    pub location: GeoPoint,
    pub media: Vec<MediaAsset>,
    pub ar_overlays: Vec<RouteArOverlay>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GradeSystem {
    YosemiteDecimal,
    Hueco,
    French,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteType {
    Sport,
    Trad,
    Boulder,
    Mixed,
    TopRope,
    Aid,
    Ice,
    Alpine,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MediaAsset {
    pub id: Uuid,
    pub kind: MediaKind,
    pub title: String,
    pub url: String,
    pub offline_path: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaKind {
    Photo,
    Topo,
    Video,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RouteArOverlay {
    pub id: Uuid,
    pub route_id: Uuid,
    pub version: u32,
    pub anchor_strategy: ArAnchorStrategy,
    pub gps_hint: GeoPoint,
    pub compass_bearing_degrees: Option<f32>,
    pub wall_plane: Option<WallPlaneEstimate>,
    pub route_trace: RouteTrace,
    pub confidence: OverlayConfidence,
    pub reviewed_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArAnchorStrategy {
    ManualAlignment,
    ReferenceImage,
    WallPlaneAndBearing,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WallPlaneEstimate {
    pub normal: [f32; 3],
    pub center: [f32; 3],
    pub width_meters: f32,
    pub height_meters: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RouteTrace {
    pub coordinate_space: TraceCoordinateSpace,
    pub points: Vec<TracePoint>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceCoordinateSpace {
    NormalizedWallImage,
    LocalWallMeters,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TracePoint {
    pub x: f32,
    pub y: f32,
    pub z: Option<f32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OverlayConfidence {
    Draft,
    FieldTested,
    Reviewed,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OfflinePack {
    pub id: Uuid,
    pub area_id: Uuid,
    pub version: u32,
    pub generated_at: DateTime<Utc>,
    pub areas: Vec<Area>,
    pub assets: Vec<MediaAsset>,
}
