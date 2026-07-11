# Development Deployment

The local stack is intentionally small and reproducible:

- PostgreSQL/PostGIS runs from `compose.yaml`.
- The Rust API runs as a native process during development or from
  `backend/Dockerfile` for a container deployment.
- The admin UI is a static Node server and can be replaced by any static host.
- Offline packs are immutable JSON snapshots in Postgres. Publish from the
  admin UI after editing guidebook data.

Required API environment variables:

```sh
CLIMBAR_DATABASE_URL=postgres://climbar:climbar@db:5432/climbar
CLIMBAR_HOST=0.0.0.0
CLIMBAR_PORT=8080
CLIMBAR_ADMIN_EMAIL=admin@example.com
CLIMBAR_ADMIN_PASSWORD=change-me
CLIMBAR_ADMIN_TOKEN=replace-with-a-long-random-token
```

For an eventual hosted environment, put the API behind TLS and a reverse
proxy, store credentials in a secret manager, and move media URLs to object
storage plus a CDN. The current URL/offline-path media model keeps that change
localized to the media repository and admin upload flow.
