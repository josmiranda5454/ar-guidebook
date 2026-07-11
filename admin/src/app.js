import {
  REVIEW_STATUSES,
  applyCalibrationCapture,
  archiveEntity,
  createArea,
  createMedia,
  createOverlay,
  createRoute,
  createWall,
  listAreas,
  listArchived,
  login,
  listCalibrationCaptures,
  publishAreaPack,
  reviewCalibrationCapture,
  updateOverlay,
  updateMedia,
  updateArea,
  updateWall,
  restoreArchived,
  updateRoute,
} from "./api.js";
import { draftArea, draftOverlay, draftRoute, draftWall } from "./drafts.js";
import { formatAlignment, formatDateTime, formatReviewStatus } from "./format.js";
import { parseTracePoints, tracePointsToText, validateNormalizedTrace } from "./trace.js";

const state = {
  activeView: "guidebook",
  apiBaseUrl: "http://127.0.0.1:8080/api/v1",
  areas: [],
  routes: [],
  selectedAreaId: null,
  selectedWallId: null,
  selectedRouteId: null,
  captures: [],
  selectedCaptureId: null,
  archived: [],
  authenticated: Boolean(globalThis.localStorage?.getItem("climbar-admin-token")),
};

const elements = {
  apiForm: document.querySelector("#api-form"),
  apiBaseUrl: document.querySelector("#api-base-url"),
  authForm: document.querySelector("#auth-form"),
  adminEmail: document.querySelector("#admin-email"),
  adminPassword: document.querySelector("#admin-password"),
  tabButtons: [...document.querySelectorAll(".tab-button")],
  routeFilter: document.querySelector("#route-filter"),
  overlayFilter: document.querySelector("#overlay-filter"),
  createAreaButton: document.querySelector("#create-area-button"),
  createWallButton: document.querySelector("#create-wall-button"),
  createRouteButton: document.querySelector("#create-route-button"),
  createOverlayButton: document.querySelector("#create-overlay-button"),
  refreshButton: document.querySelector("#refresh-button"),
  statusText: document.querySelector("#status-text"),
  guidebookView: document.querySelector("#guidebook-view"),
  calibrationView: document.querySelector("#calibration-view"),
  routeCount: document.querySelector("#route-count"),
  routeList: document.querySelector("#route-list"),
  archivedCount: document.querySelector("#archived-count"),
  archivedList: document.querySelector("#archived-list"),
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

elements.authForm.addEventListener("submit", async (event) => {
  event.preventDefault();
  try {
    await login(state.apiBaseUrl, elements.adminEmail.value, elements.adminPassword.value);
    state.authenticated = true;
    setStatus("Signed in.");
    await loadActiveView();
  } catch (error) {
    setStatus(`Unable to sign in: ${error.message}`);
  }
});

elements.refreshButton.addEventListener("click", () => {
  loadActiveView();
});

elements.createAreaButton.addEventListener("click", () => {
  openCreateForm("area");
});

elements.createWallButton.addEventListener("click", () => {
  openCreateForm("wall");
});

elements.createRouteButton.addEventListener("click", () => {
  openCreateForm("route");
});

elements.createOverlayButton.addEventListener("click", () => {
  createOverlayForSelectedRoute();
});

document.querySelector("#publish-pack-button").addEventListener("click", () => {
  publishSelectedArea();
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
    if (state.authenticated) {
      try { state.archived = await listArchived(state.apiBaseUrl); }
      catch { state.archived = []; state.authenticated = false; }
    } else {
      state.archived = [];
    }
    state.routes = flattenRoutes(state.areas);

    if (!state.routes.some(({ route }) => route.id === state.selectedRouteId)) {
      state.selectedRouteId = state.routes[0]?.route.id ?? null;
    }
    if (!state.areas.some((area) => area.id === state.selectedAreaId)) {
      state.selectedAreaId = state.areas[0]?.id ?? null;
    }
    if (!state.areas.some((area) => area.walls.some((wall) => wall.id === state.selectedWallId))) {
      state.selectedWallId = state.areas.find((area) => area.id === state.selectedAreaId)?.walls[0]?.id ?? null;
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

async function publishSelectedArea() {
  const entry = selectedRouteEntry();
  const area = entry?.area ?? state.areas[0];
  if (!area) { setStatus("Load an area before publishing."); return; }
  setBusy(true, "Publishing offline pack...");
  try {
    const pack = await publishAreaPack(state.apiBaseUrl, area.id);
    setStatus(`Published ${area.name} offline pack v${pack.version}.`);
  } catch (error) {
    setStatus(`Unable to publish offline pack: ${error.message}`);
  } finally { setBusy(false); }
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
  document.querySelector(".guidebook-only").classList.toggle("hidden", state.activeView !== "guidebook");
  document.querySelector(".calibration-only").classList.toggle("hidden", state.activeView !== "calibration");
}

function renderGuidebook() {
  renderActiveView();
  renderRouteList();
  renderArchivedList();
  const entry = selectedRouteEntry();
  if (entry) {
    renderRouteEditor(entry);
  } else if (selectedWall()) {
    renderWallEditor(selectedWall());
  } else {
    renderAreaEditor(selectedArea());
  }
}

function renderArchivedList() {
  elements.archivedCount.textContent = String(state.archived.length);
  elements.archivedList.replaceChildren();
  if (state.archived.length === 0) {
    const empty = document.createElement("p"); empty.className = "empty-state"; empty.textContent = "Nothing archived."; elements.archivedList.append(empty); return;
  }
  for (const entry of state.archived) {
    const row = document.createElement("div"); row.className = "archived-row";
    row.innerHTML = `<div><strong>${escapeHtml(entry.name)}</strong><span>${escapeHtml(entry.entity_type)}</span></div><button type="button">Restore</button>`;
    row.querySelector("button").addEventListener("click", async () => {
      setBusy(true, `Restoring ${entry.entity_type}...`);
      try { await restoreArchived(state.apiBaseUrl, entry.id); await loadGuidebook(); setStatus(`${entry.entity_type} restored.`); }
      catch (error) { setStatus(`Unable to restore: ${error.message}`); }
      finally { setBusy(false); }
    });
    elements.archivedList.append(row);
  }
}

function renderCalibration() {
  renderActiveView();
  renderCaptureList();
  renderCaptureDetail(selectedCapture());
}

function renderRouteList() {
  elements.routeCount.textContent = String(state.routes.length);
  elements.routeList.replaceChildren();

  if (state.areas.length === 0) {
    const empty = document.createElement("p");
    empty.className = "empty-state";
    empty.textContent = "No routes found.";
    elements.routeList.append(empty);
    return;
  }

  for (const area of state.areas) {
    const areaRow = hierarchyRow(area.name, "Area", area.id === state.selectedAreaId);
    areaRow.addEventListener("click", () => {
      state.selectedAreaId = area.id;
      state.selectedWallId = area.walls[0]?.id ?? null;
      state.selectedRouteId = null;
      renderGuidebook();
    });
    elements.routeList.append(areaRow);

    for (const wall of area.walls) {
      const wallRow = hierarchyRow(wall.name, `Wall · ${area.name}`, wall.id === state.selectedWallId);
      wallRow.classList.add("hierarchy-child");
      wallRow.addEventListener("click", () => {
        state.selectedAreaId = area.id;
        state.selectedWallId = wall.id;
        state.selectedRouteId = null;
        renderGuidebook();
      });
      elements.routeList.append(wallRow);

      for (const route of wall.routes) {
        const routeRow = hierarchyRow(route.name, `${area.name} / ${wall.name}`, route.id === state.selectedRouteId);
        routeRow.classList.add("hierarchy-child", "hierarchy-route");
        routeRow.innerHTML += `<span class="review-status">${escapeHtml(route.grade)} • ${route.ar_overlays.length} overlay${route.ar_overlays.length === 1 ? "" : "s"}</span>`;
        routeRow.addEventListener("click", () => {
          state.selectedAreaId = area.id;
          state.selectedWallId = wall.id;
          state.selectedRouteId = route.id;
          renderGuidebook();
        });
        elements.routeList.append(routeRow);
      }
    }
  }
}

function hierarchyRow(name, context, selected) {
  const row = document.createElement("button");
  row.className = "capture-row hierarchy-row";
  row.type = "button";
  row.setAttribute("aria-selected", String(selected));
  row.innerHTML = `<span class="route-name">${escapeHtml(name)}</span><span class="capture-meta">${escapeHtml(context)}</span>`;
  return row;
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
  const media = route.media[0] ?? null;

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
      ${media ? mediaEditor(media) : ""}

      <div class="actions">
        <button type="submit">Save Route</button>
        ${overlay ? '<button id="save-overlay-button" class="secondary" type="button">Save Overlay</button>' : ""}
        ${media ? '<button id="save-media-button" class="secondary" type="button">Save Media</button>' : ""}
        <button id="add-media-button" class="secondary" type="button">Add Media</button>
        <button id="archive-route-button" class="danger" type="button">Archive Route</button>
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
  if (overlay) setupTraceCanvas(overlay);
  document.querySelector("#save-media-button")?.addEventListener("click", async () => {
    await saveMedia(media);
  });
  document.querySelector("#add-media-button")?.addEventListener("click", () => openMediaCreateForm(route));
  document.querySelector("#archive-route-button")?.addEventListener("click", async () => {
    await archiveSelected("route", route.id);
  });
}

function openMediaCreateForm(route) {
  elements.routeEditor.innerHTML = `<form id="media-create-form" class="editor-form"><section class="form-section"><h2>Add Media Asset</h2><p class="muted">Store a photo, topo, or video reference for offline publishing.</p><div class="form-grid">${selectField("new-media-kind", "Kind", ["photo", "topo", "video"], "photo")}${inputField("new-media-title", "Title", "")}${inputField("new-media-url", "Source URL", "", "url")}${inputField("new-media-offline-path", "Offline path", "")}</div></section><div class="actions"><button type="submit">Add Media</button><button id="cancel-media-button" class="secondary" type="button">Cancel</button></div></form>`;
  document.querySelector("#media-create-form").addEventListener("submit", async (event) => {
    event.preventDefault();
    const title = value("new-media-title"); const url = value("new-media-url");
    if (!title || !url) { setStatus("Media title and source URL are required."); return; }
    setBusy(true, "Adding media...");
    try {
      await createMedia(state.apiBaseUrl, route.id, { id: crypto.randomUUID(), kind: value("new-media-kind"), title, url, offline_path: optionalText("new-media-offline-path") });
      await loadGuidebook();
      setStatus("Media asset added.");
    } catch (error) { setStatus(`Unable to add media: ${error.message}`); }
    finally { setBusy(false); }
  });
  document.querySelector("#cancel-media-button").addEventListener("click", () => renderGuidebook());
}

function openCreateForm(kind) {
  const parent = kind === "wall" ? selectedArea() ?? state.areas[0] : kind === "route" ? selectedWall() ?? firstWall() : null;
  if ((kind === "wall" || kind === "route") && !parent) {
    setStatus(`Create a ${kind === "wall" ? "area" : "wall"} first.`);
    return;
  }

  const label = kind[0].toUpperCase() + kind.slice(1);
  elements.routeEditor.innerHTML = `<form id="create-entity-form" class="editor-form"><section class="form-section"><div class="detail-header"><div><h2>New ${label}</h2><p class="muted">${parent ? `Under ${escapeHtml(parent.name)}` : "Add a guidebook area."}</p></div></div><div class="form-grid">${inputField("new-entity-name", "Name", "")} ${textareaField("new-entity-description", "Description", "")} ${kind === "area" ? textareaField("new-entity-notes", "Access notes", "") : kind === "wall" ? textareaField("new-entity-notes", "Approach notes", "") : inputField("new-entity-grade", "Grade", "5.7")}</div></section><div class="actions"><button type="submit">Create ${label}</button><button id="cancel-create-button" class="secondary" type="button">Cancel</button></div></form>`;

  document.querySelector("#create-entity-form").addEventListener("submit", async (event) => {
    event.preventDefault();
    const name = value("new-entity-name");
    if (!name) { setStatus("Name is required."); return; }
    setBusy(true, `Creating ${kind}...`);
    try {
      let created;
      if (kind === "area") {
        created = await createArea(state.apiBaseUrl, { ...draftArea(name), description: value("new-entity-description"), access_notes: optionalText("new-entity-notes") });
        state.areas = [...state.areas, created];
        state.selectedAreaId = created.id; state.selectedWallId = null; state.selectedRouteId = null;
      } else if (kind === "wall") {
        created = await createWall(state.apiBaseUrl, { ...draftWall(name, parent), description: value("new-entity-description"), approach_notes: optionalText("new-entity-notes") });
        state.areas = state.areas.map((area) => area.id === parent.id ? { ...area, walls: [...area.walls, created] } : area);
        state.selectedAreaId = parent.id; state.selectedWallId = created.id; state.selectedRouteId = null;
      } else {
        created = await createRoute(state.apiBaseUrl, { ...draftRoute(name, parent), grade: value("new-entity-grade"), description: value("new-entity-description") });
        state.areas = state.areas.map((area) => ({ ...area, walls: area.walls.map((wall) => wall.id === parent.id ? { ...wall, routes: [...wall.routes, created] } : wall) }));
        state.selectedAreaId = state.areas.find((area) => area.walls.some((wall) => wall.id === parent.id))?.id ?? null;
        state.selectedWallId = parent.id; state.selectedRouteId = created.id;
      }
      state.routes = flattenRoutes(state.areas);
      renderGuidebook();
      setStatus(`${label} "${created.name}" created.`);
    } catch (error) { setStatus(`Unable to create ${kind}: ${error.message}`); }
    finally { setBusy(false); }
  });
  document.querySelector("#cancel-create-button").addEventListener("click", () => renderGuidebook());
}

function renderAreaEditor(area) {
  if (!area) {
    elements.routeEditor.innerHTML = `<div class="empty-state"><h2>Select an area</h2><p>Choose an area, wall, or route from the guidebook hierarchy.</p></div>`;
    return;
  }
  elements.routeEditor.innerHTML = entityForm("area", area, `Area: ${area.name}`, [
    inputField("entity-name", "Name", area.name), inputField("entity-slug", "Slug", area.slug),
    textareaField("entity-description", "Description", area.description), textareaField("entity-notes", "Access notes", area.access_notes ?? ""),
    geoFields(area.location),
  ]);
  bindEntityForm("area", area);
}

function renderWallEditor(wall) {
  elements.routeEditor.innerHTML = entityForm("wall", wall, `Wall: ${wall.name}`, [
    inputField("entity-name", "Name", wall.name), inputField("entity-slug", "Slug", wall.slug),
    textareaField("entity-description", "Description", wall.description), textareaField("entity-notes", "Approach notes", wall.approach_notes ?? ""),
    inputField("entity-aspect", "Aspect", wall.aspect ?? ""), geoFields(wall.location),
  ]);
  bindEntityForm("wall", wall);
}

function entityForm(kind, entity, title, fields) {
  const childCount = entity.walls?.length ?? entity.routes?.length ?? 0;
  return `<form id="entity-editor-form" class="editor-form"><section class="form-section"><div class="detail-header"><div><h2>${escapeHtml(title)}</h2><p class="muted">Edit the ${kind} properties defined by the guidebook schema.</p></div><span class="badge">${childCount} ${childCount === 1 ? "child" : "children"}</span></div><div class="form-grid">${fields.join("")}</div></section><div class="actions"><button type="submit">Save ${kind}</button><button id="archive-entity-button" class="danger" type="button">Archive ${kind}</button></div></form>`;
}

function geoFields(location) {
  return [inputField("entity-latitude", "Latitude", location.latitude, "number", "0.000001"), inputField("entity-longitude", "Longitude", location.longitude, "number", "0.000001"), inputField("entity-elevation", "Elevation meters", location.elevation_meters ?? "", "number", "0.1")].join("");
}

function bindEntityForm(kind, entity) {
  document.querySelector("#entity-editor-form").addEventListener("submit", async (event) => {
    event.preventDefault();
    const location = { latitude: requiredNumber("entity-latitude"), longitude: requiredNumber("entity-longitude"), elevation_meters: optionalNumber("entity-elevation") };
    const payload = { ...entity, name: value("entity-name"), slug: value("entity-slug"), description: value("entity-description"), location };
    if (kind === "area") payload.access_notes = optionalText("entity-notes");
    if (kind === "wall") { payload.approach_notes = optionalText("entity-notes"); payload.aspect = optionalText("entity-aspect"); }
    setBusy(true, `Saving ${kind}...`);
    try {
      const updated = kind === "area" ? await updateArea(state.apiBaseUrl, payload) : await updateWall(state.apiBaseUrl, payload);
      if (kind === "area") state.areas = state.areas.map((area) => area.id === updated.id ? updated : area);
      else state.areas = state.areas.map((area) => ({ ...area, walls: area.walls.map((wall) => wall.id === updated.id ? updated : wall) }));
      state.routes = flattenRoutes(state.areas);
      renderGuidebook();
      setStatus(`${kind[0].toUpperCase()}${kind.slice(1)} saved.`);
    } catch (error) { setStatus(`Unable to save ${kind}: ${error.message}`); }
    finally { setBusy(false); }
  });
  document.querySelector("#archive-entity-button").addEventListener("click", async () => {
    await archiveSelected(kind, entity.id);
  });
}

async function archiveSelected(kind, id) {
  setBusy(true, `Archiving ${kind}...`);
  try {
    await archiveEntity(state.apiBaseUrl, kind, id);
    await loadGuidebook();
    setStatus(`${kind[0].toUpperCase()}${kind.slice(1)} archived.`);
  } catch (error) { setStatus(`Unable to archive ${kind}: ${error.message}`); }
  finally { setBusy(false); }
}

function mediaEditor(media) {
  return `<section class="form-section"><h3>Photo / Topo Asset</h3><div class="form-grid">
    ${selectField("media-kind", "Kind", ["photo", "topo", "video"], media.kind)}
    ${inputField("media-title", "Title", media.title)}
    ${inputField("media-url", "Source URL", media.url, "url")}
    ${inputField("media-offline-path", "Offline path", media.offline_path ?? "")}
  </div></section>`;
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
      <p class="muted">Editing overlay v${overlay.version}. Trace points use one point per line as x,y,z. The z value is optional.</p>

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
        ${selectField("trace-coordinate-space", "Trace coordinate space", ["normalized_wall_image", "local_wall_meters"], overlay.route_trace.coordinate_space)}
        <div class="trace-editor"><canvas id="trace-canvas" width="640" height="360" aria-label="Visual route trace editor"></canvas><p class="muted">Click the wall image area to add trace points. The text field remains available for precise edits.</p></div>
        ${textareaField("trace-points", "Trace points", tracePointsToText(overlay.route_trace.points))}
      </div>
    </section>
  `;
}

function setupTraceCanvas(overlay) {
  const canvas = document.querySelector("#trace-canvas");
  if (!canvas || overlay.route_trace.coordinate_space !== "normalized_wall_image") return;
  const context = canvas.getContext("2d");
  const draw = () => {
    const points = parseTracePoints(value("trace-points"));
    context.clearRect(0, 0, canvas.width, canvas.height);
    context.fillStyle = "#e8e3d7"; context.fillRect(0, 0, canvas.width, canvas.height);
    context.strokeStyle = "rgba(40, 40, 40, .18)"; context.lineWidth = 1;
    for (let x = 0; x <= canvas.width; x += 64) { context.beginPath(); context.moveTo(x, 0); context.lineTo(x, canvas.height); context.stroke(); }
    for (let y = 0; y <= canvas.height; y += 60) { context.beginPath(); context.moveTo(0, y); context.lineTo(canvas.width, y); context.stroke(); }
    if (points.length < 2) return;
    context.strokeStyle = "#f3b51b"; context.lineWidth = 5; context.beginPath();
    points.forEach((point, index) => { const x = point.x * canvas.width; const y = point.y * canvas.height; index === 0 ? context.moveTo(x, y) : context.lineTo(x, y); });
    context.stroke();
    context.fillStyle = "#f3b51b";
    points.forEach((point) => { context.beginPath(); context.arc(point.x * canvas.width, point.y * canvas.height, 6, 0, Math.PI * 2); context.fill(); });
  };
  draw();
  canvas.addEventListener("click", (event) => {
    const rect = canvas.getBoundingClientRect();
    const point = { x: Math.max(0, Math.min(1, (event.clientX - rect.left) / rect.width)), y: Math.max(0, Math.min(1, (event.clientY - rect.top) / rect.height)), z: null };
    const points = parseTracePoints(value("trace-points"));
    points.push(point);
    document.querySelector("#trace-points").value = tracePointsToText(points);
    draw();
  });
  document.querySelector("#trace-points").addEventListener("input", () => { try { draw(); } catch {} });
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

async function createAreaFromPrompt() {
  const name = promptForName("New area name");
  if (!name) {
    return;
  }

  setBusy(true, "Creating area...");
  try {
    const area = await createArea(state.apiBaseUrl, draftArea(name));
    state.areas = [...state.areas, area];
    state.routes = flattenRoutes(state.areas);
    state.selectedAreaId = area.id;
    state.selectedWallId = null;
    state.selectedRouteId = null;
    renderGuidebook();
    setStatus(`Created area "${area.name}".`);
  } catch (error) {
    setStatus(`Unable to create area: ${error.message}`);
  } finally {
    setBusy(false);
  }
}

async function createWallFromPrompt() {
  const area = selectedArea() ?? state.areas[0];
  if (!area) {
    setStatus("Create an area before creating a wall.");
    return;
  }

  const name = promptForName(`New wall name under ${area.name}`);
  if (!name) {
    return;
  }

  setBusy(true, "Creating wall...");
  try {
    const wall = await createWall(state.apiBaseUrl, draftWall(name, area));
    state.areas = state.areas.map((existingArea) =>
      existingArea.id === area.id
        ? { ...existingArea, walls: [...existingArea.walls, wall] }
        : existingArea,
    );
    state.routes = flattenRoutes(state.areas);
    state.selectedAreaId = area.id;
    state.selectedWallId = wall.id;
    state.selectedRouteId = null;
    renderGuidebook();
    setStatus(`Created wall "${wall.name}".`);
  } catch (error) {
    setStatus(`Unable to create wall: ${error.message}`);
  } finally {
    setBusy(false);
  }
}

async function createRouteFromPrompt() {
  const entry = selectedRouteEntry();
  const wall = selectedWall() ?? entry?.wall ?? firstWall();
  if (!wall) {
    setStatus("Create a wall before creating a route.");
    return;
  }

  const name = promptForName(`New route name under ${wall.name}`);
  if (!name) {
    return;
  }

  setBusy(true, "Creating route...");
  try {
    const route = await createRoute(state.apiBaseUrl, draftRoute(name, wall));
    state.areas = state.areas.map((area) => ({
      ...area,
      walls: area.walls.map((existingWall) =>
        existingWall.id === wall.id
          ? { ...existingWall, routes: [...existingWall.routes, route] }
          : existingWall,
      ),
    }));
    state.routes = flattenRoutes(state.areas);
    state.selectedAreaId = state.areas.find((area) => area.walls.some((existingWall) => existingWall.id === wall.id))?.id ?? state.selectedAreaId;
    state.selectedWallId = wall.id;
    state.selectedRouteId = route.id;
    renderGuidebook();
    setStatus(`Created route "${route.name}".`);
  } catch (error) {
    setStatus(`Unable to create route: ${error.message}`);
  } finally {
    setBusy(false);
  }
}

async function createOverlayForSelectedRoute() {
  const entry = selectedRouteEntry();
  if (!entry) {
    setStatus("Select or create a route before creating an overlay.");
    return;
  }

  setBusy(true, "Creating AR overlay...");
  try {
    const overlay = await createOverlay(state.apiBaseUrl, draftOverlay(entry.route));
    replaceRoute({
      ...entry.route,
      ar_overlays: [overlay, ...entry.route.ar_overlays],
    });
    setStatus(`Created AR overlay v${overlay.version}.`);
  } catch (error) {
    setStatus(`Unable to create overlay: ${error.message}`);
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

async function saveMedia(media) {
  setBusy(true, "Saving media metadata...");
  try {
    const updated = await updateMedia(state.apiBaseUrl, {
      ...media,
      kind: value("media-kind"),
      title: value("media-title"),
      url: value("media-url"),
      offline_path: optionalText("media-offline-path"),
    });
    state.routes = state.routes.map((entry) => entry.route.id === updated.route_id ? entry : entry);
    const route = selectedRouteEntry()?.route;
    if (route) {
      route.media = route.media.map((item) => item.id === updated.id ? updated : item);
    }
    renderGuidebook();
    setStatus("Media metadata saved.");
  } catch (error) { setStatus(`Unable to save media: ${error.message}`); }
  finally { setBusy(false); }
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
    route_trace: readTraceForm(),
  };
}

function readTraceForm() {
  const coordinateSpace = value("trace-coordinate-space");
  const points = parseTracePoints(value("trace-points"));
  return {
    coordinate_space: coordinateSpace,
    points: coordinateSpace === "normalized_wall_image" ? validateNormalizedTrace(points) : points,
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

function selectedArea() {
  return state.areas.find((area) => area.id === state.selectedAreaId) ?? null;
}

function selectedWall() {
  return state.areas.flatMap((area) => area.walls).find((wall) => wall.id === state.selectedWallId) ?? null;
}

function selectedCapture() {
  return state.captures.find((capture) => capture.id === state.selectedCaptureId) ?? null;
}

function firstWall() {
  return state.areas.flatMap((area) => area.walls)[0] ?? null;
}

function promptForName(message) {
  const name = window.prompt(message);
  return name?.trim() || null;
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
  elements.createAreaButton.disabled = isBusy;
  elements.createWallButton.disabled = isBusy;
  elements.createRouteButton.disabled = isBusy;
  elements.createOverlayButton.disabled = isBusy;
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
