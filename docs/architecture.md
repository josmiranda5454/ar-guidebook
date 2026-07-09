# Architecture

## System Components

ClimbAR is split into three applications:

1. Native iOS app for browsing, offline use, location awareness, and AR overlays.
2. Backend API for route data, offline-pack generation, auth, and admin access.
3. Admin web app for entering guidebook data and calibrating AR route overlays.

## Technology Stack

### iOS

- Swift and SwiftUI.
- ARKit for camera tracking and world tracking.
- RealityKit for rendering route traces and labels.
- MapKit and Core Location for nearby route discovery.
- SwiftData or SQLite for offline packs.

### Backend

- Rust and Axum.
- PostgreSQL with PostGIS for production geospatial storage.
- Object storage for photos, topo images, and AR calibration captures.
- REST API for app and admin clients.

### Admin

- Next.js or SvelteKit.
- Route hierarchy editor.
- Topo and route-trace editor.
- Offline pack publishing workflow.

## Data Boundaries

Guidebook fields describe the route. AR overlay fields describe how to align and
render the route outside. Keeping them separate allows guidebook data to remain
stable while AR calibration improves over time.

## Offline Packs

Offline packs are versioned immutable snapshots. An area pack includes:

- Area metadata.
- Walls.
- Routes.
- Photos and topo asset references.
- AR overlay metadata.
- Manifest version and generated timestamp.

