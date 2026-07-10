import {
  REVIEW_STATUSES,
  applyCalibrationCapture,
  listAreas,
  listCalibrationCaptures,
  reviewCalibrationCapture,
  updateOverlay,
  updateRoute,
} from "./api.js";
import { formatAlignment, formatDateTime, formatReviewStatus } from "./format.js";

const state = {
  activeView: "guidebook",
  apiBaseUrl: "http://127.0.0.1:8080/api/v1",
  areas: [],
  routes: [],
  selectedRouteId: null,
  captures: [],
  selectedCaptureId: null,
};

const elements = {
  apiForm: document.querySelector("#api-form"),
  apiBaseUrl: document.querySelector("#api-base-url"),
  tabButtons: [...document.querySelectorAll(".tab-button")],
  routeFilter: document.querySelector("#route-filter"),
  overlayFilter: document.querySelector("#overlay-filter"),
  refreshButton: document.querySelector("#refresh-button"),
  statusText: document.querySelector("#status-text"),
  guidebookView: document.querySelector("#guidebook-view"),
  calibrationView: document.querySelector("#calibration-view"),
  routeCount: document.querySelector("#route-count"),
  routeList: document.querySelector("#route-list"),
  routeEditor: document.querySelector("#route-editor"),
  captureCount: document.querySelector("#capture-count"),
  captureList: document.querySelector("#capture-list"),
  captureDetail: document.querySelector("#capture-detail"),
  captureRowTemplate: document.querySelector("#capture-row-template"),
};

elements.apiForm.addEventListener("submit", (event) => {
  event.preventDefault();
  state.apiBaseUrl = elements.apiBaseUrl.value;
  loadActiveView();
});

elements.refreshButton.addEventListener("click", () => {
  loadActiveView();
});

for (const button of elements.tabButtons) {
  button.addEventListener("click", () => {
    state.activeView = button.dataset.view;
    renderActiveView();
    loadActiveView();
  });
}

loadGuidebook();

async function loadActiveView() {
  if (state.activeView === "guidebook") {
    await loadGuidebook();
  } else {
    await loadCaptures();
  }
}

async function loadGuidebook() {
  setBusy(true, "Loading guidebook data...");

  try {
    state.areas = await listAreas(state.apiBaseUrl);
    state.routes = flattenRoutes(state.areas);

    if (!state.routes.some(({ route }) => route.id === state.selectedRouteId)) {
      state.selectedRouteId = state.routes[0]?.route.id ?? null;
    }

    renderGuidebook();
    setStatus(`Loaded ${state.routes.length} route${state.routes.length === 1 ? "" : "s"}.`);
  } catch (error) {
    renderGuidebook();
    setStatus(`Unable to load guidebook data: ${error.message}`);
  } finally {
    setBusy(false);
  }
}

async function loadCaptures() {
  setBusy(true, "Loading calibration captures...");

  try {
    state.captures = await listCalibrationCaptures(state.apiBaseUrl, {
      routeId: elements.routeFilter.value,
      overlayId: elements.overlayFilter.value,
    });

    if (!state.captures.some((capture) => capture.id === state.selectedCaptureId)) {
      state.selectedCaptureId = state.captures[0]?.id ?? null;
    }

    renderCalibration();
    setStatus(`Loaded ${state.captures.length} calibration capture${state.captures.length === 1 ? "" : "s"}.`);
  } catch (error) {
    renderCalibration();
    setStatus(`Unable to load captures: ${error.message}`);
  } finally {
    setBusy(false);
  }
}

function renderActiveView() {
  for (const button of elements.tabButtons) {
    button.setAttribute("aria-selected", String(button.dataset.view === state.activeView));
  }

  elements.guidebookView.classList.toggle("hidden", state.activeView !== "guidebook");
  elements.calibrationView.classList.toggle("hidden", state.activeView !== "calibration");
  document.querySelector(".calibration-only").classList.toggle("hidden", state.activeView !== "calibration");
}

