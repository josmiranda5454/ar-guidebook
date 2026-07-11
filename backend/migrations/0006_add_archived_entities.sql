CREATE TABLE archived_entities (
    entity_id uuid PRIMARY KEY,
    entity_type text NOT NULL,
    archived_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX archived_entities_type_idx ON archived_entities(entity_type);
