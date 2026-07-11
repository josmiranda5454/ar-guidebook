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

## Recommended Hosted Baseline

For the first hosted environment, use:

- **Render Web Service** for the Rust API from `backend/Dockerfile`.
- **Render Postgres** with PostGIS enabled for the guidebook database.
- **Render Static Site** for `admin/`, with `config.js` set to the public API URL.
- **GitHub Actions and GHCR** for tests, image publishing, provenance attestations,
  and the protected deployment trigger.

Render is a good first fit because it supports Docker services, managed Postgres,
static sites, TLS, and deploy hooks. The runtime can later move to Cloud Run or
AWS ECS without changing the application container or database contract.

## GitHub Actions Setup

The repository has two workflows:

- `ci.yml` runs Rust formatting/tests and admin tests on pushes and pull requests.
- `deploy.yml` builds `backend/Dockerfile`, publishes
  `ghcr.io/<owner>/<repository>/backend`, and creates a build provenance
  attestation on `main` pushes and version tags.

To enable the guarded Render deployment job, create a GitHub **production**
environment and add these settings:

1. Add the repository variable `RENDER_DEPLOY_ENABLED=true`.
2. Add the environment secret `RENDER_DEPLOY_HOOK_URL` using the API service's
   Render deploy hook. Keep the hook secret and require approval for production
   deployments if desired.
3. Configure Render's API service environment variables from the list above,
   using the internal Postgres connection string and a long random admin token.
4. Configure the admin static site to serve `admin/config.js` containing the
   public API URL, for example:

   ```js
   globalThis.CLIMBAR_API_BASE_URL = "https://api.example.com/api/v1";
   ```

The workflow remains inert until `RENDER_DEPLOY_ENABLED` is set. This makes pull
requests and initial repository setup safe while preserving a one-switch path to
continuous deployment.

## Data and Operations

Run migrations as part of API startup; `sqlx::migrate!` currently applies the
checked-in migrations when the Postgres repository connects. Before production,
take a database backup, configure TLS/custom domains, add error/latency
monitoring, and verify a restore procedure. Offline pack publishing remains an
explicit admin action after guidebook edits.

## Alternatives

- **Cloud Run + Cloud SQL/PostGIS + Cloud Storage**: stronger GCP integration and
  autoscaling, with more IAM and service configuration.
- **AWS ECS/Fargate + RDS PostgreSQL/PostGIS + S3/CloudFront**: strongest fit for
  a larger production platform, with the highest operational overhead.
- **Fly.io + managed external Postgres**: attractive for regional latency, but the
  database backup and failover story needs more ownership than the Render path.
