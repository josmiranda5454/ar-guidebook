# ClimbAR Backend

Rust/Axum API for guidebook data, AR overlays, and offline packs.

## Local Development

Start Postgres/PostGIS:

```sh
docker compose up -d postgres
```

Import seed data:

```sh
CLIMBAR_DATABASE_URL=postgres://climbar:climbar@127.0.0.1:5432/climbar cargo run -- import-seed
```

Run the API with Postgres:

```sh
CLIMBAR_DATABASE_URL=postgres://climbar:climbar@127.0.0.1:5432/climbar cargo run
```

Run the API with in-memory seed data:

```sh
cargo run
```

## Endpoints

- `GET /health`
- `GET /api/v1/areas`
- `GET /api/v1/areas/{area_id}`
- `GET /api/v1/offline-packs/areas/{area_id}`

