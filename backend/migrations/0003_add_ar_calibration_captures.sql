CREATE TABLE route_calibration_captures (
    id uuid PRIMARY KEY,
    route_id uuid NOT NULL REFERENCES routes(id) ON DELETE CASCADE,
    route_name text NOT NULL,
    overlay_id uuid NOT NULL REFERENCES route_ar_overlays(id) ON DELETE CASCADE,
    overlay_version integer NOT NULL,
    anchor_strategy text NOT NULL,
    alignment jsonb NOT NULL,
    captured_at timestamptz NOT NULL
);

CREATE INDEX route_calibration_captures_route_id_idx ON route_calibration_captures(route_id);
CREATE INDEX route_calibration_captures_overlay_id_idx ON route_calibration_captures(overlay_id);
