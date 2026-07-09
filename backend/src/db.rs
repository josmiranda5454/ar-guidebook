use crate::{
    models::{
        ArAnchorStrategy, Area, GeoPoint, GradeSystem, MediaAsset, MediaKind, OfflinePack,
        OverlayConfidence, Route, RouteArOverlay, RouteTrace, RouteType, Wall, WallPlaneEstimate,
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
                id, parent_area_id, name, slug, description, access_notes, location
            )
            VALUES (
                $1, $2, $3, $4, $5, $6,
                ST_SetSRID(ST_MakePoint($7, $8), 4326)::geography
            )
            ON CONFLICT (id) DO UPDATE SET
                parent_area_id = EXCLUDED.parent_area_id,
                name = EXCLUDED.name,
                slug = EXCLUDED.slug,
                description = EXCLUDED.description,
                access_notes = EXCLUDED.access_notes,
                location = EXCLUDED.location
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
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn upsert_wall(&self, wall: &Wall) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO walls (
                id, area_id, name, slug, description, approach_notes, aspect, location
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7,
                ST_SetSRID(ST_MakePoint($8, $9), 4326)::geography
            )
            ON CONFLICT (id) DO UPDATE SET
                area_id = EXCLUDED.area_id,
                name = EXCLUDED.name,
                slug = EXCLUDED.slug,
                description = EXCLUDED.description,
                approach_notes = EXCLUDED.approach_notes,
                aspect = EXCLUDED.aspect,
                location = EXCLUDED.location
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
                location_notes, protection_notes, safety_notes, location
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8,
                $9, $10, $11, $12, $13, $14, $15, $16,
                ST_SetSRID(ST_MakePoint($17, $18), 4326)::geography
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
                location = EXCLUDED.location
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

        sqlx::query(
            r#"
            INSERT INTO route_ar_overlays (
                id, route_id, version, anchor_strategy, gps_hint, compass_bearing_degrees,
                wall_plane, route_trace, confidence, reviewed_at
            )
            VALUES (
                $1, $2, $3, $4,
                ST_SetSRID(ST_MakePoint($5, $6), 4326)::geography,
                $7, $8, $9, $10, $11
            )
            ON CONFLICT (id) DO UPDATE SET
                route_id = EXCLUDED.route_id,
                version = EXCLUDED.version,
                anchor_strategy = EXCLUDED.anchor_strategy,
                gps_hint = EXCLUDED.gps_hint,
                compass_bearing_degrees = EXCLUDED.compass_bearing_degrees,
                wall_plane = EXCLUDED.wall_plane,
                route_trace = EXCLUDED.route_trace,
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
        .bind(overlay.compass_bearing_degrees)
        .bind(wall_plane)
        .bind(route_trace)
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
                       ST_Z(location::geometry) AS elevation_meters
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
                       ST_Z(location::geometry) AS elevation_meters
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
                   ST_Z(location::geometry) AS elevation_meters
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

    async fn load_routes(&self, wall_id: Uuid) -> RepositoryResult<Vec<Route>> {
        let rows = sqlx::query(
            r#"
            SELECT id, wall_id, name, slug, grade, grade_system, route_types, length_feet,
                   pitches, stars_average, rating_votes, first_ascent, description,
                   location_notes, protection_notes, safety_notes,
                   ST_Y(location::geometry) AS latitude,
                   ST_X(location::geometry) AS longitude,
                   ST_Z(location::geometry) AS elevation_meters
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
                   ST_Z(gps_hint::geometry) AS elevation_meters,
                   compass_bearing_degrees, wall_plane, route_trace, confidence, reviewed_at
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
                    confidence: OverlayConfidence::from_str(&confidence)
                        .map_err(RepositoryError::Decode)?,
                    reviewed_at: row.try_get::<Option<DateTime<Utc>>, _>("reviewed_at")?,
                })
            })
            .collect()
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

    async fn offline_pack(&self, area_id: Uuid) -> RepositoryResult<Option<OfflinePack>> {
        let area = match self.area(area_id).await? {
            Some(area) => area,
            None => return Ok(None),
        };
        let assets = area
            .walls
            .iter()
            .flat_map(|wall| wall.routes.iter())
            .flat_map(|route| route.media.iter().cloned())
            .collect();

        Ok(Some(OfflinePack {
            id: Uuid::new_v4(),
            area_id,
            version: 1,
            generated_at: Utc::now(),
            areas: vec![area],
            assets,
        }))
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