function renderGuidebook() {
  renderActiveView();
  renderRouteList();
  renderRouteEditor(selectedRouteEntry());
}

function renderCalibration() {
  renderActiveView();
  renderCaptureList();
  renderCaptureDetail(selectedCapture());
}

function renderRouteList() {
  elements.routeCount.textContent = String(state.routes.length);
  elements.routeList.replaceChildren();

  if (state.routes.length === 0) {
    const empty = document.createElement("p");
    empty.className = "empty-state";
    empty.textContent = "No routes found.";
    elements.routeList.append(empty);
    return;
  }

  for (const entry of state.routes) {
    const row = document.createElement("button");
    row.className = "capture-row";
    row.type = "button";
    row.setAttribute("aria-selected", String(entry.route.id === state.selectedRouteId));
    row.innerHTML = `
      <span class="route-name">${escapeHtml(entry.route.name)}</span>
      <span class="capture-meta">${escapeHtml(entry.area.name)} / ${escapeHtml(entry.wall.name)}</span>
      <span class="review-status">${escapeHtml(entry.route.grade)} • ${entry.route.ar_overlays.length} overlay${entry.route.ar_overlays.length === 1 ? "" : "s"}</span>
    `;
    row.addEventListener("click", () => {
      state.selectedRouteId = entry.route.id;
      renderGuidebook();
    });
    elements.routeList.append(row);
  }
}

function renderRouteEditor(entry) {
  if (!entry) {
    elements.routeEditor.innerHTML = `
      <div class="empty-state">
        <h2>No route selected</h2>
        <p>Load guidebook data, then select a route to edit route and overlay fields.</p>
      </div>
    `;
    return;
  }

  const route = entry.route;
  const overlay = route.ar_overlays[0] ?? null;

  elements.routeEditor.innerHTML = `
    <form id="route-editor-form" class="editor-form">
      <section class="form-section">
        <div class="detail-header">
          <div>
            <h2>${escapeHtml(route.name)}</h2>
            <p class="muted">${escapeHtml(entry.area.name)} / ${escapeHtml(entry.wall.name)}</p>
          </div>
          <span class="badge">${escapeHtml(route.grade)}</span>
        </div>

        <div class="form-grid">
          ${inputField("route-name", "Name", route.name)}
          ${inputField("route-slug", "Slug", route.slug)}
          ${inputField("route-grade", "Grade", route.grade)}
          ${inputField("route-types", "Types", route.route_types.join(", "))}
          ${inputField("length-feet", "Length feet", route.length_feet ?? "", "number")}
          ${inputField("pitches", "Pitches", route.pitches ?? "", "number")}
          ${inputField("stars-average", "Stars average", route.stars_average ?? "", "number", "0.1")}
          ${inputField("rating-votes", "Rating votes", route.rating_votes, "number")}
          ${textareaField("description", "Description", route.description)}
          ${textareaField("location-notes", "Location notes", route.location_notes)}
          ${textareaField("protection-notes", "Protection notes", route.protection_notes ?? "")}
          ${textareaField("safety-notes", "Safety notes", route.safety_notes ?? "")}
        </div>
      </section>

      ${overlay ? overlayEditor(overlay) : noOverlayEditor()}

      <div class="actions">
        <button type="submit">Save Route</button>
        ${overlay ? '<button id="save-overlay-button" class="secondary" type="button">Save Overlay</button>' : ""}
      </div>
    </form>
  `;

  document.querySelector("#route-editor-form").addEventListener("submit", async (event) => {
    event.preventDefault();
    await saveRoute(route);
  });

  document.querySelector("#save-overlay-button")?.addEventListener("click", async () => {
    await saveOverlay(overlay);
  });
}

