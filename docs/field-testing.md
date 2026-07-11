# Field Testing Checklist

Before taking a phone outdoors:

1. Start Postgres and the API with `CLIMBAR_HOST=0.0.0.0`.
2. Set `CLIMBAR_API_BASE_URL` to the Mac's LAN address.
3. Set `CLIMBAR_ADMIN_TOKEN` in Xcode if calibration uploads are needed.
4. Confirm the iPhone and Mac are on the same network.
5. Download and refresh an area, then test route details in Airplane Mode.
6. Grant location and camera access and verify the nearby route state.
7. Open AR at the wall, check the confidence warning, and align the trace.
8. Save a calibration snapshot and verify the upload status.
9. Review the capture in the admin app and publish a new offline pack.
10. Refresh the area in iOS and confirm the published version changed.

AR guidance is visual assistance only. Climbers must independently assess rock
quality, protection, closures, weather, and access restrictions.
