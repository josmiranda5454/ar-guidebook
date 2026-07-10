export const REVIEW_STATUSES = [
  "pending",
  "good_candidate",
  "rejected",
  "applied",
];

export function normalizeApiBaseUrl(value) {
  return value.trim().replace(/\/+$/, "");
}

export function calibrationCaptureListUrl(apiBaseUrl, filters = {}) {
  const url = new URL(`${normalizeApiBaseUrl(apiBaseUrl)}/admin/ar-calibration-captures`);

  if (filters.routeId?.trim()) {
    url.searchParams.set("route_id", filters.routeId.trim());
  }

  if (filters.overlayId?.trim()) {
    url.searchParams.set("overlay_id", filters.overlayId.trim());
  }

  return url.toString();
}

export function reviewCaptureUrl(apiBaseUrl, captureId) {
  return `${normalizeApiBaseUrl(apiBaseUrl)}/admin/ar-calibration-captures/${captureId}/review`;
}

export function applyCaptureUrl(apiBaseUrl, overlayId, captureId) {
  return `${normalizeApiBaseUrl(apiBaseUrl)}/admin/ar-overlays/${overlayId}/apply-calibration/${captureId}`;
}

export function areasUrl(apiBaseUrl) {
  return `${normalizeApiBaseUrl(apiBaseUrl)}/areas`;
}

export function updateRouteUrl(apiBaseUrl, routeId) {
  return `${normalizeApiBaseUrl(apiBaseUrl)}/admin/routes/${routeId}`;
}

export function updateOverlayUrl(apiBaseUrl, overlayId) {
  return `${normalizeApiBaseUrl(apiBaseUrl)}/admin/ar-overlays/${overlayId}`;
}

export async function listCalibrationCaptures(apiBaseUrl, filters = {}, fetchImpl = fetch) {
  const response = await fetchImpl(calibrationCaptureListUrl(apiBaseUrl, filters));
  return parseJsonResponse(response);
}

export async function listAreas(apiBaseUrl, fetchImpl = fetch) {
  const response = await fetchImpl(areasUrl(apiBaseUrl));
  return parseJsonResponse(response);
}

export async function updateRoute(apiBaseUrl, route, fetchImpl = fetch) {
  const response = await fetchImpl(updateRouteUrl(apiBaseUrl, route.id), {
    method: "PUT",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(route),
  });

  return parseJsonResponse(response);
}

export async function updateOverlay(apiBaseUrl, overlay, fetchImpl = fetch) {
  const response = await fetchImpl(updateOverlayUrl(apiBaseUrl, overlay.id), {
    method: "PUT",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(overlay),
  });

  return parseJsonResponse(response);
}

export async function reviewCalibrationCapture(
  apiBaseUrl,
  captureId,
  reviewStatus,
  reviewerNotes,
  fetchImpl = fetch,
) {
  const response = await fetchImpl(reviewCaptureUrl(apiBaseUrl, captureId), {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      review_status: reviewStatus,
      reviewer_notes: reviewerNotes?.trim() ? reviewerNotes.trim() : null,
    }),
  });

  return parseJsonResponse(response);
}

export async function applyCalibrationCapture(apiBaseUrl, overlayId, captureId, fetchImpl = fetch) {
  const response = await fetchImpl(applyCaptureUrl(apiBaseUrl, overlayId, captureId), {
    method: "POST",
  });

  return parseJsonResponse(response);
}

async function parseJsonResponse(response) {
  if (!response.ok) {
    const detail = await response.text().catch(() => "");
    throw new Error(detail || `Request failed with status ${response.status}`);
  }

  return response.json();
}
