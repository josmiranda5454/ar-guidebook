# Field Recording Guide

Field recording produces calibration snapshots for admin review. It does not
publish an overlay directly and it must never use the admin token.

## Before Going Outside

1. Confirm the iOS build points to the HTTPS API URL.
2. Confirm the backend has a recorder email/password configured. The iOS app
   signs in when an upload is requested and stores only the short-lived session
   in Keychain. Never add recorder or admin credentials to Xcode build settings.
3. Confirm the route has matching route and overlay coordinates in the admin UI.
4. Confirm the overlay has the correct wall bearing, anchor strategy, and trace.
5. Download the area's offline pack before leaving reliable service.

## Capture Procedure

1. Stand where the route can be seen clearly and allow location permission.
2. Wait for GPS accuracy to settle and verify the route distance is plausible.
3. Face the wall; avoid moving during initial ARKit tracking.
4. Use the alignment controls to place the trace over the actual route line.
5. Save a snapshot only after checking the route start, top, and major direction
   changes against the rock.
6. Save two or three independent snapshots from slightly different positions.
7. Upload when online, or use **Share** to transfer the JSON to the admin team.

The app requires a recorder session before upload. Local snapshots remain
available for sharing, so field work does not depend on continuous connectivity.

## Admin Review

Reviewers should reject captures with stale coordinates, poor tracking, an
incorrect route start, an implausible scale, or a trace that only looks correct
from one camera position. Apply only a capture that remains aligned after moving
the camera and re-establishing tracking. Publish the area's offline pack after
the overlay is updated.

AR is visual guidance, not a safety system. Climbers must verify the route and
all protection, access, and safety information on the wall.
