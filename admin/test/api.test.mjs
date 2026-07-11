import assert from "node:assert/strict";
import { test } from "node:test";
import {
  applyCalibrationCapture,
  applyCaptureUrl,
  archiveEntity,
  archiveEntityUrl,
  archivedUrl,
  areasUrl,
  calibrationCaptureListUrl,
  createArea,
  createMedia,
  createMediaUrl,
  createAreaUrl,
  createOverlay,
  createOverlayUrl,
  createRoute,
  createRouteUrl,
  createWall,
  createWallUrl,
  loginUrl,
  login,
  publishAreaPack,
  publishAreaPackUrl,
  reviewCalibrationCapture,
  reviewCaptureUrl,
  updateOverlay,
  updateOverlayUrl,
  updateMedia,
  updateMediaUrl,
  updateArea,
  updateAreaUrl,
  updateWall,
  updateWallUrl,
  restoreArchived,
  restoreArchivedUrl,
  updateRoute,
  updateRouteUrl,
} from "../src/api.js";
import { draftArea, draftOverlay, draftRoute, draftWall, slugify } from "../src/drafts.js";
import { formatAlignment, formatReviewStatus } from "../src/format.js";
import { parseTracePoints, tracePointsToText, validateNormalizedTrace } from "../src/trace.js";

test("builds calibration capture list URL with optional filters", () => {
  const url = calibrationCaptureListUrl("http://127.0.0.1:8080/api/v1/", {
    routeId: " route-id ",
    overlayId: "overlay-id",
  });

  assert.equal(
    url,
    "http://127.0.0.1:8080/api/v1/admin/ar-calibration-captures?route_id=route-id&overlay_id=overlay-id",
  );
});

test("builds review and apply endpoint URLs", () => {
  assert.equal(
    reviewCaptureUrl("http://localhost:8080/api/v1", "capture-1"),
    "http://localhost:8080/api/v1/admin/ar-calibration-captures/capture-1/review",
  );
  assert.equal(
    applyCaptureUrl("http://localhost:8080/api/v1", "overlay-1", "capture-1"),
    "http://localhost:8080/api/v1/admin/ar-overlays/overlay-1/apply-calibration/capture-1",
  );
});

test("builds guidebook editor endpoint URLs", () => {
  assert.equal(
    areasUrl("http://localhost:8080/api/v1/"),
    "http://localhost:8080/api/v1/areas",
  );
  assert.equal(
    updateRouteUrl("http://localhost:8080/api/v1", "route-1"),
    "http://localhost:8080/api/v1/admin/routes/route-1",
  );
  assert.equal(
    updateOverlayUrl("http://localhost:8080/api/v1", "overlay-1"),
    "http://localhost:8080/api/v1/admin/ar-overlays/overlay-1",
  );
  assert.equal(
    createAreaUrl("http://localhost:8080/api/v1"),
    "http://localhost:8080/api/v1/admin/areas",
  );
  assert.equal(
    createWallUrl("http://localhost:8080/api/v1"),
    "http://localhost:8080/api/v1/admin/walls",
  );
  assert.equal(
    createRouteUrl("http://localhost:8080/api/v1"),
    "http://localhost:8080/api/v1/admin/routes",
  );
  assert.equal(
    createOverlayUrl("http://localhost:8080/api/v1"),
    "http://localhost:8080/api/v1/admin/ar-overlays",
  );
});

test("builds authentication and publish endpoint URLs", () => {
  assert.equal(loginUrl("http://localhost:8080/api/v1/"), "http://localhost:8080/api/v1/admin/auth/login");
  assert.equal(publishAreaPackUrl("http://localhost:8080/api/v1", "area-1"), "http://localhost:8080/api/v1/admin/offline-packs/areas/area-1/publish");
  assert.equal(updateMediaUrl("http://localhost:8080/api/v1", "media-1"), "http://localhost:8080/api/v1/admin/media/media-1");
  assert.equal(updateAreaUrl("http://localhost:8080/api/v1", "area-1"), "http://localhost:8080/api/v1/admin/areas/area-1");
  assert.equal(updateWallUrl("http://localhost:8080/api/v1", "wall-1"), "http://localhost:8080/api/v1/admin/walls/wall-1");
  assert.equal(archiveEntityUrl("http://localhost:8080/api/v1", "area", "area-1"), "http://localhost:8080/api/v1/admin/areas/area-1/archive");
  assert.equal(createMediaUrl("http://localhost:8080/api/v1", "route-1"), "http://localhost:8080/api/v1/admin/routes/route-1/media");
  assert.equal(archivedUrl("http://localhost:8080/api/v1"), "http://localhost:8080/api/v1/admin/archived");
  assert.equal(restoreArchivedUrl("http://localhost:8080/api/v1", "route-1"), "http://localhost:8080/api/v1/admin/archived/route-1/restore");
});

