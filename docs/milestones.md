# ClimbAR Milestones

## Milestone 0: Project Spine

Goal: Establish the data model, service boundaries, and source structure.

- Define area -> wall -> route hierarchy.
- Define Mountain Project-inspired route fields without copying proprietary data.
- Define AR overlay metadata separately from guidebook content.
- Define offline pack contracts.
- Scaffold backend API and iOS model layer.

Exit criteria:

- Backend exposes seed area/wall/route data.
- Backend exposes an offline pack response.
- iOS source has matching models and services.

## Milestone 1: Offline Guidebook MVP

Goal: Browse climbing data fully offline after downloading an area pack.

- Area list.
- Area detail with walls.
- Wall detail with routes.
- Route detail with grade, type, length, pitches, description, location notes,
  protection, safety notes, photos, and coordinates.
- Download and cache area packs.
- Versioned pack manifest.

Exit criteria:

- A user can download an area and view route details with no network.
- Pack versioning supports future delta sync.

## Milestone 2: Nearby Route Discovery

Goal: Show routes and walls near the climber.

- Location permission flow.
- Nearby area/wall/route search using coordinates.
- Route proximity states: out of range, nearby, at wall.
- Map view and wall route list.

Exit criteria:

- The app can identify nearby walls/routes from a downloaded pack.
- "Find it outside" only appears when route location requirements are met.

## Milestone 3: AR Route Overlay Prototype

Goal: Render a route line over a real wall using curated calibration data.

- ARKit world tracking.
- RealityKit line rendering for route traces.
- Manual alignment controls.
- Route overlay confidence states.
- Basic wall-plane and bearing hints.

Exit criteria:

- A curated route can display an AR trace on-device.
- The UI clearly communicates alignment quality and limitations.

## Milestone 4: Admin Data Capture

Goal: Provide a separate admin workflow for route data and AR calibration.

- Calibration capture upload API.
- Calibration capture review status.
- Apply reviewed calibration captures to AR overlay default alignment.
- Admin editor for existing route fields and AR overlay placement metadata.
- Text-based AR route trace editor for existing overlays.
- Create flows for draft areas, walls, routes, and first AR overlays.
- Admin authentication.
- CRUD for areas, walls, and routes.
- Photo and topo upload.
- Route trace editor.
- AR overlay review and version publishing.

Implemented in the current development milestone: bearer-token admin sessions,
protected admin writes/review operations, and durable versioned offline-pack
publishing backed by Postgres snapshots.

Exit criteria:

- Admins can publish a new offline pack version without app changes.

## Milestone 5: Production Readiness

Goal: Prepare for TestFlight and field testing.

- Sign in with Apple.
- Safety and access warnings.
- Moderation workflow for contributed data.
- Observability and crash reporting.
- CI and deployment automation.

Development CI and a container deployment baseline are now included. Sign in
with Apple, moderation, observability, and hosted media storage remain
follow-on integrations that require provider credentials and product policy.

Exit criteria:

- TestFlight build is ready for controlled outdoor testing.
