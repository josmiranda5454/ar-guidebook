use crate::models::{
    ArAnchorStrategy, Area, CalibrationReviewStatus, GeoPoint, GradeSystem, MediaAsset, MediaKind,
    OfflinePack, OverlayConfidence, Route, RouteArOverlay, RouteCalibrationCapture, RouteTrace,
    RouteType, TraceCoordinateSpace, TracePoint, Wall, WallPlaneEstimate,
};
use crate::repository::{GuideRepository, RepositoryResult};
use async_trait::async_trait;
use chrono::Utc;
use std::sync::Mutex;
use uuid::Uuid;

pub struct SeedStore {
    areas: Mutex<Vec<Area>>,
    calibration_captures: Mutex<Vec<RouteCalibrationCapture>>,
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
                default_alignment: None,
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

        Self {
            areas: Mutex::new(vec![area]),
            calibration_captures: Mutex::new(Vec::new()),
        }
    }

    pub fn areas_seed(&self) -> Vec<Area> {
        self.areas.lock().expect("seed area store lock").clone()
    }

    pub fn area_seed(&self, area_id: Uuid) -> Option<Area> {
        self.areas
            .lock()
            .expect("seed area store lock")
            .iter()
            .find(|area| area.id == area_id)
            .cloned()
    }

    pub fn offline_pack_seed(&self, area_id: Uuid) -> Option<OfflinePack> {
        let area = self.area_seed(area_id)?;
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

    pub fn wall_seed(&self, wall_id: Uuid) -> Option<Wall> {
        self.areas
            .lock()
            .expect("seed area store lock")
            .iter()
            .flat_map(|area| area.walls.iter())
            .find(|wall| wall.id == wall_id)
            .cloned()
    }

    pub fn route_seed(&self, route_id: Uuid) -> Option<Route> {
        self.areas
            .lock()
            .expect("seed area store lock")
            .iter()
            .flat_map(|area| area.walls.iter())
            .flat_map(|wall| wall.routes.iter())
            .find(|route| route.id == route_id)
            .cloned()
    }

    pub fn search_seed(&self, query: &str) -> Vec<Route> {
        let normalized_query = query.trim().to_lowercase();
        if normalized_query.is_empty() {
            return Vec::new();
        }

        self.areas
            .lock()
            .expect("seed area store lock")
            .iter()
            .flat_map(|area| area.walls.iter())
            .flat_map(|wall| wall.routes.iter())
            .filter(|route| {
                route.name.to_lowercase().contains(&normalized_query)
                    || route.grade.to_lowercase().contains(&normalized_query)
                    || route.description.to_lowercase().contains(&normalized_query)
            })
            .cloned()
            .collect()
    }
}

#[async_trait]
impl GuideRepository for SeedStore {
    async fn areas(&self) -> RepositoryResult<Vec<Area>> {
        Ok(self.areas_seed())
    }

    async fn area(&self, area_id: Uuid) -> RepositoryResult<Option<Area>> {
        Ok(self.area_seed(area_id))
    }

    async fn wall(&self, wall_id: Uuid) -> RepositoryResult<Option<Wall>> {
        Ok(self.wall_seed(wall_id))
    }

    async fn route(&self, route_id: Uuid) -> RepositoryResult<Option<Route>> {
        Ok(self.route_seed(route_id))
    }

    async fn search(&self, query: &str) -> RepositoryResult<Vec<Route>> {
        Ok(self.search_seed(query))
    }

    async fn offline_pack(&self, area_id: Uuid) -> RepositoryResult<Option<OfflinePack>> {
        Ok(self.offline_pack_seed(area_id))
    }

    async fn create_calibration_capture(
        &self,
        capture: RouteCalibrationCapture,
    ) -> RepositoryResult<RouteCalibrationCapture> {
        self.calibration_captures
            .lock()
            .expect("calibration capture store lock")
            .push(capture.clone());
        Ok(capture)
    }

