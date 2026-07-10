import assert from "node:assert/strict";
import { test } from "node:test";
import {
  applyCalibrationCapture,
  applyCaptureUrl,
  calibrationCaptureListUrl,
  reviewCalibrationCapture,
  reviewCaptureUrl,
} from "../src/api.js";
import { formatAlignment, formatReviewStatus } from "../src/format.js";

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