function overlayEditor(overlay) {
  const alignment = overlay.default_alignment ?? {
    horizontal_offset_meters: 0,
    vertical_offset_meters: 0,
    depth_offset_meters: 0,
    scale: 1,
  };

  return `
    <section class="form-section">
      <h3>AR Overlay</h3>
      <p class="muted">Editing overlay v${overlay.version}. Trace point editing comes next; this slice edits placement metadata.</p>

      <div class="form-grid">
        ${selectField("overlay-confidence", "Confidence", ["draft", "field_tested", "reviewed"], overlay.confidence)}
        ${inputField("overlay-bearing", "Compass bearing", overlay.compass_bearing_degrees ?? "", "number", "0.1")}
        ${inputField("gps-latitude", "GPS latitude", overlay.gps_hint.latitude, "number", "0.000001")}
        ${inputField("gps-longitude", "GPS longitude", overlay.gps_hint.longitude, "number", "0.000001")}
        ${inputField("gps-elevation", "GPS elevation meters", overlay.gps_hint.elevation_meters ?? "", "number", "0.1")}
        ${selectField("anchor-strategy", "Anchor strategy", ["manual_alignment", "reference_image", "wall_plane_and_bearing"], overlay.anchor_strategy)}
        ${inputField("align-x", "Default align x", alignment.horizontal_offset_meters, "number", "0.1")}
        ${inputField("align-y", "Default align y", alignment.vertical_offset_meters, "number", "0.1")}
        ${inputField("align-z", "Default align z", alignment.depth_offset_meters, "number", "0.1")}
        ${inputField("align-scale", "Default scale", alignment.scale, "number", "0.05")}
      </div>
    </section>
  `;
}

function noOverlayEditor() {
  return `
    <section class="form-section">
      <h3>AR Overlay</h3>
      <p class="muted">This route does not have an AR overlay yet.</p>
    </section>
  `;
}

async function saveRoute(route) {
  setBusy(true, "Saving route...");

  try {
    const updatedRoute = await updateRoute(state.apiBaseUrl, readRouteForm(route));
    replaceRoute(updatedRoute);
    setStatus("Route saved.");
  } catch (error) {
    setStatus(`Unable to save route: ${error.message}`);
  } finally {
    setBusy(false);
  }
}

async function saveOverlay(overlay) {
  setBusy(true, "Saving AR overlay...");

  try {
    const updatedOverlay = await updateOverlay(state.apiBaseUrl, readOverlayForm(overlay));
    replaceOverlay(updatedOverlay);
    setStatus("AR overlay saved.");
  } catch (error) {
    setStatus(`Unable to save overlay: ${error.message}`);
  } finally {
    setBusy(false);
  }
}

function readRouteForm(route) {
  return {
    ...route,
    name: value("route-name"),
    slug: value("route-slug"),
    grade: value("route-grade"),
    route_types: value("route-types")
      .split(",")
      .map((item) => item.trim())
      .filter(Boolean),
    length_feet: optionalNumber("length-feet"),
    pitches: optionalNumber("pitches"),
    stars_average: optionalNumber("stars-average"),
    rating_votes: Number.parseInt(value("rating-votes") || "0", 10),
    description: value("description"),
    location_notes: value("location-notes"),
    protection_notes: optionalText("protection-notes"),
    safety_notes: optionalText("safety-notes"),
  };
}

function readOverlayForm(overlay) {
  return {
    ...overlay,
    anchor_strategy: value("anchor-strategy"),
    confidence: value("overlay-confidence"),
    compass_bearing_degrees: optionalNumber("overlay-bearing"),
    gps_hint: {
      latitude: requiredNumber("gps-latitude"),
      longitude: requiredNumber("gps-longitude"),
      elevation_meters: optionalNumber("gps-elevation"),
    },
    default_alignment: {
      horizontal_offset_meters: requiredNumber("align-x"),
      vertical_offset_meters: requiredNumber("align-y"),
      depth_offset_meters: requiredNumber("align-z"),
      scale: requiredNumber("align-scale"),
    },
  };
}

function replaceRoute(updatedRoute) {
  state.routes = state.routes.map((entry) =>
    entry.route.id === updatedRoute.id ? { ...entry, route: updatedRoute } : entry,
  );
  state.selectedRouteId = updatedRoute.id;
  renderGuidebook();
}

