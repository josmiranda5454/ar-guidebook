use crate::models::{
    ArAnchorStrategy, Area, CalibrationReviewStatus, GeoPoint, GradeSystem, MediaAsset, MediaKind,
    NearbyRoute, OfflinePack, OverlayConfidence, Route, RouteArOverlay, RouteCalibrationCapture,
    RouteTrace, RouteType, TraceCoordinateSpace, TracePoint, Wall, WallPlaneEstimate,
};
use crate::repository::{GuideRepository, RepositoryResult};
use async_trait::async_trait;
use chrono::Utc;
use std::sync::Mutex;
use uuid::Uuid;

pub struct SeedStore {
    areas: Mutex<Vec<Area>>,
    calibration_captures: Mutex<Vec<RouteCalibrationCapture>>,
    published_packs: Mutex<Vec<OfflinePack>>,
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

        let initial_pack = OfflinePack {
            id: Uuid::new_v4(),
            area_id,
            version: 1,
            generated_at: Utc::now(),
            areas: vec![area.clone()],
            assets: area
                .walls
                .iter()
                .flat_map(|wall| wall.routes.iter())
                .flat_map(|route| route.media.iter().cloned())
                .collect(),
        };

        Self {
            areas: Mutex::new(vec![area]),
            calibration_captures: Mutex::new(Vec::new()),
            published_packs: Mutex::new(vec![initial_pack]),
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
        self.published_packs
            .lock()
            .expect("pack store lock")
            .iter()
            .filter(|pack| pack.area_id == area_id)
            .max_by_key(|pack| pack.version)
            .cloned()
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

    async fn nearby_routes(
        &self,
        latitude: f64,
        longitude: f64,
        radius_meters: f64,
    ) -> RepositoryResult<Vec<NearbyRoute>> {
        Ok(self
            .areas_seed()
            .into_iter()
            .flat_map(|area| area.walls)
            .flat_map(|wall| wall.routes)
            .filter_map(|route| {
                let distance = distance_meters(
                    latitude,
                    longitude,
                    route.location.latitude,
                    route.location.longitude,
                );
                (distance <= radius_meters).then_some(NearbyRoute {
                    route,
                    distance_meters: distance,
                })
            })
            .collect())
    }

    async fn offline_pack(&self, area_id: Uuid) -> RepositoryResult<Option<OfflinePack>> {
        Ok(self.offline_pack_seed(area_id))
    }

    async fn publish_offline_pack(&self, area_id: Uuid) -> RepositoryResult<Option<OfflinePack>> {
        let area = match self.area_seed(area_id) {
            Some(area) => area,
            None => return Ok(None),
        };
        let version = self
            .published_packs
            .lock()
            .expect("pack store lock")
            .iter()
            .filter(|pack| pack.area_id == area_id)
            .map(|pack| pack.version)
            .max()
            .unwrap_or(0)
            + 1;
        let pack = OfflinePack {
            id: Uuid::new_v4(),
            area_id,
            version,
            generated_at: Utc::now(),
            assets: area
                .walls
                .iter()
                .flat_map(|wall| wall.routes.iter())
                .flat_map(|route| route.media.iter().cloned())
                .collect(),
            areas: vec![area],
        };
        self.published_packs
            .lock()
            .expect("pack store lock")
            .push(pack.clone());
        Ok(Some(pack))
    }

    async fn create_area(&self, mut area: Area) -> RepositoryResult<Area> {
        area.walls.clear();
        self.areas
            .lock()
            .expect("seed area store lock")
            .push(area.clone());
        Ok(area)
    }

    async fn create_wall(&self, mut wall: Wall) -> RepositoryResult<Option<Wall>> {
        let mut areas = self.areas.lock().expect("seed area store lock");
        let Some(area) = areas.iter_mut().find(|area| area.id == wall.area_id) else {
            return Ok(None);
        };

        wall.routes.clear();
        area.walls.push(wall.clone());
        Ok(Some(wall))
    }

    async fn create_route(&self, mut route: Route) -> RepositoryResult<Option<Route>> {
        let mut areas = self.areas.lock().expect("seed area store lock");

        for wall in areas.iter_mut().flat_map(|area| area.walls.iter_mut()) {
            if wall.id == route.wall_id {
                route.media.clear();
                route.ar_overlays.clear();
                wall.routes.push(route.clone());
                return Ok(Some(route));
            }
        }

        Ok(None)
    }

    async fn create_ar_overlay(
        &self,
        overlay: RouteArOverlay,
    ) -> RepositoryResult<Option<RouteArOverlay>> {
        let mut areas = self.areas.lock().expect("seed area store lock");

        for route in areas
            .iter_mut()
            .flat_map(|area| area.walls.iter_mut())
            .flat_map(|wall| wall.routes.iter_mut())
        {
            if route.id == overlay.route_id {
                route.ar_overlays.push(overlay.clone());
                return Ok(Some(overlay));
            }
        }

        Ok(None)
    }

    async fn update_route(
        &self,
        route_id: Uuid,
        mut route: Route,
    ) -> RepositoryResult<Option<Route>> {
        let mut areas = self.areas.lock().expect("seed area store lock");

        for existing_route in areas
            .iter_mut()
            .flat_map(|area| area.walls.iter_mut())
            .flat_map(|wall| wall.routes.iter_mut())
        {
            if existing_route.id == route_id {
                route.id = route_id;
                route.wall_id = existing_route.wall_id;
                route.media = existing_route.media.clone();
                route.ar_overlays = existing_route.ar_overlays.clone();
                *existing_route = route.clone();
                return Ok(Some(route));
            }
        }

        Ok(None)
    }

    async fn update_ar_overlay(
        &self,
        overlay_id: Uuid,
        mut overlay: RouteArOverlay,
    ) -> RepositoryResult<Option<RouteArOverlay>> {
        let mut areas = self.areas.lock().expect("seed area store lock");

        for existing_overlay in areas
            .iter_mut()
            .flat_map(|area| area.walls.iter_mut())
            .flat_map(|wall| wall.routes.iter_mut())
            .flat_map(|route| route.ar_overlays.iter_mut())
        {
            if existing_overlay.id == overlay_id {
                overlay.id = overlay_id;
                overlay.route_id = existing_overlay.route_id;
                *existing_overlay = overlay.clone();
                return Ok(Some(overlay));
            }
        }

        Ok(None)
    }

    async fn update_media(
        &self,
        media_id: Uuid,
        mut media: MediaAsset,
    ) -> RepositoryResult<Option<MediaAsset>> {
        let mut areas = self.areas.lock().expect("seed area store lock");
        for existing in areas
            .iter_mut()
            .flat_map(|area| area.walls.iter_mut())
            .flat_map(|wall| wall.routes.iter_mut())
            .flat_map(|route| route.media.iter_mut())
        {
            if existing.id == media_id {
                media.id = media_id;
                *existing = media.clone();
                return Ok(Some(media));
            }
        }
        Ok(None)
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

fn distance_meters(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let lat1 = lat1.to_radians();
    let lat2 = lat2.to_radians();
    let dlat = (lat2 - lat1) / 2.0;
    let dlon = (lon2 - lon1).to_radians() / 2.0;
    let a = dlat.sin().powi(2) + lat1.cos() * lat2.cos() * dlon.sin().powi(2);
    6_371_000.0 * 2.0 * a.sqrt().asin()
}

#[cfg(test)]
mod tests {
    use super::SeedStore;
    use crate::{
        models::{
            ArAnchorStrategy, CalibrationReviewStatus, RouteArAlignment, RouteCalibrationCapture,
            RouteTrace, TraceCoordinateSpace, TracePoint,
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

    #[tokio::test]
    async fn nearby_routes_returns_routes_inside_radius() {
        let store = SeedStore::new();
        let results = store.nearby_routes(34.0103, -116.1669, 10.0).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].distance_meters < 1.0);
    }

    #[tokio::test]
    async fn publish_offline_pack_increments_version() {
        let store = SeedStore::new();
        let area_id = store.areas_seed()[0].id;
        let first = store.publish_offline_pack(area_id).await.unwrap().unwrap();
        let latest = store.offline_pack(area_id).await.unwrap().unwrap();
        assert_eq!(first.version, 2);
        assert_eq!(latest.version, 2);
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
    async fn seed_store_can_update_route_and_overlay() {
        let store = SeedStore::new();
        let route_id = Uuid::from_u128(0xcccccccc_cccc_cccc_cccc_cccccccccccc);
        let overlay_id = Uuid::from_u128(0xdddddddd_dddd_dddd_dddd_dddddddddddd);

        let mut route = store
            .route(route_id)
            .await
            .expect("load route")
            .expect("route");
        route.name = "Edited Arete".to_string();
        route.grade = "5.9".to_string();
        route.description = "Edited route description.".to_string();

        let updated_route = store
            .update_route(route_id, route)
            .await
            .expect("update route")
            .expect("updated route");

        assert_eq!(updated_route.name, "Edited Arete");
        assert_eq!(updated_route.grade, "5.9");
        assert_eq!(updated_route.ar_overlays.len(), 1);

        let mut overlay = updated_route.ar_overlays[0].clone();
        overlay.compass_bearing_degrees = Some(190.0);
        overlay.default_alignment = Some(RouteArAlignment {
            horizontal_offset_meters: 0.2,
            vertical_offset_meters: 0.1,
            depth_offset_meters: -0.4,
            scale: 1.2,
        });

        let updated_overlay = store
            .update_ar_overlay(overlay_id, overlay)
            .await
            .expect("update overlay")
            .expect("updated overlay");

        assert_eq!(updated_overlay.compass_bearing_degrees, Some(190.0));
        assert_eq!(
            updated_overlay
                .default_alignment
                .expect("default alignment")
                .scale,
            1.2
        );
    }

    #[tokio::test]
    async fn seed_store_can_create_guidebook_hierarchy() {
        let store = SeedStore::new();
        let area_id = Uuid::new_v4();
        let wall_id = Uuid::new_v4();
        let route_id = Uuid::new_v4();
        let overlay_id = Uuid::new_v4();
        let location = crate::models::GeoPoint {
            latitude: 34.0,
            longitude: -116.0,
            elevation_meters: Some(1200.0),
        };

        let area = store
            .create_area(crate::models::Area {
                id: area_id,
                parent_area_id: None,
                name: "New Area".to_string(),
                slug: "new-area".to_string(),
                description: "New area description.".to_string(),
                access_notes: None,
                location: location.clone(),
                walls: vec![],
            })
            .await
            .expect("create area");

        assert_eq!(area.id, area_id);
        assert!(area.walls.is_empty());

        let wall = store
            .create_wall(crate::models::Wall {
                id: wall_id,
                area_id,
                name: "New Wall".to_string(),
                slug: "new-wall".to_string(),
                description: "New wall description.".to_string(),
                approach_notes: None,
                aspect: Some("North".to_string()),
                location: location.clone(),
                routes: vec![],
            })
            .await
            .expect("create wall")
            .expect("wall");

        assert_eq!(wall.id, wall_id);
        assert!(wall.routes.is_empty());

        let route = store
            .create_route(crate::models::Route {
                id: route_id,
                wall_id,
                name: "New Route".to_string(),
                slug: "new-route".to_string(),
                grade: "5.7".to_string(),
                grade_system: crate::models::GradeSystem::YosemiteDecimal,
                route_types: vec![crate::models::RouteType::Sport],
                length_feet: Some(60),
                pitches: Some(1),
                stars_average: None,
                rating_votes: 0,
                first_ascent: None,
                description: "New route description.".to_string(),
                location_notes: "Starts near the tree.".to_string(),
                protection_notes: None,
                safety_notes: None,
                location: location.clone(),
                media: vec![],
                ar_overlays: vec![],
            })
            .await
            .expect("create route")
            .expect("route");

        assert_eq!(route.id, route_id);
        assert!(route.ar_overlays.is_empty());

        let overlay = store
            .create_ar_overlay(crate::models::RouteArOverlay {
                id: overlay_id,
                route_id,
                version: 1,
                anchor_strategy: ArAnchorStrategy::ManualAlignment,
                gps_hint: location,
                compass_bearing_degrees: None,
                wall_plane: None,
                route_trace: RouteTrace {
                    coordinate_space: TraceCoordinateSpace::NormalizedWallImage,
                    points: vec![
                        TracePoint {
                            x: 0.45,
                            y: 0.95,
                            z: None,
                        },
                        TracePoint {
                            x: 0.55,
                            y: 0.20,
                            z: None,
                        },
                    ],
                },
                default_alignment: Some(RouteArAlignment {
                    horizontal_offset_meters: 0.0,
                    vertical_offset_meters: 0.0,
                    depth_offset_meters: 0.0,
                    scale: 1.0,
                }),
                confidence: crate::models::OverlayConfidence::Draft,
                reviewed_at: None,
            })
            .await
            .expect("create overlay")
            .expect("overlay");

        assert_eq!(overlay.id, overlay_id);
        assert_eq!(overlay.route_trace.points.len(), 2);

        let created_area = store.area(area_id).await.expect("load area").expect("area");
        assert_eq!(created_area.walls.len(), 1);
        assert_eq!(created_area.walls[0].routes.len(), 1);
        assert_eq!(created_area.walls[0].routes[0].ar_overlays.len(), 1);
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