test("login reports invalid credentials instead of an expired session", async () => {
  const fetchImpl = async () => new Response(null, { status: 401 });

  await assert.rejects(
    login("http://localhost:8080/api/v1", "wrong@example.com", "wrong", fetchImpl),
    { message: "Invalid admin email or password." },
  );
});

test("restores an archived entity with an authenticated POST", async () => {
  const calls = [];
  const fetchImpl = async (url, options) => { calls.push({ url, options }); return new Response(null, { status: 204 }); };
  await restoreArchived("http://localhost:8080/api/v1", "route-1", fetchImpl);
  assert.equal(calls[0].options.method, "POST");
});

test("media creation posts an asset to its route", async () => {
  const calls = [];
  const fetchImpl = async (url, options) => { calls.push({ url, options }); return Response.json({ id: "media-1" }); };
  await createMedia("http://localhost:8080/api/v1", "route-1", { id: "media-1", kind: "photo" }, fetchImpl);
  assert.equal(calls[0].options.method, "POST");
  assert.deepEqual(JSON.parse(calls[0].options.body), { id: "media-1", kind: "photo" });
});

test("archive requests post to the matching guidebook entity endpoint", async () => {
  const calls = [];
  const fetchImpl = async (url, options) => { calls.push({ url, options }); return new Response(null, { status: 204 }); };
  await archiveEntity("http://localhost:8080/api/v1", "wall", "wall-1", fetchImpl);
  assert.equal(calls[0].options.method, "POST");
  assert.equal(calls[0].url, "http://localhost:8080/api/v1/admin/walls/wall-1/archive");
});

test("area and wall updates use PUT with JSON", async () => {
  const calls = [];
  const fetchImpl = async (url, options) => { calls.push({ url, options }); return Response.json(JSON.parse(options.body)); };
  await updateArea("http://localhost:8080/api/v1", { id: "area-1", name: "Edited Area" }, fetchImpl);
  await updateWall("http://localhost:8080/api/v1", { id: "wall-1", name: "Edited Wall" }, fetchImpl);
  assert.deepEqual(calls.map(({ options }) => options.method), ["PUT", "PUT"]);
  assert.deepEqual(JSON.parse(calls[0].options.body), { id: "area-1", name: "Edited Area" });
  assert.deepEqual(JSON.parse(calls[1].options.body), { id: "wall-1", name: "Edited Wall" });
});

test("media updates use PUT with JSON", async () => {
  const calls = [];
  const fetchImpl = async (url, options) => { calls.push({ url, options }); return Response.json({ id: "media-1" }); };
  await updateMedia("http://localhost:8080/api/v1", { id: "media-1", kind: "topo" }, fetchImpl);
  assert.equal(calls[0].options.method, "PUT");
  assert.deepEqual(JSON.parse(calls[0].options.body), { id: "media-1", kind: "topo" });
});

test("publish request includes the admin bearer token when configured", async () => {
  const calls = [];
  const fetchImpl = async (url, options) => {
    calls.push({ url, options });
    return Response.json({ version: 2 });
  };

  await publishAreaPack("http://localhost:8080/api/v1", "area-1", fetchImpl);
  assert.equal(calls[0].options.method, "POST");
  assert.ok("headers" in calls[0].options);
});

test("review request sends expected JSON payload", async () => {
  const calls = [];
  const fetchImpl = async (url, options) => {
    calls.push({ url, options });
    return Response.json({ id: "capture-1", review_status: "good_candidate" });
  };

  await reviewCalibrationCapture(
    "http://localhost:8080/api/v1",
    "capture-1",
    "good_candidate",
    " looks good ",
    fetchImpl,
  );

  assert.equal(calls[0].options.method, "POST");
  assert.deepEqual(JSON.parse(calls[0].options.body), {
    review_status: "good_candidate",
    reviewer_notes: "looks good",
  });
});

