# Production Readiness Boundary

The local development system is complete enough for controlled device and
field testing. Before production release, the deployment must add:

- Sign in with Apple and user-scoped sessions.
- Secret-manager-backed credentials and rotating tokens.
- TLS, reverse proxy, rate limiting, and request audit logs.
- Object storage/CDN for uploaded photos and topo images.
- Moderation and contributor attribution for guidebook changes.
- Crash reporting and backend metrics/traces.
- TestFlight signing, release automation, and a rollback procedure.

Those items depend on Apple Developer, cloud, and observability accounts that
are intentionally not embedded in this local repository.
