export const REVIEW_STATUSES = [
  "pending",
  "good_candidate",
  "rejected",
  "applied",
];

let adminToken = globalThis.localStorage?.getItem("climbar-admin-token") ?? "";

export function setAdminToken(token) {
  adminToken = token ?? "";
  globalThis.localStorage?.setItem("climbar-admin-token", adminToken);
}

export function adminAuthHeaders() {
  return adminToken ? { Authorization: `Bearer ${adminToken}` } : {};
}

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

export function createAreaUrl(apiBaseUrl) {
  return `${normalizeApiBaseUrl(apiBaseUrl)}/admin/areas`;
}

export function updateAreaUrl(apiBaseUrl, areaId) {
  return `${normalizeApiBaseUrl(apiBaseUrl)}/admin/areas/${areaId}`;
}

export function createWallUrl(apiBaseUrl) {
  return `${normalizeApiBaseUrl(apiBaseUrl)}/admin/walls`;
}

export function updateWallUrl(apiBaseUrl, wallId) {
  return `${normalizeApiBaseUrl(apiBaseUrl)}/admin/walls/${wallId}`;
}

export function createRouteUrl(apiBaseUrl) {
  return `${normalizeApiBaseUrl(apiBaseUrl)}/admin/routes`;
}

export function createOverlayUrl(apiBaseUrl) {
  return `${normalizeApiBaseUrl(apiBaseUrl)}/admin/ar-overlays`;
}

export function updateRouteUrl(apiBaseUrl, routeId) {
  return `${normalizeApiBaseUrl(apiBaseUrl)}/admin/routes/${routeId}`;
}

export function updateOverlayUrl(apiBaseUrl, overlayId) {
  return `${normalizeApiBaseUrl(apiBaseUrl)}/admin/ar-overlays/${overlayId}`;
}

export function updateMediaUrl(apiBaseUrl, mediaId) {
  return `${normalizeApiBaseUrl(apiBaseUrl)}/admin/media/${mediaId}`;
}

export function loginUrl(apiBaseUrl) {
  return `${normalizeApiBaseUrl(apiBaseUrl)}/admin/auth/login`;
}

export function publishAreaPackUrl(apiBaseUrl, areaId) {
  return `${normalizeApiBaseUrl(apiBaseUrl)}/admin/offline-packs/areas/${areaId}/publish`;
}

export async function login(apiBaseUrl, email, password, fetchImpl = fetch) {
  const response = await fetchImpl(loginUrl(apiBaseUrl), {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ email, password }),
  });
  const result = await parseJsonResponse(response);
  setAdminToken(result.token);
  return result;
}

export async function publishAreaPack(apiBaseUrl, areaId, fetchImpl = fetch) {
  const response = await fetchImpl(publishAreaPackUrl(apiBaseUrl, areaId), {
    method: "POST",
    headers: adminAuthHeaders(),
  });
  return parseJsonResponse(response);
}

export async function listCalibrationCaptures(apiBaseUrl, filters = {}, fetchImpl = fetch) {
  const response = await fetchImpl(calibrationCaptureListUrl(apiBaseUrl, filters), { headers: adminAuthHeaders() });
  return parseJsonResponse(response);
}

export async function listAreas(apiBaseUrl, fetchImpl = fetch) {
  const response = await fetchImpl(areasUrl(apiBaseUrl));
  return parseJsonResponse(response);
}

export async function createArea(apiBaseUrl, area, fetchImpl = fetch) {
  return postJson(createAreaUrl(apiBaseUrl), area, fetchImpl);
}

export async function createWall(apiBaseUrl, wall, fetchImpl = fetch) {
  return postJson(createWallUrl(apiBaseUrl), wall, fetchImpl);
}

export async function updateArea(apiBaseUrl, area, fetchImpl = fetch) {
  return putJson(updateAreaUrl(apiBaseUrl, area.id), area, fetchImpl);
}

export async function updateWall(apiBaseUrl, wall, fetchImpl = fetch) {
  return putJson(updateWallUrl(apiBaseUrl, wall.id), wall, fetchImpl);
}

export async function createRoute(apiBaseUrl, route, fetchImpl = fetch) {
  return postJson(createRouteUrl(apiBaseUrl), route, fetchImpl);
}

export async function createOverlay(apiBaseUrl, overlay, fetchImpl = fetch) {
  return postJson(createOverlayUrl(apiBaseUrl), overlay, fetchImpl);
}

export async function updateRoute(apiBaseUrl, route, fetchImpl = fetch) {
  const response = await fetchImpl(updateRouteUrl(apiBaseUrl, route.id), {
    method: "PUT",
    headers: {
      "Content-Type": "application/json",
      ...adminAuthHeaders(),
    },
    body: JSON.stringify(route),
  });

  return parseJsonResponse(response);
}

async function postJson(url, body, fetchImpl) {
  const response = await fetchImpl(url, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      ...adminAuthHeaders(),
    },
    body: JSON.stringify(body),
  });

  return parseJsonResponse(response);
}

async function putJson(url, body, fetchImpl) {
  const response = await fetchImpl(url, {
    method: "PUT",
    headers: { "Content-Type": "application/json", ...adminAuthHeaders() },
    body: JSON.stringify(body),
  });
  return parseJsonResponse(response);
}

export async function updateOverlay(apiBaseUrl, overlay, fetchImpl = fetch) {
  const response = await fetchImpl(updateOverlayUrl(apiBaseUrl, overlay.id), {
    method: "PUT",
    headers: {
      "Content-Type": "application/json",
      ...adminAuthHeaders(),
    },
    body: JSON.stringify(overlay),
  });

  return parseJsonResponse(response);
}

export async function updateMedia(apiBaseUrl, media, fetchImpl = fetch) {
  const response = await fetchImpl(updateMediaUrl(apiBaseUrl, media.id), {
    method: "PUT",
    headers: { "Content-Type": "application/json", ...adminAuthHeaders() },
    body: JSON.stringify(media),
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
      ...adminAuthHeaders(),
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
    headers: adminAuthHeaders(),
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