test("route and overlay updates use PUT with JSON bodies", async () => {
  const calls = [];
  const fetchImpl = async (url, options) => {
    calls.push({ url, options });
    return Response.json(JSON.parse(options.body));
  };

  await updateRoute(
    "http://localhost:8080/api/v1",
    { id: "route-1", name: "Edited Route" },
    fetchImpl,
  );
  await updateOverlay(
    "http://localhost:8080/api/v1",
    { id: "overlay-1", confidence: "field_tested" },
    fetchImpl,
  );

  assert.equal(calls[0].options.method, "PUT");
  assert.equal(calls[0].options.headers["Content-Type"], "application/json");
  assert.deepEqual(JSON.parse(calls[0].options.body), {
    id: "route-1",
    name: "Edited Route",
  });
  assert.equal(calls[1].options.method, "PUT");
  assert.deepEqual(JSON.parse(calls[1].options.body), {
    id: "overlay-1",
    confidence: "field_tested",
  });
});

test("guidebook create requests use POST with JSON bodies", async () => {
  const calls = [];
  const fetchImpl = async (url, options) => {
    calls.push({ url, options });
    return Response.json(JSON.parse(options.body));
  };

  await createArea("http://localhost:8080/api/v1", { id: "area-1" }, fetchImpl);
  await createWall("http://localhost:8080/api/v1", { id: "wall-1" }, fetchImpl);
  await createRoute("http://localhost:8080/api/v1", { id: "route-1" }, fetchImpl);
  await createOverlay("http://localhost:8080/api/v1", { id: "overlay-1" }, fetchImpl);

  assert.deepEqual(
    calls.map((call) => call.options.method),
    ["POST", "POST", "POST", "POST"],
  );
  assert.deepEqual(JSON.parse(calls[2].options.body), { id: "route-1" });
  assert.equal(calls[3].url, "http://localhost:8080/api/v1/admin/ar-overlays");
});

test("apply request posts to overlay endpoint", async () => {
  const calls = [];
  const fetchImpl = async (url, options) => {
    calls.push({ url, options });
    return Response.json({ id: "overlay-1" });
  };

  await applyCalibrationCapture(
    "http://localhost:8080/api/v1",
    "overlay-1",
    "capture-1",
    fetchImpl,
  );

  assert.equal(
    calls[0].url,
    "http://localhost:8080/api/v1/admin/ar-overlays/overlay-1/apply-calibration/capture-1",
  );
  assert.equal(calls[0].options.method, "POST");
});

test("format helpers make review data readable", () => {
  assert.equal(formatReviewStatus("good_candidate"), "Good Candidate");
  assert.equal(
    formatAlignment({
      horizontal_offset_meters: 0.1,
      vertical_offset_meters: -0.2,
      depth_offset_meters: 0,
      scale: 1.05,
    }),
    "x +0.1m  y -0.2m  z +0.0m  scale 1.05x",
  );
});

test("trace helpers serialize and parse route trace points", () => {
  const text = tracePointsToText([
    { x: 0.1, y: 0.9, z: null },
    { x: 0.2, y: 0.7, z: -1.5 },
  ]);

  assert.equal(text, "0.1,0.9,\n0.2,0.7,-1.5");
  assert.deepEqual(parseTracePoints(text), [
    { x: 0.1, y: 0.9, z: null },
    { x: 0.2, y: 0.7, z: -1.5 },
  ]);
});

test("trace parser rejects incomplete traces", () => {
  assert.throws(() => parseTracePoints("0.1,0.2"), /at least two points/);
  assert.throws(() => parseTracePoints("0.1\n0.2,0.3"), /must be x,y or x,y,z/);
});

test("normalized trace validation rejects points outside the image", () => {
  assert.throws(() => validateNormalizedTrace([{ x: 1.2, y: 0.5, z: null }]), /between 0 and 1/);
});

test("draft helpers create valid guidebook hierarchy payloads", () => {
  assert.equal(slugify(" The Main Wall! "), "the-main-wall");

  const area = draftArea("Test Area");
  const wall = draftWall("Test Wall", area);
  const route = draftRoute("Test Route", wall);
  const overlay = draftOverlay(route);

  assert.equal(area.slug, "test-area");
  assert.equal(wall.area_id, area.id);
  assert.equal(route.wall_id, wall.id);
  assert.equal(route.grade_system, "yosemite_decimal");
  assert.equal(overlay.route_id, route.id);
  assert.equal(overlay.version, 1);
  assert.equal(overlay.route_trace.points.length, 2);
});
