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
- `admin/` - Static admin UI for reviewing AR calibration captures.
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

The local admin API uses development credentials by default: `admin@example.com`
and `dev-password`. Set `CLIMBAR_ADMIN_EMAIL`, `CLIMBAR_ADMIN_PASSWORD`, and
`CLIMBAR_ADMIN_TOKEN` before starting the backend to change them. Guidebook
mutations and calibration review require the bearer token returned by the admin
login endpoint.

Open the iOS app in Xcode:

```sh
open ios/ClimbAR.xcodeproj
```

The simulator uses `http://127.0.0.1:8080/api/v1` by default.

Run the admin calibration review UI:

```sh
cd admin
npm run dev
```

Open `http://127.0.0.1:5173`.

For the end-to-end device workflow, see [docs/field-testing.md](docs/field-testing.md).

Sign in to the admin UI, edit guidebook data, then use **Publish Offline Pack**.
The iOS app downloads the latest published version when the user taps **Update
Offline Area** or pulls to refresh a downloaded area.

Nearby discovery is available from `GET /api/v1/nearby/routes` with latitude,
longitude, and an optional radius in meters. It returns routes sorted by the
stored guidebook order with their computed distance; the iOS API client exposes
the same contract for the location-driven UI milestone.

For a physical iPhone, the backend must listen on your Mac's network interface:

```sh
cd backend
CLIMBAR_DATABASE_URL=postgres://climbar:climbar@127.0.0.1:5432/climbar CLIMBAR_HOST=0.0.0.0 cargo run
```

Then set the Xcode build setting `CLIMBAR_API_BASE_URL` to your Mac's LAN API
URL, for example `http://192.168.1.42:8080/api/v1`. The iPhone and Mac must be
on the same network, and macOS may ask you to allow incoming connections for the
backend process.

To upload calibration snapshots from the iOS app during development, set the
Xcode build setting `CLIMBAR_ADMIN_TOKEN` to the backend's
`CLIMBAR_ADMIN_TOKEN` value. Browsing and AR rendering do not require this
token; only calibration uploads do.
