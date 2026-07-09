ALTER TABLE areas ADD COLUMN IF NOT EXISTS elevation_meters double precision;
ALTER TABLE walls ADD COLUMN IF NOT EXISTS elevation_meters double precision;
ALTER TABLE routes ADD COLUMN IF NOT EXISTS elevation_meters double precision;
ALTER TABLE route_ar_overlays ADD COLUMN IF NOT EXISTS gps_hint_elevation_meters double precision;
