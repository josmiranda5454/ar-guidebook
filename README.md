# ClimbAR

ClimbAR is a native iOS climbing guide and AR route-finding app. It organizes
climbing information by area, wall, and route, supports offline guidebook packs,
and uses ARKit/RealityKit route overlays to help nearby climbers identify a
route outside.

## Milestones

See [docs/milestones.md](docs/milestones.md) for the implementation roadmap.

## Repository Layout

- `backend/` - Rust API service for route data, offline packs, and admin access.
- `ios/ClimbAR/` - Swift source skeleton for the native iOS app.
- `admin/` - Placeholder for the future route administration web app.
- `docs/` - Product, architecture, and implementation notes.

## Quick Start

Backend with in-memory seed data:

```sh
cd backend
cargo run
```

Backend with Postgres/PostGIS:

```sh
docker compose up -d postgres
cd backend
CLIMBAR_DATABASE_URL=postgres://climbar:climbar@127.0.0.1:5432/climbar cargo run -- import-seed
CLIMBAR_DATABASE_URL=postgres://climbar:climbar@127.0.0.1:5432/climbar cargo run
```

Open the iOS app in Xcode:

```sh
open ios/ClimbAR.xcodeproj
```