    async fn calibration_captures(
        &self,
        route_id: Option<Uuid>,
        overlay_id: Option<Uuid>,
    ) -> RepositoryResult<Vec<RouteCalibrationCapture>> {
        Ok(self
            .calibration_captures
            .lock()
            .expect("calibration capture store lock")
            .iter()
            .filter(|capture| route_id.is_none_or(|route_id| capture.route_id == route_id))
            .filter(|capture| overlay_id.is_none_or(|overlay_id| capture.overlay_id == overlay_id))
            .cloned()
            .collect())
    }

    async fn review_calibration_capture(
        &self,
        capture_id: Uuid,
        review_status: CalibrationReviewStatus,
        reviewer_notes: Option<String>,
    ) -> RepositoryResult<Option<RouteCalibrationCapture>> {
        let mut captures = self
            .calibration_captures
            .lock()
            .expect("calibration capture store lock");

        let Some(capture) = captures.iter_mut().find(|capture| capture.id == capture_id) else {
            return Ok(None);
        };

        capture.review_status = review_status;
        capture.reviewer_notes = reviewer_notes;
        capture.reviewed_at = Some(Utc::now());

        Ok(Some(capture.clone()))
    }

    async fn apply_calibration_capture_to_overlay(
        &self,
        overlay_id: Uuid,
        capture_id: Uuid,
    ) -> RepositoryResult<Option<RouteArOverlay>> {
        let alignment = {
            let mut captures = self
                .calibration_captures
                .lock()
                .expect("calibration capture store lock");

            let Some(capture) = captures
                .iter_mut()
                .find(|capture| capture.id == capture_id && capture.overlay_id == overlay_id)
            else {
                return Ok(None);
            };

            capture.review_status = CalibrationReviewStatus::Applied;
            capture.reviewed_at = Some(Utc::now());
            capture.alignment.clone()
        };

        let mut areas = self.areas.lock().expect("seed area store lock");
        for overlay in areas
            .iter_mut()
            .flat_map(|area| area.walls.iter_mut())
            .flat_map(|wall| wall.routes.iter_mut())
            .flat_map(|route| route.ar_overlays.iter_mut())
        {
            if overlay.id == overlay_id {
                overlay.default_alignment = Some(alignment);
                overlay.confidence = OverlayConfidence::FieldTested;
                overlay.reviewed_at = Some(Utc::now());
                return Ok(Some(overlay.clone()));
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::SeedStore;
    use crate::{
        models::{
            ArAnchorStrategy, CalibrationReviewStatus, RouteArAlignment, RouteCalibrationCapture,
        },
        repository::GuideRepository,
    };
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn seed_data_has_area_wall_route_hierarchy() {
        let store = SeedStore::new();
        let areas = store.areas_seed();

        assert_eq!(areas.len(), 1);
        assert_eq!(areas[0].walls.len(), 1);
        assert_eq!(areas[0].walls[0].routes.len(), 1);
        assert_eq!(areas[0].walls[0].routes[0].ar_overlays.len(), 1);
    }

    #[test]
    fn offline_pack_contains_area_and_assets() {
        let store = SeedStore::new();
        let area = store.areas_seed().remove(0);
        let pack = store.offline_pack_seed(area.id).expect("area pack");

        assert_eq!(pack.area_id, area.id);
        assert_eq!(pack.version, 1);
        assert_eq!(pack.areas.len(), 1);
        assert_eq!(pack.assets.len(), 1);
    }

    #[test]
    fn seed_store_can_find_wall_route_and_search() {
        let store = SeedStore::new();

        assert!(store
            .wall_seed(uuid::Uuid::from_u128(
                0xbbbbbbbb_bbbb_bbbb_bbbb_bbbbbbbbbbbb
            ))
            .is_some());
        assert!(store
            .route_seed(uuid::Uuid::from_u128(
                0xcccccccc_cccc_cccc_cccc_cccccccccccc
            ))
            .is_some());
        assert_eq!(store.search_seed("5.8").len(), 1);
        assert_eq!(store.search_seed("arete").len(), 1);
    }

    #[tokio::test]
    async fn seed_store_can_capture_ar_calibration() {
        let store = SeedStore::new();
        let route_id = Uuid::from_u128(0xcccccccc_cccc_cccc_cccc_cccccccccccc);
        let overlay_id = Uuid::from_u128(0xdddddddd_dddd_dddd_dddd_dddddddddddd);
        let capture = RouteCalibrationCapture {
            id: Uuid::new_v4(),
            route_id,
            route_name: "Sample Arete".to_string(),
            overlay_id,
            overlay_version: 1,
            anchor_strategy: ArAnchorStrategy::ManualAlignment,
            alignment: RouteArAlignment {
                horizontal_offset_meters: 0.1,
                vertical_offset_meters: -0.2,
                depth_offset_meters: 0.3,
                scale: 1.1,
            },
            captured_at: Utc::now(),
            review_status: CalibrationReviewStatus::Pending,
            reviewer_notes: None,
            reviewed_at: None,
        };

        store
            .create_calibration_capture(capture.clone())
            .await
            .expect("capture calibration");

        let captures_by_route = store
            .calibration_captures(Some(route_id), None)
            .await
            .expect("captures by route");
        assert_eq!(captures_by_route.len(), 1);
        assert_eq!(captures_by_route[0].id, capture.id);

        let captures_by_overlay = store
            .calibration_captures(None, Some(overlay_id))
            .await
            .expect("captures by overlay");
        assert_eq!(captures_by_overlay.len(), 1);
        assert_eq!(captures_by_overlay[0].id, capture.id);
    }

    #[tokio::test]
    async fn seed_store_can_review_and_apply_ar_calibration() {
        let store = SeedStore::new();
        let route_id = Uuid::from_u128(0xcccccccc_cccc_cccc_cccc_cccccccccccc);
        let overlay_id = Uuid::from_u128(0xdddddddd_dddd_dddd_dddd_dddddddddddd);
        let capture = RouteCalibrationCapture {
            id: Uuid::new_v4(),
            route_id,
            route_name: "Sample Arete".to_string(),
            overlay_id,
            overlay_version: 1,
            anchor_strategy: ArAnchorStrategy::ManualAlignment,
            alignment: RouteArAlignment {
                horizontal_offset_meters: 0.4,
                vertical_offset_meters: -0.1,
                depth_offset_meters: 0.2,
                scale: 1.05,
            },
            captured_at: Utc::now(),
            review_status: CalibrationReviewStatus::Pending,
            reviewer_notes: None,
            reviewed_at: None,
        };

        store
            .create_calibration_capture(capture.clone())
            .await
            .expect("capture calibration");

        let reviewed = store
            .review_calibration_capture(
                capture.id,
                CalibrationReviewStatus::GoodCandidate,
                Some("Looks aligned from base stance.".to_string()),
            )
            .await
            .expect("review capture")
            .expect("reviewed capture");

        assert!(matches!(
            reviewed.review_status,
            CalibrationReviewStatus::GoodCandidate
        ));
        assert_eq!(
            reviewed.reviewer_notes.as_deref(),
            Some("Looks aligned from base stance.")
        );
        assert!(reviewed.reviewed_at.is_some());

        let overlay = store
            .apply_calibration_capture_to_overlay(overlay_id, capture.id)
            .await
            .expect("apply calibration")
            .expect("updated overlay");

        let alignment = overlay.default_alignment.expect("default alignment");
        assert_eq!(alignment.horizontal_offset_meters, 0.4);
        assert_eq!(alignment.vertical_offset_meters, -0.1);
        assert!(matches!(
            overlay.confidence,
            crate::models::OverlayConfidence::FieldTested
        ));

        let captures = store
            .calibration_captures(Some(route_id), Some(overlay_id))
            .await
            .expect("captures");
        assert!(matches!(
            captures[0].review_status,
            CalibrationReviewStatus::Applied
        ));
    }
}
