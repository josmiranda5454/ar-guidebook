ALTER TABLE route_ar_overlays
    ADD COLUMN default_alignment jsonb;

ALTER TABLE route_calibration_captures
    ADD COLUMN review_status text NOT NULL DEFAULT 'pending',
    ADD COLUMN reviewer_notes text,
    ADD COLUMN reviewed_at timestamptz;

CREATE INDEX route_calibration_captures_review_status_idx
    ON route_calibration_captures(review_status);
