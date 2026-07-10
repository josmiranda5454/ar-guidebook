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

Run the API so a physical iPhone on the same network can reach it:

```sh
CLIMBAR_DATABASE_URL=postgres://climbar:climbar@127.0.0.1:5432/climbar CLIMBAR_HOST=0.0.0.0 cargo run
```

Use your Mac's LAN address in the iOS `CLIMBAR_API_BASE_URL` build setting,
for example `http://192.168.1.42:8080/api/v1`.

Run the API with in-memory seed data:

```sh
cargo run
```

## Endpoints

- `GET /health`
- `GET /api/v1/areas`
- `GET /api/v1/areas/{area_id}`
- `GET /api/v1/walls/{wall_id}`
- `GET /api/v1/routes/{route_id}`
- `GET /api/v1/search?q={query}`
- `GET /api/v1/offline-packs/areas/{area_id}`
- `PUT /api/v1/admin/routes/{route_id}`
- `PUT /api/v1/admin/ar-overlays/{overlay_id}`
- `GET /api/v1/admin/ar-calibration-captures?route_id={route_id}&overlay_id={overlay_id}`
- `POST /api/v1/admin/ar-calibration-captures`
- `POST /api/v1/admin/ar-calibration-captures/{capture_id}/review`
- `POST /api/v1/admin/ar-overlays/{overlay_id}/apply-calibration/{capture_id}`
