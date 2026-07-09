use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
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

impl fmt::Display for GradeSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::YosemiteDecimal => "yosemite_decimal",
            Self::Hueco => "hueco",
            Self::French => "french",
        })
    }
}

impl FromStr for GradeSystem {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "yosemite_decimal" => Ok(Self::YosemiteDecimal),
            "hueco" => Ok(Self::Hueco),
            "french" => Ok(Self::French),
            _ => Err(format!("unknown grade system: {value}")),
        }
    }
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

impl fmt::Display for RouteType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Sport => "sport",
            Self::Trad => "trad",
            Self::Boulder => "boulder",
            Self::Mixed => "mixed",
            Self::TopRope => "top_rope",
            Self::Aid => "aid",
            Self::Ice => "ice",
            Self::Alpine => "alpine",
        })
    }
}

impl FromStr for RouteType {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "sport" => Ok(Self::Sport),
            "trad" => Ok(Self::Trad),
            "boulder" => Ok(Self::Boulder),
            "mixed" => Ok(Self::Mixed),
            "top_rope" => Ok(Self::TopRope),
            "aid" => Ok(Self::Aid),
            "ice" => Ok(Self::Ice),
            "alpine" => Ok(Self::Alpine),
            _ => Err(format!("unknown route type: {value}")),
        }
    }
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

impl fmt::Display for MediaKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Photo => "photo",
            Self::Topo => "topo",
            Self::Video => "video",
        })
    }
}

impl FromStr for MediaKind {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "photo" => Ok(Self::Photo),
            "topo" => Ok(Self::Topo),
            "video" => Ok(Self::Video),
            _ => Err(format!("unknown media kind: {value}")),
        }
    }
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

impl fmt::Display for ArAnchorStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::ManualAlignment => "manual_alignment",
            Self::ReferenceImage => "reference_image",
            Self::WallPlaneAndBearing => "wall_plane_and_bearing",
        })
    }
}

impl FromStr for ArAnchorStrategy {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "manual_alignment" => Ok(Self::ManualAlignment),
            "reference_image" => Ok(Self::ReferenceImage),
            "wall_plane_and_bearing" => Ok(Self::WallPlaneAndBearing),
            _ => Err(format!("unknown AR anchor strategy: {value}")),
        }
    }
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

impl fmt::Display for OverlayConfidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Draft => "draft",
            Self::FieldTested => "field_tested",
            Self::Reviewed => "reviewed",
        })
    }
}

impl FromStr for OverlayConfidence {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "draft" => Ok(Self::Draft),
            "field_tested" => Ok(Self::FieldTested),
            "reviewed" => Ok(Self::Reviewed),
            _ => Err(format!("unknown overlay confidence: {value}")),
        }
    }
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
