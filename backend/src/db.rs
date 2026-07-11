use crate::{
    models::{
        ArAnchorStrategy, Area, CalibrationReviewStatus, GeoPoint, GradeSystem, MediaAsset,
        MediaKind, OfflinePack, OverlayConfidence, Route, RouteArAlignment, RouteArOverlay,
        RouteCalibrationCapture, RouteTrace, RouteType, Wall, WallPlaneEstimate,
    },
    repository::{GuideRepository, RepositoryError, RepositoryResult},
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use std::str::FromStr;
use uuid::Uuid;

pub struct PgGuideRepository {
    pool: PgPool,
}

impl PgGuideRepository {
    pub async fn connect(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }

    pub async fn import_seed(&self, areas: &[Area]) -> RepositoryResult<()> {
        for area in areas {
            self.upsert_area(area).await?;

            for wall in &area.walls {
                self.upsert_wall(wall).await?;

                for route in &wall.routes {
                    self.upsert_route(route).await?;

                    for media in &route.media {
                        self.upsert_media(route.id, media).await?;
                    }

                    for overlay in &route.ar_overlays {
                        self.upsert_overlay(overlay).await?;
                    }
                }
            }
        }

        Ok(())
    }

    async fn upsert_area(&self, area: &Area) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO areas (
                id, parent_area_id, name, slug, description, access_notes, location,
                elevation_meters
            )
            VALUES (
                $1, $2, $3, $4, $5, $6,
                ST_SetSRID(ST_MakePoint($7, $8), 4326)::geography,
                $9
            )
            ON CONFLICT (id) DO UPDATE SET
                parent_area_id = EXCLUDED.parent_area_id,
                name = EXCLUDED.name,
                slug = EXCLUDED.slug,
                description = EXCLUDED.description,
                access_notes = EXCLUDED.access_notes,
                location = EXCLUDED.location,
                elevation_meters = EXCLUDED.elevation_meters
            "#,
        )
        .bind(area.id)
        .bind(area.parent_area_id)
        .bind(&area.name)
        .bind(&area.slug)
        .bind(&area.description)
        .bind(&area.access_notes)
        .bind(area.location.longitude)
        .bind(area.location.latitude)
        .bind(area.location.elevation_meters)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn upsert_wall(&self, wall: &Wall) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO walls (
                id, area_id, name, slug, description, approach_notes, aspect, location,
                elevation_meters
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7,
                ST_SetSRID(ST_MakePoint($8, $9), 4326)::geography,
                $10
            )
            ON CONFLICT (id) DO UPDATE SET
                area_id = EXCLUDED.area_id,
                name = EXCLUDED.name,
                slug = EXCLUDED.slug,
                description = EXCLUDED.description,
                approach_notes = EXCLUDED.approach_notes,
                aspect = EXCLUDED.aspect,
                location = EXCLUDED.location,
                elevation_meters = EXCLUDED.elevation_meters
            "#,
        )
        .bind(wall.id)
        .bind(wall.area_id)
        .bind(&wall.name)
        .bind(&wall.slug)
        .bind(&wall.description)
        .bind(&wall.approach_notes)
        .bind(&wall.aspect)
        .bind(wall.location.longitude)
        .bind(wall.location.latitude)
        .bind(wall.location.elevation_meters)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn upsert_route(&self, route: &Route) -> RepositoryResult<()> {
        let route_types = route
            .route_types
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        sqlx::query(
            r#"
            INSERT INTO routes (
                id, wall_id, name, slug, grade, grade_system, route_types, length_feet,
                pitches, stars_average, rating_votes, first_ascent, description,
                location_notes, protection_notes, safety_notes, location, elevation_meters
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8,
                $9, $10, $11, $12, $13, $14, $15, $16,
                ST_SetSRID(ST_MakePoint($17, $18), 4326)::geography,
                $19
            )
            ON CONFLICT (id) DO UPDATE SET
                wall_id = EXCLUDED.wall_id,
                name = EXCLUDED.name,
                slug = EXCLUDED.slug,
                grade = EXCLUDED.grade,
                grade_system = EXCLUDED.grade_system,
                route_types = EXCLUDED.route_types,
                length_feet = EXCLUDED.length_feet,
                pitches = EXCLUDED.pitches,
                stars_average = EXCLUDED.stars_average,
                rating_votes = EXCLUDED.rating_votes,
                first_ascent = EXCLUDED.first_ascent,
                description = EXCLUDED.description,
                location_notes = EXCLUDED.location_notes,
                protection_notes = EXCLUDED.protection_notes,
                safety_notes = EXCLUDED.safety_notes,
                location = EXCLUDED.location,
                elevation_meters = EXCLUDED.elevation_meters
            "#,
        )
        .bind(route.id)
        .bind(route.wall_id)
        .bind(&route.name)
        .bind(&route.slug)
        .bind(&route.grade)
        .bind(route.grade_system.to_string())
        .bind(route_types)
        .bind(route.length_feet.map(i32::from))
        .bind(route.pitches.map(i32::from))
        .bind(route.stars_average)
        .bind(route.rating_votes as i32)
        .bind(&route.first_ascent)
        .bind(&route.description)
        .bind(&route.location_notes)
        .bind(&route.protection_notes)
        .bind(&route.safety_notes)
        .bind(route.location.longitude)
        .bind(route.location.latitude)
        .bind(route.location.elevation_meters)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn upsert_media(&self, route_id: Uuid, media: &MediaAsset) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO media_assets (id, route_id, kind, title, url, offline_path)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (id) DO UPDATE SET
                route_id = EXCLUDED.route_id,
                kind = EXCLUDED.kind,
                title = EXCLUDED.title,
                url = EXCLUDED.url,
                offline_path = EXCLUDED.offline_path
            "#,
        )
        .bind(media.id)
        .bind(route_id)
        .bind(media.kind.to_string())
        .bind(&media.title)
        .bind(&media.url)
        .bind(&media.offline_path)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn upsert_overlay(&self, overlay: &RouteArOverlay) -> RepositoryResult<()> {
        let wall_plane = serde_json::to_value(&overlay.wall_plane)
            .map_err(|error| RepositoryError::Decode(error.to_string()))?;
        let route_trace = serde_json::to_value(&overlay.route_trace)
            .map_err(|error| RepositoryError::Decode(error.to_string()))?;
        let default_alignment = serde_json::to_value(&overlay.default_alignment)
            .map_err(|error| RepositoryError::Decode(error.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO route_ar_overlays (
                id, route_id, version, anchor_strategy, gps_hint, gps_hint_elevation_meters,
                compass_bearing_degrees, wall_plane, route_trace, default_alignment, confidence,
                reviewed_at
            )
            VALUES (
                $1, $2, $3, $4,
                ST_SetSRID(ST_MakePoint($5, $6), 4326)::geography,
                $7, $8, $9, $10, $11, $12, $13
            )
            ON CONFLICT (id) DO UPDATE SET
                route_id = EXCLUDED.route_id,
                version = EXCLUDED.version,
                anchor_strategy = EXCLUDED.anchor_strategy,
                gps_hint = EXCLUDED.gps_hint,
                gps_hint_elevation_meters = EXCLUDED.gps_hint_elevation_meters,
                compass_bearing_degrees = EXCLUDED.compass_bearing_degrees,
                wall_plane = EXCLUDED.wall_plane,
                route_trace = EXCLUDED.route_trace,
                default_alignment = EXCLUDED.default_alignment,
                confidence = EXCLUDED.confidence,
                reviewed_at = EXCLUDED.reviewed_at
            "#,
        )
        .bind(overlay.id)
        .bind(overlay.route_id)
        .bind(overlay.version as i32)
        .bind(overlay.anchor_strategy.to_string())
        .bind(overlay.gps_hint.longitude)
        .bind(overlay.gps_hint.latitude)
        .bind(overlay.gps_hint.elevation_meters)
        .bind(overlay.compass_bearing_degrees)
        .bind(wall_plane)
        .bind(route_trace)
        .bind(default_alignment)
        .bind(overlay.confidence.to_string())
        .bind(overlay.reviewed_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn load_areas(&self, area_filter: Option<Uuid>) -> RepositoryResult<Vec<Area>> {
        let area_rows = match area_filter {
            Some(area_id) => {
                sqlx::query(
                    r#"
                SELECT id, parent_area_id, name, slug, description, access_notes,
                       ST_Y(location::geometry) AS latitude,
                       ST_X(location::geometry) AS longitude,
                       elevation_meters
                FROM areas
                WHERE id = $1
                ORDER BY name
                "#,
                )
                .bind(area_id)
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query(
                    r#"
                SELECT id, parent_area_id, name, slug, description, access_notes,
                       ST_Y(location::geometry) AS latitude,
                       ST_X(location::geometry) AS longitude,
                       elevation_meters
                FROM areas
                ORDER BY name
                "#,
                )
                .fetch_all(&self.pool)
                .await?
            }
        };

        let mut areas = Vec::with_capacity(area_rows.len());

        for row in area_rows {
            let area_id: Uuid = row.try_get("id")?;

            areas.push(Area {
                id: area_id,
                parent_area_id: row.try_get("parent_area_id")?,
                name: row.try_get("name")?,
                slug: row.try_get("slug")?,
                description: row.try_get("description")?,
                access_notes: row.try_get("access_notes")?,
                location: geo_point_from_row(&row)?,
                walls: self.load_walls(area_id).await?,
            });
        }

        Ok(areas)
    }

    async fn load_walls(&self, area_id: Uuid) -> RepositoryResult<Vec<Wall>> {
        let rows = sqlx::query(
            r#"
            SELECT id, area_id, name, slug, description, approach_notes, aspect,
                   ST_Y(location::geometry) AS latitude,
                   ST_X(location::geometry) AS longitude,
                   elevation_meters
            FROM walls
            WHERE area_id = $1
            ORDER BY name
            "#,
        )
        .bind(area_id)
        .fetch_all(&self.pool)
        .await?;

        let mut walls = Vec::with_capacity(rows.len());

        for row in rows {
            let wall_id: Uuid = row.try_get("id")?;

            walls.push(Wall {
                id: wall_id,
                area_id: row.try_get("area_id")?,
                name: row.try_get("name")?,
                slug: row.try_get("slug")?,
                description: row.try_get("description")?,
                approach_notes: row.try_get("approach_notes")?,
                aspect: row.try_get("aspect")?,
                location: geo_point_from_row(&row)?,
                routes: self.load_routes(wall_id).await?,
            });
        }

        Ok(walls)
    }

    async fn load_wall_by_id(&self, wall_id: Uuid) -> RepositoryResult<Option<Wall>> {
        let row = sqlx::query(
            r#"
            SELECT id, area_id, name, slug, description, approach_notes, aspect,
                   ST_Y(location::geometry) AS latitude,
                   ST_X(location::geometry) AS longitude,
                   elevation_meters
            FROM walls
            WHERE id = $1
            "#,
        )
        .bind(wall_id)
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        Ok(Some(Wall {
            id: row.try_get("id")?,
            area_id: row.try_get("area_id")?,
            name: row.try_get("name")?,
            slug: row.try_get("slug")?,
            description: row.try_get("description")?,
            approach_notes: row.try_get("approach_notes")?,
            aspect: row.try_get("aspect")?,
            location: geo_point_from_row(&row)?,
            routes: self.load_routes(wall_id).await?,
        }))
    }

    async fn load_routes(&self, wall_id: Uuid) -> RepositoryResult<Vec<Route>> {
        let rows = sqlx::query(
            r#"
            SELECT id, wall_id, name, slug, grade, grade_system, route_types, length_feet,
                   pitches, stars_average, rating_votes, first_ascent, description,
                   location_notes, protection_notes, safety_notes,
                   ST_Y(location::geometry) AS latitude,
                   ST_X(location::geometry) AS longitude,
                   elevation_meters
            FROM routes
            WHERE wall_id = $1
            ORDER BY name
            "#,
        )
        .bind(wall_id)
        .fetch_all(&self.pool)
        .await?;

        let mut routes = Vec::with_capacity(rows.len());

        for row in rows {
            let route_id: Uuid = row.try_get("id")?;
            let grade_system: String = row.try_get("grade_system")?;
            let route_type_values: Vec<String> = row.try_get("route_types")?;

            routes.push(Route {
                id: route_id,
                wall_id: row.try_get("wall_id")?,
                name: row.try_get("name")?,
                slug: row.try_get("slug")?,
                grade: row.try_get("grade")?,
                grade_system: GradeSystem::from_str(&grade_system)
                    .map_err(RepositoryError::Decode)?,
                route_types: route_type_values
                    .iter()
                    .map(|value| RouteType::from_str(value).map_err(RepositoryError::Decode))
                    .collect::<RepositoryResult<Vec<_>>>()?,
                length_feet: optional_u16(row.try_get("length_feet")?)?,
                pitches: optional_u8(row.try_get("pitches")?)?,
                stars_average: row.try_get("stars_average")?,
                rating_votes: required_u32(row.try_get("rating_votes")?)?,
                first_ascent: row.try_get("first_ascent")?,
                description: row.try_get("description")?,
                location_notes: row.try_get("location_notes")?,
                protection_notes: row.try_get("protection_notes")?,
                safety_notes: row.try_get("safety_notes")?,
                location: geo_point_from_row(&row)?,
                media: self.load_media(route_id).await?,
                ar_overlays: self.load_overlays(route_id).await?,
            });
        }

        Ok(routes)
    }

    async fn load_route_by_id(&self, route_id: Uuid) -> RepositoryResult<Option<Route>> {
        let row = sqlx::query(
            r#"
            SELECT id, wall_id, name, slug, grade, grade_system, route_types, length_feet,
                   pitches, stars_average, rating_votes, first_ascent, description,
                   location_notes, protection_notes, safety_notes,
                   ST_Y(location::geometry) AS latitude,
                   ST_X(location::geometry) AS longitude,
                   elevation_meters
            FROM routes
            WHERE id = $1
            "#,
        )
        .bind(route_id)
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let grade_system: String = row.try_get("grade_system")?;
        let route_type_values: Vec<String> = row.try_get("route_types")?;

        Ok(Some(Route {
            id: route_id,
            wall_id: row.try_get("wall_id")?,
            name: row.try_get("name")?,
            slug: row.try_get("slug")?,
            grade: row.try_get("grade")?,
            grade_system: GradeSystem::from_str(&grade_system).map_err(RepositoryError::Decode)?,
            route_types: route_type_values
                .iter()
                .map(|value| RouteType::from_str(value).map_err(RepositoryError::Decode))
                .collect::<RepositoryResult<Vec<_>>>()?,
            length_feet: optional_u16(row.try_get("length_feet")?)?,
            pitches: optional_u8(row.try_get("pitches")?)?,
            stars_average: row.try_get("stars_average")?,
            rating_votes: required_u32(row.try_get("rating_votes")?)?,
            first_ascent: row.try_get("first_ascent")?,
            description: row.try_get("description")?,
            location_notes: row.try_get("location_notes")?,
            protection_notes: row.try_get("protection_notes")?,
            safety_notes: row.try_get("safety_notes")?,
            location: geo_point_from_row(&row)?,
            media: self.load_media(route_id).await?,
            ar_overlays: self.load_overlays(route_id).await?,
        }))
    }

    async fn search_routes(&self, query: &str) -> RepositoryResult<Vec<Route>> {
        let normalized_query = query.trim();
        if normalized_query.is_empty() {
            return Ok(Vec::new());
        }

        let rows = sqlx::query(
            r#"
            SELECT id
            FROM routes
            WHERE name ILIKE $1
               OR grade ILIKE $1
               OR description ILIKE $1
               OR location_notes ILIKE $1
            ORDER BY name
            LIMIT 50
            "#,
        )
        .bind(format!("%{normalized_query}%"))
        .fetch_all(&self.pool)
        .await?;

        let mut routes = Vec::with_capacity(rows.len());
        for row in rows {
            if let Some(route) = self.load_route_by_id(row.try_get("id")?).await? {
                routes.push(route);
            }
        }

        Ok(routes)
    }

    async fn load_media(&self, route_id: Uuid) -> RepositoryResult<Vec<MediaAsset>> {
        let rows = sqlx::query(
            r#"
            SELECT id, kind, title, url, offline_path
            FROM media_assets
            WHERE route_id = $1
            ORDER BY title
            "#,
        )
        .bind(route_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                let kind: String = row.try_get("kind")?;
                Ok(MediaAsset {
                    id: row.try_get("id")?,
                    kind: MediaKind::from_str(&kind).map_err(RepositoryError::Decode)?,
                    title: row.try_get("title")?,
                    url: row.try_get("url")?,
                    offline_path: row.try_get("offline_path")?,
                })
            })
            .collect()
    }

    async fn load_overlays(&self, route_id: Uuid) -> RepositoryResult<Vec<RouteArOverlay>> {
        let rows = sqlx::query(
            r#"
            SELECT id, route_id, version, anchor_strategy,
                   ST_Y(gps_hint::geometry) AS latitude,
                   ST_X(gps_hint::geometry) AS longitude,
                   gps_hint_elevation_meters AS elevation_meters,
                   compass_bearing_degrees, wall_plane, route_trace, default_alignment,
                   confidence, reviewed_at
            FROM route_ar_overlays
            WHERE route_id = $1
            ORDER BY version DESC
            "#,
        )
        .bind(route_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                let anchor_strategy: String = row.try_get("anchor_strategy")?;
                let confidence: String = row.try_get("confidence")?;
                let wall_plane_json: Option<serde_json::Value> = row.try_get("wall_plane")?;
                let route_trace_json: serde_json::Value = row.try_get("route_trace")?;
                let default_alignment_json: Option<serde_json::Value> =
                    row.try_get("default_alignment")?;

                Ok(RouteArOverlay {
                    id: row.try_get("id")?,
                    route_id: row.try_get("route_id")?,
                    version: required_u32(row.try_get("version")?)?,
                    anchor_strategy: ArAnchorStrategy::from_str(&anchor_strategy)
                        .map_err(RepositoryError::Decode)?,
                    gps_hint: geo_point_from_row(&row)?,
                    compass_bearing_degrees: row.try_get("compass_bearing_degrees")?,
                    wall_plane: match wall_plane_json {
                        Some(value) => {
                            serde_json::from_value::<Option<WallPlaneEstimate>>(value)
                                .map_err(|error| RepositoryError::Decode(error.to_string()))?
                        }
                        None => None,
                    },
                    route_trace: serde_json::from_value::<RouteTrace>(route_trace_json)
                        .map_err(|error| RepositoryError::Decode(error.to_string()))?,
                    default_alignment: match default_alignment_json {
                        Some(value) => serde_json::from_value::<Option<RouteArAlignment>>(value)
                            .map_err(|error| RepositoryError::Decode(error.to_string()))?,
                        None => None,
                    },
                    confidence: OverlayConfidence::from_str(&confidence)
                        .map_err(RepositoryError::Decode)?,
                    reviewed_at: row.try_get::<Option<DateTime<Utc>>, _>("reviewed_at")?,
                })
            })
            .collect()
    }

    async fn insert_calibration_capture(
        &self,
        capture: &RouteCalibrationCapture,
    ) -> RepositoryResult<()> {
        let alignment = serde_json::to_value(&capture.alignment)
            .map_err(|error| RepositoryError::Decode(error.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO route_calibration_captures (
                id, route_id, route_name, overlay_id, overlay_version, anchor_strategy,
                alignment, captured_at, review_status, reviewer_notes, reviewed_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (id) DO UPDATE SET
                route_id = EXCLUDED.route_id,
                route_name = EXCLUDED.route_name,
                overlay_id = EXCLUDED.overlay_id,
                overlay_version = EXCLUDED.overlay_version,
                anchor_strategy = EXCLUDED.anchor_strategy,
                alignment = EXCLUDED.alignment,
                captured_at = EXCLUDED.captured_at,
                review_status = EXCLUDED.review_status,
                reviewer_notes = EXCLUDED.reviewer_notes,
                reviewed_at = EXCLUDED.reviewed_at
            "#,
        )
        .bind(capture.id)
        .bind(capture.route_id)
        .bind(&capture.route_name)
        .bind(capture.overlay_id)
        .bind(capture.overlay_version as i32)
        .bind(capture.anchor_strategy.to_string())
        .bind(alignment)
        .bind(capture.captured_at)
        .bind(capture.review_status.to_string())
        .bind(&capture.reviewer_notes)
        .bind(capture.reviewed_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn load_calibration_captures(
        &self,
        route_id: Option<Uuid>,
        overlay_id: Option<Uuid>,
    ) -> RepositoryResult<Vec<RouteCalibrationCapture>> {
        let rows = sqlx::query(
            r#"
            SELECT id, route_id, route_name, overlay_id, overlay_version, anchor_strategy,
                   alignment, captured_at, review_status, reviewer_notes, reviewed_at
            FROM route_calibration_captures
            WHERE ($1::uuid IS NULL OR route_id = $1)
              AND ($2::uuid IS NULL OR overlay_id = $2)
            ORDER BY captured_at DESC
            "#,
        )
        .bind(route_id)
        .bind(overlay_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                let anchor_strategy: String = row.try_get("anchor_strategy")?;
                let alignment_json: serde_json::Value = row.try_get("alignment")?;
                let review_status: String = row.try_get("review_status")?;

                Ok(RouteCalibrationCapture {
                    id: row.try_get("id")?,
                    route_id: row.try_get("route_id")?,
                    route_name: row.try_get("route_name")?,
                    overlay_id: row.try_get("overlay_id")?,
                    overlay_version: required_u32(row.try_get("overlay_version")?)?,
                    anchor_strategy: ArAnchorStrategy::from_str(&anchor_strategy)
                        .map_err(RepositoryError::Decode)?,
                    alignment: serde_json::from_value::<RouteArAlignment>(alignment_json)
                        .map_err(|error| RepositoryError::Decode(error.to_string()))?,
                    captured_at: row.try_get("captured_at")?,
                    review_status: CalibrationReviewStatus::from_str(&review_status)
                        .map_err(RepositoryError::Decode)?,
                    reviewer_notes: row.try_get("reviewer_notes")?,
                    reviewed_at: row.try_get("reviewed_at")?,
                })
            })
            .collect()
    }

    async fn update_calibration_capture_review(
        &self,
        capture_id: Uuid,
        review_status: CalibrationReviewStatus,
        reviewer_notes: Option<String>,
    ) -> RepositoryResult<Option<RouteCalibrationCapture>> {
        let reviewed_at = Utc::now();
        let row = sqlx::query(
            r#"
            UPDATE route_calibration_captures
            SET review_status = $2,
                reviewer_notes = $3,
                reviewed_at = $4
            WHERE id = $1
            RETURNING route_id, overlay_id
            "#,
        )
        .bind(capture_id)
        .bind(review_status.to_string())
        .bind(reviewer_notes)
        .bind(reviewed_at)
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let route_id = row.try_get("route_id")?;
        let overlay_id = row.try_get("overlay_id")?;

        Ok(self
            .load_calibration_captures(Some(route_id), Some(overlay_id))
            .await?
            .into_iter()
            .find(|capture| capture.id == capture_id))
    }

    async fn apply_calibration_capture(
        &self,
        overlay_id: Uuid,
        capture_id: Uuid,
    ) -> RepositoryResult<Option<RouteArOverlay>> {
        let mut transaction = self.pool.begin().await?;

        let capture_row = sqlx::query(
            r#"
            SELECT route_id, alignment
            FROM route_calibration_captures
            WHERE id = $1 AND overlay_id = $2
            "#,
        )
        .bind(capture_id)
        .bind(overlay_id)
        .fetch_optional(&mut *transaction)
        .await?;

        let Some(capture_row) = capture_row else {
            transaction.commit().await?;
            return Ok(None);
        };

        let route_id = capture_row.try_get("route_id")?;
        let alignment: serde_json::Value = capture_row.try_get("alignment")?;
        let reviewed_at = Utc::now();

        let updated_overlay = sqlx::query(
            r#"
            UPDATE route_ar_overlays
            SET default_alignment = $2,
                confidence = 'field_tested',
                reviewed_at = $3
            WHERE id = $1
            RETURNING id
            "#,
        )
        .bind(overlay_id)
        .bind(alignment)
        .bind(reviewed_at)
        .fetch_optional(&mut *transaction)
        .await?;

        if updated_overlay.is_none() {
            transaction.commit().await?;
            return Ok(None);
        }

        sqlx::query(
            r#"
            UPDATE route_calibration_captures
            SET review_status = 'applied',
                reviewed_at = $2
            WHERE id = $1
            "#,
        )
        .bind(capture_id)
        .bind(reviewed_at)
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;

        Ok(self
            .load_overlays(route_id)
            .await?
            .into_iter()
            .find(|overlay| overlay.id == overlay_id))
    }
}

#[async_trait]
impl GuideRepository for PgGuideRepository {
    async fn areas(&self) -> RepositoryResult<Vec<Area>> {
        self.load_areas(None).await
    }

    async fn area(&self, area_id: Uuid) -> RepositoryResult<Option<Area>> {
        Ok(self.load_areas(Some(area_id)).await?.into_iter().next())
    }

    async fn wall(&self, wall_id: Uuid) -> RepositoryResult<Option<Wall>> {
        self.load_wall_by_id(wall_id).await
    }

    async fn route(&self, route_id: Uuid) -> RepositoryResult<Option<Route>> {
        self.load_route_by_id(route_id).await
    }

    async fn search(&self, query: &str) -> RepositoryResult<Vec<Route>> {
        self.search_routes(query).await
    }

    async fn offline_pack(&self, area_id: Uuid) -> RepositoryResult<Option<OfflinePack>> {
        let row = sqlx::query("SELECT payload FROM offline_pack_versions WHERE area_id = $1 ORDER BY version DESC LIMIT 1")
            .bind(area_id).fetch_optional(&self.pool).await?;
        row.map(|row| {
            let payload: serde_json::Value = row.try_get("payload")?;
            serde_json::from_value(payload)
                .map_err(|error| RepositoryError::Decode(error.to_string()))
        })
        .transpose()
    }

    async fn publish_offline_pack(&self, area_id: Uuid) -> RepositoryResult<Option<OfflinePack>> {
        let area = match self.area(area_id).await? {
            Some(area) => area,
            None => return Ok(None),
        };
        let version: i32 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(version), 0) + 1 FROM offline_pack_versions WHERE area_id = $1",
        )
        .bind(area_id)
        .fetch_one(&self.pool)
        .await?;
        let pack = OfflinePack {
            id: Uuid::new_v4(),
            area_id,
            version: version as u32,
            generated_at: Utc::now(),
            assets: area
                .walls
                .iter()
                .flat_map(|wall| wall.routes.iter())
                .flat_map(|route| route.media.iter().cloned())
                .collect(),
            areas: vec![area],
        };
        let payload = serde_json::to_value(&pack)
            .map_err(|error| RepositoryError::Decode(error.to_string()))?;
        sqlx::query("INSERT INTO offline_pack_versions (id, area_id, version, generated_at, payload) VALUES ($1, $2, $3, $4, $5)")
            .bind(pack.id).bind(area_id).bind(version).bind(pack.generated_at).bind(payload).execute(&self.pool).await?;
        Ok(Some(pack))
    }

    async fn create_area(&self, mut area: Area) -> RepositoryResult<Area> {
        area.walls.clear();
        self.upsert_area(&area).await?;

        Ok(self
            .area(area.id)
            .await?
            .expect("created area should be readable"))
    }

    async fn create_wall(&self, mut wall: Wall) -> RepositoryResult<Option<Wall>> {
        if self.area(wall.area_id).await?.is_none() {
            return Ok(None);
        }

        wall.routes.clear();
        self.upsert_wall(&wall).await?;
        self.wall(wall.id).await
    }

    async fn create_route(&self, mut route: Route) -> RepositoryResult<Option<Route>> {
        if self.wall(route.wall_id).await?.is_none() {
            return Ok(None);
        }

        route.media.clear();
        route.ar_overlays.clear();
        self.upsert_route(&route).await?;
        self.route(route.id).await
    }

    async fn create_ar_overlay(
        &self,
        overlay: RouteArOverlay,
    ) -> RepositoryResult<Option<RouteArOverlay>> {
        if self.route(overlay.route_id).await?.is_none() {
            return Ok(None);
        }

        self.upsert_overlay(&overlay).await?;

        Ok(self
            .load_overlays(overlay.route_id)
            .await?
            .into_iter()
            .find(|existing_overlay| existing_overlay.id == overlay.id))
    }

    async fn update_route(
        &self,
        route_id: Uuid,
        mut route: Route,
    ) -> RepositoryResult<Option<Route>> {
        if self.load_route_by_id(route_id).await?.is_none() {
            return Ok(None);
        }

        route.id = route_id;
        self.upsert_route(&route).await?;
        self.load_route_by_id(route_id).await
    }

    async fn update_ar_overlay(
        &self,
        overlay_id: Uuid,
        mut overlay: RouteArOverlay,
    ) -> RepositoryResult<Option<RouteArOverlay>> {
        let route_id = overlay.route_id;
        let overlay_exists = self
            .load_overlays(route_id)
            .await?
            .into_iter()
            .any(|existing_overlay| existing_overlay.id == overlay_id);

        if !overlay_exists {
            return Ok(None);
        }

        overlay.id = overlay_id;
        self.upsert_overlay(&overlay).await?;

        Ok(self
            .load_overlays(route_id)
            .await?
            .into_iter()
            .find(|overlay| overlay.id == overlay_id))
    }

    async fn create_calibration_capture(
        &self,
        capture: RouteCalibrationCapture,
    ) -> RepositoryResult<RouteCalibrationCapture> {
        self.insert_calibration_capture(&capture).await?;
        Ok(capture)
    }

    async fn calibration_captures(
        &self,
        route_id: Option<Uuid>,
        overlay_id: Option<Uuid>,
    ) -> RepositoryResult<Vec<RouteCalibrationCapture>> {
        self.load_calibration_captures(route_id, overlay_id).await
    }

    async fn review_calibration_capture(
        &self,
        capture_id: Uuid,
        review_status: CalibrationReviewStatus,
        reviewer_notes: Option<String>,
    ) -> RepositoryResult<Option<RouteCalibrationCapture>> {
        self.update_calibration_capture_review(capture_id, review_status, reviewer_notes)
            .await
    }

    async fn apply_calibration_capture_to_overlay(
        &self,
        overlay_id: Uuid,
        capture_id: Uuid,
    ) -> RepositoryResult<Option<RouteArOverlay>> {
        self.apply_calibration_capture(overlay_id, capture_id).await
    }
}

fn geo_point_from_row(row: &sqlx::postgres::PgRow) -> RepositoryResult<GeoPoint> {
    Ok(GeoPoint {
        latitude: row.try_get("latitude")?,
        longitude: row.try_get("longitude")?,
        elevation_meters: row.try_get("elevation_meters").ok(),
    })
}

fn optional_u16(value: Option<i32>) -> RepositoryResult<Option<u16>> {
    value
        .map(|value| {
            u16::try_from(value).map_err(|error| RepositoryError::Decode(error.to_string()))
        })
        .transpose()
}

fn optional_u8(value: Option<i32>) -> RepositoryResult<Option<u8>> {
    value
        .map(|value| {
            u8::try_from(value).map_err(|error| RepositoryError::Decode(error.to_string()))
        })
        .transpose()
}

fn required_u32(value: i32) -> RepositoryResult<u32> {
    u32::try_from(value).map_err(|error| RepositoryError::Decode(error.to_string()))
}
