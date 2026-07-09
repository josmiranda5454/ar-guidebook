use crate::models::{
    ArAnchorStrategy, Area, GeoPoint, GradeSystem, MediaAsset, MediaKind, OfflinePack,
    OverlayConfidence, Route, RouteArOverlay, RouteTrace, RouteType, TraceCoordinateSpace,
    TracePoint, Wall, WallPlaneEstimate,
};
use chrono::Utc;
use uuid::Uuid;

pub struct SeedStore {
    areas: Vec<Area>,
}

impl SeedStore {
    pub fn new() -> Self {
        let area_id = Uuid::from_u128(0xaaaaaaaa_aaaa_aaaa_aaaa_aaaaaaaaaaaa);
        let wall_id = Uuid::from_u128(0xbbbbbbbb_bbbb_bbbb_bbbb_bbbbbbbbbbbb);
        let route_id = Uuid::from_u128(0xcccccccc_cccc_cccc_cccc_cccccccccccc);
        let overlay_id = Uuid::from_u128(0xdddddddd_dddd_dddd_dddd_dddddddddddd);
        let photo_id = Uuid::from_u128(0xeeeeeeee_eeee_eeee_eeee_eeeeeeeeeeee);

        let location = GeoPoint {
            latitude: 34.0103,
            longitude: -116.1669,
            elevation_meters: Some(1280.0),
        };

        let photo = MediaAsset {
            id: photo_id,
            kind: MediaKind::Photo,
            title: "Route overview".to_string(),
            url: "https://example.invalid/assets/twinkie-style-overview.jpg".to_string(),
            offline_path: Some("assets/route-overview.jpg".to_string()),
        };

        let route = Route {
            id: route_id,
            wall_id,
            name: "Sample Arete".to_string(),
            slug: "sample-arete".to_string(),
            grade: "5.8".to_string(),
            grade_system: GradeSystem::YosemiteDecimal,
            route_types: vec![RouteType::Trad],
            length_feet: Some(80),
            pitches: Some(1),
            stars_average: Some(3.4),
            rating_votes: 12,
            first_ascent: Some("Unknown".to_string()),
            description: "Follow positive edges up the clean arete to a comfortable stance."
                .to_string(),
            location_notes: "Starts on the right side of the wall below the obvious arete."
                .to_string(),
            protection_notes: Some("Single rack to 2 inches; bolted anchor.".to_string()),
            safety_notes: Some(
                "Check seasonal access and rock quality before climbing.".to_string(),
            ),
            location: location.clone(),
            media: vec![photo.clone()],
            ar_overlays: vec![RouteArOverlay {
                id: overlay_id,
                route_id,
                version: 1,
                anchor_strategy: ArAnchorStrategy::ManualAlignment,
                gps_hint: location.clone(),
                compass_bearing_degrees: Some(235.0),
                wall_plane: Some(WallPlaneEstimate {
                    normal: [0.0, 0.0, 1.0],
                    center: [0.0, 2.0, -4.0],
                    width_meters: 9.0,
                    height_meters: 18.0,
                }),
                route_trace: RouteTrace {
                    coordinate_space: TraceCoordinateSpace::NormalizedWallImage,
                    points: vec![
                        TracePoint {
                            x: 0.48,
                            y: 0.95,
                            z: None,
                        },
                        TracePoint {
                            x: 0.52,
                            y: 0.72,
                            z: None,
                        },
                        TracePoint {
                            x: 0.57,
                            y: 0.48,
                            z: None,
                        },
                        TracePoint {
                            x: 0.61,
                            y: 0.20,
                            z: None,
                        },
                    ],
                },
                confidence: OverlayConfidence::Draft,
                reviewed_at: None,
            }],
        };

        let wall = Wall {
            id: wall_id,
            area_id,
            name: "Demo Wall".to_string(),
            slug: "demo-wall".to_string(),
            description: "A compact wall used to validate route hierarchy and AR overlays."
                .to_string(),
            approach_notes: Some("Short walk from the signed trailhead.".to_string()),
            aspect: Some("Southwest".to_string()),
            location: location.clone(),
            routes: vec![route],
        };

        let area = Area {
            id: area_id,
            parent_area_id: None,
            name: "Demo Climbing Area".to_string(),
            slug: "demo-climbing-area".to_string(),
            description: "Seed area for ClimbAR development.".to_string(),
            access_notes: Some("Respect closures and local land-manager guidance.".to_string()),
            location,
            walls: vec![wall],
        };

        Self { areas: vec![area] }
    }

    pub fn areas(&self) -> Vec<Area> {
        self.areas.clone()
    }

    pub fn area(&self, area_id: Uuid) -> Option<Area> {
        self.areas.iter().find(|area| area.id == area_id).cloned()
    }

    pub fn offline_pack(&self, area_id: Uuid) -> Option<OfflinePack> {
        let area = self.area(area_id)?;
        let assets = area
            .walls
            .iter()
            .flat_map(|wall| wall.routes.iter())
            .flat_map(|route| route.media.iter().cloned())
            .collect();

        Some(OfflinePack {
            id: Uuid::new_v4(),
            area_id,
            version: 1,
            generated_at: Utc::now(),
            areas: vec![area],
            assets,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::SeedStore;

    #[test]
    fn seed_data_has_area_wall_route_hierarchy() {
        let store = SeedStore::new();
        let areas = store.areas();

        assert_eq!(areas.len(), 1);
        assert_eq!(areas[0].walls.len(), 1);
        assert_eq!(areas[0].walls[0].routes.len(), 1);
        assert_eq!(areas[0].walls[0].routes[0].ar_overlays.len(), 1);
    }

    #[test]
    fn offline_pack_contains_area_and_assets() {
        let store = SeedStore::new();
        let area = store.areas().remove(0);
        let pack = store.offline_pack(area.id).expect("area pack");

        assert_eq!(pack.area_id, area.id);
        assert_eq!(pack.version, 1);
        assert_eq!(pack.areas.len(), 1);
        assert_eq!(pack.assets.len(), 1);
    }
}
