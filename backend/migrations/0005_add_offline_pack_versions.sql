CREATE TABLE offline_pack_versions (
    id uuid PRIMARY KEY,
    area_id uuid NOT NULL REFERENCES areas(id) ON DELETE CASCADE,
    version integer NOT NULL,
    generated_at timestamptz NOT NULL,
    payload jsonb NOT NULL,
    UNIQUE(area_id, version)
);

CREATE INDEX offline_pack_versions_area_version_idx
    ON offline_pack_versions(area_id, version DESC);