function replaceOverlay(updatedOverlay) {
  state.routes = state.routes.map((entry) => {
    if (entry.route.id !== updatedOverlay.route_id) {
      return entry;
    }

    return {
      ...entry,
      route: {
        ...entry.route,
        ar_overlays: entry.route.ar_overlays.map((overlay) =>
          overlay.id === updatedOverlay.id ? updatedOverlay : overlay,
        ),
      },
    };
  });
  renderGuidebook();
}

function renderCaptureList() {
  elements.captureCount.textContent = String(state.captures.length);
  elements.captureList.replaceChildren();

  if (state.captures.length === 0) {
    const empty = document.createElement("p");
    empty.className = "empty-state";
    empty.textContent = "No calibration captures found.";
    elements.captureList.append(empty);
    return;
  }

  for (const capture of state.captures) {
    const row = elements.captureRowTemplate.content.firstElementChild.cloneNode(true);
    row.querySelector(".route-name").textContent = capture.route_name;
    row.querySelector(".capture-meta").textContent = `Overlay v${capture.overlay_version} • ${formatDateTime(capture.captured_at)}`;
    row.querySelector(".review-status").textContent = formatReviewStatus(capture.review_status);
    row.setAttribute("aria-selected", String(capture.id === state.selectedCaptureId));
    row.addEventListener("click", () => {
      state.selectedCaptureId = capture.id;
      renderCalibration();
    });
    elements.captureList.append(row);
  }
}

function renderCaptureDetail(capture) {
  if (!capture) {
    elements.captureDetail.innerHTML = `
      <div class="empty-state">
        <h2>No capture selected</h2>
        <p>Load calibration captures, then select one to review or apply to an overlay.</p>
      </div>
    `;
    return;
  }

  elements.captureDetail.innerHTML = `
    <div class="detail-header">
      <div>
        <h2>${escapeHtml(capture.route_name)}</h2>
        <p class="muted">Captured ${formatDateTime(capture.captured_at)}</p>
      </div>
      <span class="badge">${formatReviewStatus(capture.review_status)}</span>
    </div>

    <div class="detail-grid">
      <div class="metric">
        <span>Horizontal</span>
        <strong>${capture.alignment.horizontal_offset_meters.toFixed(1)}m</strong>
      </div>
      <div class="metric">
        <span>Vertical</span>
        <strong>${capture.alignment.vertical_offset_meters.toFixed(1)}m</strong>
      </div>
      <div class="metric">
        <span>Depth</span>
        <strong>${capture.alignment.depth_offset_meters.toFixed(1)}m</strong>
      </div>
      <div class="metric">
        <span>Scale</span>
        <strong>${capture.alignment.scale.toFixed(2)}x</strong>
      </div>
    </div>

    <p class="muted">${formatAlignment(capture.alignment)}</p>

    <form id="review-form" class="review-form">
      <label>
        Review status
        <select id="review-status">
          ${REVIEW_STATUSES.map((status) => `
            <option value="${status}" ${status === capture.review_status ? "selected" : ""}>
              ${formatReviewStatus(status)}
            </option>
          `).join("")}
        </select>
      </label>

      <label>
        Reviewer notes
        <textarea id="reviewer-notes" placeholder="What made this capture good or unusable?">${escapeHtml(capture.reviewer_notes ?? "")}</textarea>
      </label>

      <div class="actions">
        <button type="submit">Save Review</button>
        <button id="apply-button" class="secondary" type="button">Apply to Overlay</button>
        <button id="reject-button" class="danger" type="button">Reject</button>
      </div>
    </form>

    <div class="id-grid">
      <div>
        <p class="muted">Capture ID</p>
        <code>${capture.id}</code>
      </div>
      <div>
        <p class="muted">Overlay ID</p>
        <code>${capture.overlay_id}</code>
      </div>
      <div>
        <p class="muted">Route ID</p>
        <code>${capture.route_id}</code>
      </div>
      <div>
        <p class="muted">Reviewed</p>
        <code>${formatDateTime(capture.reviewed_at)}</code>
      </div>
    </div>
  `;

  document.querySelector("#review-form").addEventListener("submit", async (event) => {
    event.preventDefault();
    await saveReview(capture.id);
  });

  document.querySelector("#apply-button").addEventListener("click", async () => {
    await applyCapture(capture);
  });

  document.querySelector("#reject-button").addEventListener("click", async () => {
    document.querySelector("#review-status").value = "rejected";
    await saveReview(capture.id);
  });
}

