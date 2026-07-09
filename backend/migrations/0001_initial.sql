-- Production schema sketch for PostgreSQL + PostGIS.
-- The executable MVP currently uses seed data while the API surface settles.

CREATE EXTENSION IF NOT EXISTS postgis;

CREATE TABLE areas (
    id uuid PRIMARY KEY,
    parent_area_id uuid REFERENCES areas(id),
    name text NOT NULL,
    slug text NOT NULL UNIQUE,
    description text NOT NULL,
    access_notes text,
    location geography(Point, 4326) NOT NULL
);

CREATE TABLE walls (
    id uuid PRIMARY KEY,
    area_id uuid NOT NULL REFERENCES areas(id),
    name text NOT NULL,
    slug text NOT NULL,
    description text NOT NULL,
    approach_notes text,
    aspect text,
    location geography(Point, 4326) NOT NULL,
    UNIQUE(area_id, slug)
);

CREATE TABLE routes (
    id uuid PRIMARY KEY,
    wall_id uuid NOT NULL REFERENCES walls(id),
    name text NOT NULL,
    slug text NOT NULL,
    grade text NOT NULL,
    grade_system text NOT NULL,
    route_types text[] NOT NULL,
    length_feet integer,
    pitches integer,
    stars_average real,
    rating_votes integer NOT NULL DEFAULT 0,
    first_ascent text,
    description text NOT NULL,
    location_notes text NOT NULL,
    protection_notes text,
    safety_notes text,
    location geography(Point, 4326) NOT NULL,
    UNIQUE(wall_id, slug)
);

CREATE TABLE media_assets (
    id uuid PRIMARY KEY,
    route_id uuid NOT NULL REFERENCES routes(id) ON DELETE CASCADE,
    kind text NOT NULL,
    title text NOT NULL,
    url text NOT NULL,
    offline_path text
);

CREATE TABLE route_ar_overlays (
    id uuid PRIMARY KEY,
    route_id uuid NOT NULL REFERENCES routes(id),
    version integer NOT NULL,
    anchor_strategy text NOT NULL,
    gps_hint geography(Point, 4326) NOT NULL,
    compass_bearing_degrees real,
    wall_plane jsonb,
    route_trace jsonb NOT NULL,
    confidence text NOT NULL,
    reviewed_at timestamptz,
    UNIQUE(route_id, version)
);

CREATE INDEX areas_location_idx ON areas USING gist (location);
CREATE INDEX walls_location_idx ON walls USING gist (location);
CREATE INDEX routes_location_idx ON routes USING gist (location);
CREATE INDEX walls_area_id_idx ON walls(area_id);
CREATE INDEX routes_wall_id_idx ON routes(wall_id);
CREATE INDEX media_assets_route_id_idx ON media_assets(route_id);
CREATE INDEX route_ar_overlays_route_id_idx ON route_ar_overlays(route_id);
