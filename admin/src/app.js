import {
  REVIEW_STATUSES,
  applyCalibrationCapture,
  listCalibrationCaptures,
  reviewCalibrationCapture,
} from "./api.js";
import { formatAlignment, formatDateTime, formatReviewStatus } from "./format.js";

const state = {
  apiBaseUrl: "http://127.0.0.1:8080/api/v1",
  captures: [],
  selectedCaptureId: null,
};

const elements = {
  apiForm: document.querySelector("#api-form"),
  apiBaseUrl: document.querySelector("#api-base-url"),
  routeFilter: document.querySelector("#route-filter"),
  overlayFilter: document.querySelector("#overlay-filter"),
  refreshButton: document.querySelector("#refresh-button"),
  statusText: document.querySelector("#status-text"),
  captureCount: document.querySelector("#capture-count"),
  captureList: document.querySelector("#capture-list"),
  captureDetail: document.querySelector("#capture-detail"),
  captureRowTemplate: document.querySelector("#capture-row-template"),
};

elements.apiForm.addEventListener("submit", (event) => {
  event.preventDefault();
  state.apiBaseUrl = elements.apiBaseUrl.value;
  loadCaptures();
});

elements.refreshButton.addEventListener("click", () => {
  loadCaptures();
});

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

    render();
    setStatus(`Loaded ${state.captures.length} calibration capture${state.captures.length === 1 ? "" : "s"}.`);
  } catch (error) {
    render();
    setStatus(`Unable to load captures: ${error.message}`);
  } finally {
    setBusy(false);
  }
}

function render() {
  renderCaptureList();
  renderCaptureDetail(selectedCapture());
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
      render();
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
  render();
}

function selectedCapture() {
  return state.captures.find((capture) => capture.id === state.selectedCaptureId) ?? null;
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