async function saveReview(captureId) {
  const reviewStatus = document.querySelector("#review-status").value;
  const reviewerNotes = document.querySelector("#reviewer-notes").value;
  setBusy(true, "Saving review...");

  try {
    const updatedCapture = await reviewCalibrationCapture(
      state.apiBaseUrl,
      captureId,
      reviewStatus,
      reviewerNotes,
    );
    replaceCapture(updatedCapture);
    setStatus("Review saved.");
  } catch (error) {
    setStatus(`Unable to save review: ${error.message}`);
  } finally {
    setBusy(false);
  }
}

async function applyCapture(capture) {
  setBusy(true, "Applying calibration to overlay...");

  try {
    await applyCalibrationCapture(state.apiBaseUrl, capture.overlay_id, capture.id);
    await loadCaptures();
    setStatus("Calibration applied to overlay default alignment.");
  } catch (error) {
    setStatus(`Unable to apply calibration: ${error.message}`);
  } finally {
    setBusy(false);
  }
}

function replaceCapture(updatedCapture) {
  state.captures = state.captures.map((capture) =>
    capture.id === updatedCapture.id ? updatedCapture : capture,
  );
  state.selectedCaptureId = updatedCapture.id;
  renderCalibration();
}

function flattenRoutes(areas) {
  return areas.flatMap((area) =>
    area.walls.flatMap((wall) =>
      wall.routes.map((route) => ({
        area,
        wall,
        route,
      })),
    ),
  );
}

function selectedRouteEntry() {
  return state.routes.find((entry) => entry.route.id === state.selectedRouteId) ?? null;
}

function selectedCapture() {
  return state.captures.find((capture) => capture.id === state.selectedCaptureId) ?? null;
}

function inputField(id, label, fieldValue, type = "text", step = "") {
  return `
    <label>
      ${label}
      <input id="${id}" type="${type}" ${step ? `step="${step}"` : ""} value="${escapeHtml(String(fieldValue))}" />
    </label>
  `;
}

function textareaField(id, label, fieldValue) {
  return `
    <label class="wide">
      ${label}
      <textarea id="${id}">${escapeHtml(String(fieldValue))}</textarea>
    </label>
  `;
}

function selectField(id, label, options, selectedValue) {
  return `
    <label>
      ${label}
      <select id="${id}">
        ${options.map((option) => `
          <option value="${option}" ${option === selectedValue ? "selected" : ""}>
            ${formatReviewStatus(option)}
          </option>
        `).join("")}
      </select>
    </label>
  `;
}

function value(id) {
  return document.querySelector(`#${id}`).value.trim();
}

function optionalText(id) {
  const text = value(id);
  return text.length > 0 ? text : null;
}

function optionalNumber(id) {
  const text = value(id);
  return text.length > 0 ? Number.parseFloat(text) : null;
}

function requiredNumber(id) {
  return Number.parseFloat(value(id));
}

function setStatus(message) {
  elements.statusText.textContent = message;
}

function setBusy(isBusy, message) {
  elements.refreshButton.disabled = isBusy;
  elements.apiForm.querySelector("button").disabled = isBusy;
  if (message) {
    setStatus(message);
  }
}

function escapeHtml(value) {
  return value.replace(/[&<>"']/g, (character) => {
    const entities = {
      "&": "&amp;",
      "<": "&lt;",
      ">": "&gt;",
      '"': "&quot;",
      "'": "&#039;",
    };
    return entities[character];
  });
}
