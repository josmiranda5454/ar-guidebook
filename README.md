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

The simulator uses `http://127.0.0.1:8080/api/v1` by default.

For a physical iPhone, the backend must listen on your Mac's network interface:

```sh
cd backend
CLIMBAR_DATABASE_URL=postgres://climbar:climbar@127.0.0.1:5432/climbar CLIMBAR_HOST=0.0.0.0 cargo run
```

Then set the Xcode build setting `CLIMBAR_API_BASE_URL` to your Mac's LAN API
URL, for example `http://192.168.1.42:8080/api/v1`. The iPhone and Mac must be
on the same network, and macOS may ask you to allow incoming connections for the
backend process.
