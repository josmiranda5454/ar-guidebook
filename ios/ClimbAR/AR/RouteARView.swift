import ARKit
import CoreLocation
import RealityKit
import SwiftUI
import UIKit

private enum RouteARTrackingStatus: Equatable {
    case initializing
    case relocalizing
    case ready
    case limited(String)
    case unavailable

    var message: String {
        switch self {
        case .initializing: "Starting AR tracking..."
        case .relocalizing: "Restoring the saved wall reference..."
        case .ready: "Wall tracking ready"
        case .limited(let reason): "Move slowly: \(reason)"
        case .unavailable: "AR tracking is unavailable"
        }
    }

    var systemImage: String {
        switch self {
        case .initializing, .relocalizing: "location.viewfinder"
        case .ready: "checkmark.circle.fill"
        case .limited: "exclamationmark.triangle"
        case .unavailable: "xmark.circle"
        }
    }

    var color: Color {
        switch self {
        case .initializing, .relocalizing: .orange
        case .ready: .green
        case .limited: .orange
        case .unavailable: .red
        }
    }
}

struct RouteARView: View {
    let route: Route
    let overlay: RouteAROverlay

    @State private var alignment: RouteARAlignment
    @State private var captureCount: Int
    @State private var captureMessage: String?
    @State private var uploadMessage: String?
    @State private var isUploading = false
    @State private var latestCapture: RouteCalibrationCapture?
    @State private var latestCaptureJSON: String?
    @State private var isControlPanelExpanded = false
    @State private var isRecorderLoginPresented = false
    @State private var hasRecorderSession: Bool
    @State private var routeStartWorldPosition: SIMD3<Float>?
    @State private var isPlacingRouteStart = false
    @State private var anchorMessage: String?
    @State private var trackingStatus: RouteARTrackingStatus = .initializing
    @State private var hasRelocalizedSavedWorldMap = true
    @State private var hasSeenRelocalizingState = false
    @StateObject private var locationService = LocationService()
    private let initialSavedWorldMapLoaded: Bool

    init(route: Route, overlay: RouteAROverlay) {
        self.route = route
        self.overlay = overlay
        let savedAlignment = RouteARAlignmentStore.load(routeId: route.id, overlayId: overlay.id)
        _alignment = State(
            initialValue: savedAlignment ?? overlay.defaultAlignment ?? .zero
        )
        _captureCount = State(
            initialValue: RouteCalibrationCaptureStore.count(routeId: route.id, overlayId: overlay.id)
        )
        let latestCapture = RouteCalibrationCaptureStore.latest(routeId: route.id, overlayId: overlay.id)
        _latestCapture = State(
            initialValue: latestCapture
        )
        _latestCaptureJSON = State(
            initialValue: latestCapture.flatMap(RouteCalibrationCaptureStore.jsonString(for:))
        )
        _hasRecorderSession = State(initialValue: AppConfiguration.recorderSessionToken != nil)
        let savedRouteStart = RouteARPlacementStore.load(routeId: route.id, overlayId: overlay.id)
        let savedWorldMapLoaded = savedRouteStart != nil
            && RouteARWorldMapStore.load(routeId: route.id, overlayId: overlay.id) != nil
        initialSavedWorldMapLoaded = savedWorldMapLoaded
        _routeStartWorldPosition = State(initialValue: savedRouteStart)
        _anchorMessage = State(
            initialValue: savedRouteStart == nil
                ? nil
                : "Saved wall anchor loaded; move slowly to relocalize."
        )
        _trackingStatus = State(
            initialValue: !savedWorldMapLoaded
                ? .initializing
                : .relocalizing
        )
        _hasRelocalizedSavedWorldMap = State(initialValue: !savedWorldMapLoaded)
        _hasSeenRelocalizingState = State(initialValue: false)
    }

    var body: some View {
        ZStack(alignment: .bottom) {
            RouteARSceneView(
                routeId: route.id,
                overlayId: overlay.id,
                routeName: route.name,
                overlay: overlay,
                alignment: alignment,
                routeStartWorldPosition: routeStartWorldPosition,
                isPlacingRouteStart: isPlacingRouteStart,
                isTraceVisible: isTraceVisible,
                onRouteStartPlaced: placeRouteStart,
                onPlacementRejected: { message in anchorMessage = message },
                onTrackingStatusChanged: { status in
                    switch status {
                    case .relocalizing:
                        trackingStatus = status
                        hasSeenRelocalizingState = true
                        hasRelocalizedSavedWorldMap = false
                    case .ready where initialSavedWorldMapLoaded && !hasSeenRelocalizingState:
                        // A fresh AR session can become "normal" without matching
                        // the saved map. Keep the persisted coordinates hidden until
                        // ARKit has reported its relocalization phase.
                        trackingStatus = .relocalizing
                    case .ready:
                        trackingStatus = status
                        hasRelocalizedSavedWorldMap = true
                    default:
                        trackingStatus = status
                    }
                },
                onWorldMapSaved: { didSave in
                    if didSave, let routeStartWorldPosition {
                        RouteARPlacementStore.save(
                            routeStartWorldPosition,
                            routeId: route.id,
                            overlayId: overlay.id
                        )
                        anchorMessage = "Route start and wall reference saved for the next session."
                    } else {
                        RouteARPlacementStore.delete(routeId: route.id, overlayId: overlay.id)
                        anchorMessage = "Route start saved for this session. Scan more of the wall to save its reference."
                    }
                }
            )
                .ignoresSafeArea()

            RouteARControlPanel(
                routeName: route.name,
                overlay: overlay,
                alignmentHint: alignmentHint,
                alignmentStatus: alignmentStatus,
                anchorStatus: anchorStatus,
                trackingStatus: trackingStatus,
                locationStatus: locationStatus,
                alignment: $alignment,
                isExpanded: $isControlPanelExpanded,
                captureStatus: captureStatus,
                uploadMessage: uploadMessage,
                latestCaptureJSON: latestCaptureJSON,
                hasLatestCapture: latestCapture != nil,
                hasRecorderSession: hasRecorderSession,
                isUploading: isUploading,
                isPlacingRouteStart: isPlacingRouteStart,
                hasSavedPlacement: routeStartWorldPosition != nil,
                canPlaceRouteStart: isAtWall,
                saveCalibrationCapture: saveCalibrationCapture,
                uploadLatestCapture: uploadLatestCapture,
                presentRecorderLogin: { isRecorderLoginPresented = true },
                signOutRecorder: {
                    ClimbARAPI.logoutRecorder()
                    hasRecorderSession = false
                    uploadMessage = "Recorder signed out. Sign in again before uploading."
                },
                beginRouteStartPlacement: beginRouteStartPlacement,
                clearRouteStartPlacement: clearRouteStartPlacement
            )
            .padding(.horizontal, 12)
            .padding(.bottom, 12)
        }
        .navigationTitle("Find It Outside")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar(.hidden, for: .tabBar)
        .onChange(of: alignment) { _, newValue in
            RouteARAlignmentStore.save(newValue, routeId: route.id, overlayId: overlay.id)
        }
        .task {
            locationService.requestLocation()
        }
        .sheet(isPresented: $isRecorderLoginPresented) {
            RecorderLoginView { email, password in
                try await ClimbARAPI().loginRecorder(email: email, password: password)
                hasRecorderSession = true
                isRecorderLoginPresented = false
            }
        }
    }

    private var captureStatus: String {
        if let captureMessage {
            return captureMessage
        }

        if captureCount == 0 {
            return "No calibration snapshots saved for this route yet."
        }

        return "\(captureCount) calibration snapshot\(captureCount == 1 ? "" : "s") saved locally."
    }

    private var alignmentHint: String {
        switch overlay.anchorStrategy {
        case .manualAlignment:
            "Nudge and scale the yellow trace until it sits over the route."
        case .referenceImage:
            "Point the camera at the reference topo or wall image to align the route."
        case .wallPlaneAndBearing:
            "Face the wall and let ARKit detect the plane before following the trace."
        }
    }

    private var alignmentStatus: String {
        captureCount == 0 ? "Align the yellow trace with the route." : "Latest alignment saved on this device."
    }

    private var anchorStatus: String {
        if let anchorMessage {
            return anchorMessage
        }

        return routeStartWorldPosition == nil
            ? "Tap Set Route Start, then tap the route start on the wall."
            : "Trace anchored to the wall for this session."
    }

    private var proximityState: RouteProximityState {
        RouteProximityService().state(for: route, userLocation: locationService.userLocation)
    }

    private var isAtWall: Bool {
        if case .atWall = proximityState {
            return true
        }

        return false
    }

    private var isTraceVisible: Bool {
        isAtWall && trackingStatus == .ready && hasRelocalizedSavedWorldMap
    }

    private var locationStatus: String {
        switch proximityState {
        case .locationUnavailable:
            "Waiting for precise location"
        case .outOfRange(let distance):
            "Move closer to the route · \(formattedDistance(distance))"
        case .nearby(let distance):
            "Near the route · move closer to the wall · \(formattedDistance(distance))"
        case .atWall(let distance):
            "At the wall · \(formattedDistance(distance))"
        }
    }

    private func beginRouteStartPlacement() {
        guard isAtWall else {
            anchorMessage = locationStatus
            return
        }

        anchorMessage = nil
        isPlacingRouteStart = true
    }

    private func placeRouteStart(at worldPosition: SIMD3<Float>) {
        routeStartWorldPosition = worldPosition
        hasSeenRelocalizingState = true
        hasRelocalizedSavedWorldMap = true
        isPlacingRouteStart = false
        anchorMessage = "Route start anchored for this session. Saving the wall reference..."
    }

    private func clearRouteStartPlacement() {
        routeStartWorldPosition = nil
        hasSeenRelocalizingState = true
        hasRelocalizedSavedWorldMap = true
        isPlacingRouteStart = false
        anchorMessage = "Saved wall reference removed."
        RouteARPlacementStore.delete(routeId: route.id, overlayId: overlay.id)
        RouteARWorldMapStore.delete(routeId: route.id, overlayId: overlay.id)
    }

    private func saveCalibrationCapture() {
        let capture = RouteCalibrationCapture(
            routeId: route.id,
            routeName: route.name,
            overlayId: overlay.id,
            overlayVersion: overlay.version,
            anchorStrategy: overlay.anchorStrategy,
            alignment: alignment
        )
        captureCount = RouteCalibrationCaptureStore.save(capture)
        latestCapture = capture
        latestCaptureJSON = RouteCalibrationCaptureStore.jsonString(for: capture)
        captureMessage = "Saved calibration snapshot \(capture.capturedAt.formatted(date: .abbreviated, time: .shortened))."
        uploadMessage = nil
    }

    private func uploadLatestCapture() async {
        guard let latestCapture else {
            return
        }

        isUploading = true
        defer { isUploading = false }

        do {
            try await ClimbARAPI().post(
                path: "admin/ar-calibration-captures",
                body: latestCapture
            )
            let captureId = latestCapture.id.uuidString.prefix(8)
            uploadMessage = "Uploaded snapshot \(captureId). Refresh Calibration Review."
        } catch let APIError.requestFailed(statusCode) {
            let status = statusCode.map(String.init) ?? "unknown"
            uploadMessage = "Upload failed (HTTP \(status)). Check the API URL and recorder sign-in."
        } catch {
            uploadMessage = "Upload failed. Check that the backend is reachable."
        }
    }

    private func formattedDistance(_ meters: Double) -> String {
        meters >= 1000 ? String(format: "%.1f km", meters / 1000) : "(Int(meters.rounded())) m"
    }
}

private struct RouteARControlPanel: View {
    let routeName: String
    let overlay: RouteAROverlay
    let alignmentHint: String
    let alignmentStatus: String
    let anchorStatus: String
    let trackingStatus: RouteARTrackingStatus
    let locationStatus: String
    @Binding var alignment: RouteARAlignment
    @Binding var isExpanded: Bool
    let captureStatus: String
    let uploadMessage: String?
    let latestCaptureJSON: String?
    let hasLatestCapture: Bool
    let hasRecorderSession: Bool
    let isUploading: Bool
    let isPlacingRouteStart: Bool
    let hasSavedPlacement: Bool
    let canPlaceRouteStart: Bool
    let saveCalibrationCapture: () -> Void
    let uploadLatestCapture: () async -> Void
    let presentRecorderLogin: () -> Void
    let signOutRecorder: () -> Void
    let beginRouteStartPlacement: () -> Void
    let clearRouteStartPlacement: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack(alignment: .top, spacing: 12) {
                VStack(alignment: .leading, spacing: 2) {
                    Text(routeName)
                        .font(.headline)
                        .lineLimit(1)

                    Text("Overlay v\(overlay.version) • \(overlay.confidence.rawValue)")
                        .font(.caption2)
                        .foregroundStyle(.secondary)

                    Label(trackingStatus.message, systemImage: trackingStatus.systemImage)
                        .font(.caption2.weight(.semibold))
                        .foregroundStyle(trackingStatus.color)

                    Label(locationStatus, systemImage: "location.fill")
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                }

                Spacer()

                Button {
                    withAnimation(.snappy) {
                        isExpanded.toggle()
                    }
                } label: {
                    Label(isExpanded ? "Done" : "Tune", systemImage: isExpanded ? "checkmark" : "slider.horizontal.3")
                }
                .font(.caption.weight(.semibold))
                .buttonStyle(.bordered)
                .controlSize(.small)
                .accessibilityLabel(isExpanded ? "Finish tuning" : "Tune route trace")
            }

            if isExpanded {
                ScrollView {
                    VStack(alignment: .leading, spacing: 10) {
                        Label(alignmentStatus, systemImage: "scope")
                            .font(.subheadline.weight(.semibold))
                            .foregroundStyle(ClimbARStyle.tint)

                        placementAction

                        Text(alignmentHint)
                            .font(.caption)
                            .foregroundStyle(.secondary)
                            .lineLimit(2)

                        Label(confidenceMessage, systemImage: confidenceIcon)
                            .font(.caption2.weight(.semibold))
                            .foregroundStyle(confidenceColor)

                        RouteAlignmentControls(alignment: $alignment)

                        calibrationActions

                        statusText
                    }
                }
                .scrollIndicators(.hidden)
                .frame(maxHeight: 280)
            } else {
                VStack(alignment: .leading, spacing: 8) {
                    Label(alignmentStatus, systemImage: "scope")
                        .font(.subheadline.weight(.semibold))
                        .foregroundStyle(ClimbARStyle.tint)

                    Text(anchorStatus)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .lineLimit(2)

                    HStack {
                        Text(alignment.summary)
                            .font(.caption2.monospacedDigit())
                            .foregroundStyle(.secondary)
                            .lineLimit(1)

                        Spacer()

                        Button {
                            saveCalibrationCapture()
                        } label: {
                            Label("Save snapshot", systemImage: "square.and.arrow.down")
                        }
                        .buttonStyle(.borderedProminent)
                        .controlSize(.small)
                    }
                }
            }
        }
        .padding(12)
        .background(.regularMaterial, in: RoundedRectangle(cornerRadius: 18, style: .continuous))
        .shadow(color: .black.opacity(0.18), radius: 18, y: 8)
    }

    private var confidenceMessage: String {
        switch overlay.confidence {
        case .draft: "Draft overlay: confirm the trace visually before climbing."
        case .fieldTested: "Field-tested alignment: check the wall and route start."
        case .reviewed: "Reviewed overlay: use the trace as a visual guide, not a safety guarantee."
        }
    }

    private var confidenceIcon: String {
        overlay.confidence == .reviewed ? "checkmark.seal" : "exclamationmark.triangle"
    }

    private var confidenceColor: Color {
        overlay.confidence == .reviewed ? .green : .orange
    }

    private var calibrationActions: some View {
        VStack(spacing: 8) {
            Button {
                saveCalibrationCapture()
            } label: {
                Label("Save calibration snapshot", systemImage: "square.and.arrow.down")
                    .frame(maxWidth: .infinity)
            }
            .buttonStyle(.borderedProminent)

            HStack(spacing: 8) {
                if let latestCaptureJSON {
                    ShareLink(item: latestCaptureJSON) {
                        Label("Share", systemImage: "square.and.arrow.up")
                            .frame(maxWidth: .infinity)
                    }
                    .buttonStyle(.bordered)
                }

                if hasLatestCapture {
                    if hasRecorderSession {
                        Button {
                            Task {
                                await uploadLatestCapture()
                            }
                        } label: {
                            if isUploading {
                                ProgressView()
                                    .frame(maxWidth: .infinity)
                            } else {
                                Label("Upload", systemImage: "icloud.and.arrow.up")
                                    .frame(maxWidth: .infinity)
                            }
                        }
                        .buttonStyle(.bordered)
                        .disabled(isUploading)

                    } else {
                        Button {
                            presentRecorderLogin()
                        } label: {
                            Label("Recorder sign-in", systemImage: "person.badge.key")
                                .frame(maxWidth: .infinity)
                        }
                        .buttonStyle(.bordered)
                    }
                }
            }

            if hasRecorderSession {
                Button {
                    signOutRecorder()
                } label: {
                    Label("Sign out recorder", systemImage: "rectangle.portrait.and.arrow.right")
                        .frame(maxWidth: .infinity)
                }
                .buttonStyle(.bordered)
            }
        }
        .font(.caption.weight(.semibold))
        .controlSize(.small)
        .lineLimit(1)
    }

    private var statusText: some View {
        VStack(alignment: .leading, spacing: 3) {
            Text(captureStatus)
            if !hasRecorderSession {
                Text("Sign in as a field recorder to upload snapshots for review.")
            }
            if let uploadMessage {
                Text(uploadMessage)
            }

            Text("Confirm the route start, holds, and protection against the wall before climbing.")
        }
        .font(.caption2)
        .foregroundStyle(.secondary)
    }

    private var placementAction: some View {
        VStack(alignment: .leading, spacing: 4) {
            Button {
                beginRouteStartPlacement()
            } label: {
                Label(
                    isPlacingRouteStart ? "Tap the route start on the wall" : "Set Route Start",
                    systemImage: isPlacingRouteStart ? "hand.tap" : "scope"
                )
                .frame(maxWidth: .infinity)
            }
            .buttonStyle(.borderedProminent)
            .disabled(isPlacingRouteStart || !canPlaceRouteStart)

            Text(anchorStatus)
                .font(.caption2)
                .foregroundStyle(.secondary)
                .lineLimit(2)

            if hasSavedPlacement {
                Button("Forget saved wall reference", role: .destructive) {
                    clearRouteStartPlacement()
                }
                .font(.caption)
            }
        }
    }
}

private struct RecorderLoginView: View {
    let onLogin: (String, String) async throws -> Void
    @Environment(\.dismiss) private var dismiss
    @State private var email = ""
    @State private var password = ""
    @State private var errorMessage: String?
    @State private var isSubmitting = false

    var body: some View {
        NavigationStack {
            Form {
                Section("Field recorder sign in") {
                    TextField("Email", text: $email)
                        .textContentType(.emailAddress)
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()
                    SecureField("Password", text: $password)
                        .textContentType(.password)
                }

                if let errorMessage {
                    Text(errorMessage)
                        .foregroundStyle(.red)
                }

                Button(isSubmitting ? "Signing in..." : "Sign in") {
                    Task {
                        isSubmitting = true
                        defer { isSubmitting = false }
                        do {
                            try await onLogin(email, password)
                            dismiss()
                        } catch {
                            errorMessage = "Unable to sign in. Check the recorder credentials and network."
                        }
                    }
                }
                .disabled(isSubmitting || email.isEmpty || password.isEmpty)
            }
            .navigationTitle("Recorder Access")
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") { dismiss() }
                }
            }
        }
    }
}

private struct RouteAlignmentControls: View {
    @Binding var alignment: RouteARAlignment

    var body: some View {
        VStack(spacing: 10) {
            HStack {
                Text(alignment.summary)
                    .font(.caption2.monospacedDigit())
                    .foregroundStyle(.secondary)
                    .lineLimit(1)

                Spacer()

                Button("Reset") {
                    alignment = .zero
                }
                .font(.caption)
            }

            HStack(alignment: .center, spacing: 14) {
                nudgePad

                VStack(alignment: .leading, spacing: 8) {
                    sliderRow(
                        title: "Depth",
                        value: $alignment.depthOffsetMeters,
                        range: -3...3,
                        step: 0.1
                    )

                    sliderRow(
                        title: "Scale",
                        value: $alignment.scale,
                        range: 0.5...1.75,
                        step: 0.05
                    )
                }
            }
        }
        .padding(.top, 4)
        .accessibilityElement(children: .contain)
        .accessibilityLabel("Route trace alignment controls")
    }

    private var nudgePad: some View {
        VStack(spacing: 6) {
            nudgeButton(systemName: "arrow.up") {
                alignment.verticalOffsetMeters += 0.1
            }

            HStack(spacing: 6) {
                nudgeButton(systemName: "arrow.left") {
                    alignment.horizontalOffsetMeters -= 0.1
                }

                nudgeButton(systemName: "arrow.right") {
                    alignment.horizontalOffsetMeters += 0.1
                }
            }

            nudgeButton(systemName: "arrow.down") {
                alignment.verticalOffsetMeters -= 0.1
            }
        }
    }

    private func nudgeButton(systemName: String, action: @escaping () -> Void) -> some View {
        Button(action: action) {
            Image(systemName: systemName)
                .frame(width: 30, height: 26)
        }
        .buttonStyle(.bordered)
        .controlSize(.small)
        .accessibilityLabel(nudgeLabel(for: systemName))
    }

    private func nudgeLabel(for systemName: String) -> String {
        switch systemName {
        case "arrow.up": "Move trace up"
        case "arrow.down": "Move trace down"
        case "arrow.left": "Move trace left"
        case "arrow.right": "Move trace right"
        default: "Move trace"
        }
    }

    private func sliderRow(
        title: String,
        value: Binding<Float>,
        range: ClosedRange<Float>,
        step: Float
    ) -> some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(title)
                .font(.caption2)
                .foregroundStyle(.secondary)
            Slider(
                value: value,
                in: range,
                step: step
            )
        }
    }
}

private struct RouteARSceneView: UIViewRepresentable {
    let routeId: UUID
    let overlayId: UUID
    let routeName: String
    let overlay: RouteAROverlay
    let alignment: RouteARAlignment
    let routeStartWorldPosition: SIMD3<Float>?
    let isPlacingRouteStart: Bool
    let isTraceVisible: Bool
    let onRouteStartPlaced: (SIMD3<Float>) -> Void
    let onPlacementRejected: (String) -> Void
    let onTrackingStatusChanged: (RouteARTrackingStatus) -> Void
    let onWorldMapSaved: (Bool) -> Void

    func makeCoordinator() -> Coordinator {
        Coordinator(
            routeId: routeId,
            overlayId: overlayId,
            onRouteStartPlaced: onRouteStartPlaced,
            onPlacementRejected: onPlacementRejected,
            onTrackingStatusChanged: onTrackingStatusChanged,
            onWorldMapSaved: onWorldMapSaved
        )
    }

    func makeUIView(context: Context) -> ARView {
        let arView = ARView(frame: .zero)
        arView.environment.sceneUnderstanding.options.insert(.occlusion)
        arView.automaticallyConfigureSession = false
        arView.session.delegate = context.coordinator

        if ARWorldTrackingConfiguration.isSupported {
            let configuration = ARWorldTrackingConfiguration()
            configuration.planeDetection = [.vertical]
            configuration.environmentTexturing = .automatic
            configuration.initialWorldMap = RouteARWorldMapStore.load(
                routeId: routeId,
                overlayId: overlayId
            )
            arView.session.run(configuration)
        } else {
            onTrackingStatusChanged(.unavailable)
        }

        let tapGesture = UITapGestureRecognizer(
            target: context.coordinator,
            action: #selector(Coordinator.handleTap(_:))
        )
        arView.addGestureRecognizer(tapGesture)

        renderTrace(in: arView)
        return arView
    }

    func updateUIView(_ arView: ARView, context: Context) {
        context.coordinator.isPlacingRouteStart = isPlacingRouteStart
        context.coordinator.onRouteStartPlaced = onRouteStartPlaced
        context.coordinator.onPlacementRejected = onPlacementRejected
        context.coordinator.onTrackingStatusChanged = onTrackingStatusChanged
        context.coordinator.onWorldMapSaved = onWorldMapSaved
        arView.scene.anchors.removeAll()
        renderTrace(in: arView)
    }

    private func renderTrace(in arView: ARView) {
        guard isTraceVisible else {
            return
        }

        let projectedPoints = RouteTraceProjector().project(overlay: overlay, alignment: alignment)
        let points = anchoredPoints(projectedPoints)
        guard points.count >= 2 else {
            return
        }

        let anchor = AnchorEntity(world: .zero)
        let material = UnlitMaterial(color: .systemYellow)

        for segment in zip(points, points.dropFirst()) {
            let entity = RouteTraceSegmentEntity.make(
                start: segment.0,
                end: segment.1,
                material: material
            )
            anchor.addChild(entity)
        }

        for point in points {
            let holdMarker = ModelEntity(
                mesh: .generateSphere(radius: 0.05),
                materials: [material]
            )
            holdMarker.position = point
            anchor.addChild(holdMarker)
        }

        addRouteNameLabel(routeName, near: points[0], to: anchor)

        arView.scene.addAnchor(anchor)
    }

    private func anchoredPoints(_ points: [SIMD3<Float>]) -> [SIMD3<Float>] {
        guard let routeStartWorldPosition, let firstPoint = points.first else {
            return points
        }

        let translation = routeStartWorldPosition - firstPoint
        return points.map { $0 + translation }
    }

    private func addRouteNameLabel(_ name: String, near point: SIMD3<Float>, to anchor: AnchorEntity) {
        let labelPosition = point + SIMD3<Float>(0, 0.18, 0)
        let connector = RouteTraceSegmentEntity.make(
            start: point,
            end: labelPosition,
            material: UnlitMaterial(color: .systemYellow)
        )
        anchor.addChild(connector)

        let textMesh = MeshResource.generateText(
            name,
            extrusionDepth: 0.004,
            font: .systemFont(ofSize: 0.12, weight: .bold),
            containerFrame: .zero,
            alignment: .center,
            lineBreakMode: .byTruncatingTail
        )
        let label = ModelEntity(
            mesh: textMesh,
            materials: [UnlitMaterial(color: .white)]
        )
        label.position = labelPosition
        if #available(iOS 18.0, *) {
            label.components.set(BillboardComponent())
        }
        anchor.addChild(label)
    }

    final class Coordinator: NSObject, ARSessionDelegate {
        let routeId: UUID
        let overlayId: UUID
        var isPlacingRouteStart = false
        var onRouteStartPlaced: (SIMD3<Float>) -> Void
        var onPlacementRejected: (String) -> Void
        var onTrackingStatusChanged: (RouteARTrackingStatus) -> Void
        var onWorldMapSaved: (Bool) -> Void

        init(
            routeId: UUID,
            overlayId: UUID,
            onRouteStartPlaced: @escaping (SIMD3<Float>) -> Void,
            onPlacementRejected: @escaping (String) -> Void,
            onTrackingStatusChanged: @escaping (RouteARTrackingStatus) -> Void,
            onWorldMapSaved: @escaping (Bool) -> Void
        ) {
            self.routeId = routeId
            self.overlayId = overlayId
            self.onRouteStartPlaced = onRouteStartPlaced
            self.onPlacementRejected = onPlacementRejected
            self.onTrackingStatusChanged = onTrackingStatusChanged
            self.onWorldMapSaved = onWorldMapSaved
        }

        func session(_ session: ARSession, cameraDidChangeTrackingState camera: ARCamera) {
            let status: RouteARTrackingStatus
            switch camera.trackingState {
            case .normal:
                status = .ready
            case .notAvailable:
                status = .unavailable
            case .limited(let reason):
                switch reason {
                case .initializing:
                    status = .initializing
                case .relocalizing:
                    status = .relocalizing
                case .excessiveMotion:
                    status = .limited("hold the phone steady")
                case .insufficientFeatures:
                    status = .limited("more visual detail needed")
                @unknown default:
                    status = .limited("tracking is limited")
                }
            @unknown default:
                status = .limited("tracking is limited")
            }

            Task { @MainActor in
                onTrackingStatusChanged(status)
            }
        }

        func session(_ session: ARSession, didFailWithError error: Error) {
            Task { @MainActor in
                onTrackingStatusChanged(.unavailable)
            }
        }

        @objc func handleTap(_ gesture: UITapGestureRecognizer) {
            guard isPlacingRouteStart,
                  let arView = gesture.view as? ARView else {
                return
            }

            let location = gesture.location(in: arView)
            guard let frame = arView.session.currentFrame,
                  frame.camera.trackingState == .normal else {
                onPlacementRejected("Wait for wall tracking to become ready, then try again.")
                return
            }

            guard frame.worldMappingStatus == .mapped || frame.worldMappingStatus == .extending else {
                onPlacementRejected("Scan the wall slowly for a moment before setting the route start.")
                return
            }

            guard let result = arView.raycast(
                from: location,
                allowing: .estimatedPlane,
                alignment: .vertical
            ).first else {
                onPlacementRejected("Aim at a visible vertical section of the wall and try again.")
                return
            }

            let transform = result.worldTransform
            let worldPosition = SIMD3<Float>(
                transform.columns.3.x,
                transform.columns.3.y,
                transform.columns.3.z
            )
            arView.session.getCurrentWorldMap { worldMap, _ in
                guard let worldMap else {
                    DispatchQueue.main.async {
                        self.onWorldMapSaved(false)
                    }
                    return
                }

                let didSave = RouteARWorldMapStore.save(
                    worldMap,
                    routeId: self.routeId,
                    overlayId: self.overlayId
                )
                DispatchQueue.main.async {
                    self.onWorldMapSaved(didSave)
                }
            }
            onRouteStartPlaced(worldPosition)
        }
    }
}

private struct RouteTraceProjector {
    func project(overlay: RouteAROverlay, alignment: RouteARAlignment = .zero) -> [SIMD3<Float>] {
        apply(alignment: alignment, to: basePoints(for: overlay))
    }

    private func basePoints(for overlay: RouteAROverlay) -> [SIMD3<Float>] {
        switch overlay.routeTrace.coordinateSpace {
        case .normalizedWallImage:
            guard let wallPlane = overlay.wallPlane,
                  let center = vector(from: wallPlane.center),
                  wallPlane.widthMeters > 0,
                  wallPlane.heightMeters > 0 else {
                return fallbackNormalizedTrace(overlay.routeTrace.points)
            }

            let normal = simd_normalize(vector(from: wallPlane.normal) ?? SIMD3<Float>(0, 0, 1))
            let worldUp = abs(simd_dot(normal, SIMD3<Float>(0, 1, 0))) > 0.95 ? SIMD3<Float>(0, 0, 1) : SIMD3<Float>(0, 1, 0)
            let right = simd_normalize(simd_cross(worldUp, normal))
            let up = simd_normalize(simd_cross(normal, right))

            return overlay.routeTrace.points.map { point in
                let x = (point.x - 0.5) * wallPlane.widthMeters
                let y = (0.5 - point.y) * wallPlane.heightMeters
                let z = point.z ?? 0
                return center + (right * x) + (up * y) + (normal * z)
            }

        case .localWallMeters:
            return overlay.routeTrace.points.map { point in
                SIMD3<Float>(point.x, point.y, point.z ?? -2)
            }
        }
    }

    private func apply(
        alignment: RouteARAlignment,
        to points: [SIMD3<Float>]
    ) -> [SIMD3<Float>] {
        guard !points.isEmpty else {
            return []
        }

        let center = points.reduce(SIMD3<Float>.zero, +) / Float(points.count)
        let offset = SIMD3<Float>(
            alignment.horizontalOffsetMeters,
            alignment.verticalOffsetMeters,
            alignment.depthOffsetMeters
        )

        return points.map { point in
            center + ((point - center) * alignment.scale) + offset
        }
    }

    private func fallbackNormalizedTrace(_ points: [TracePoint]) -> [SIMD3<Float>] {
        points.map { point in
            SIMD3<Float>(
                (point.x - 0.5) * 2,
                (0.5 - point.y) * 3,
                -2
            )
        }
    }

    private func vector(from values: [Float]) -> SIMD3<Float>? {
        guard values.count == 3 else {
            return nil
        }

        return SIMD3<Float>(values[0], values[1], values[2])
    }
}

private enum RouteARAlignmentStore {
    static func load(routeId: UUID, overlayId: UUID) -> RouteARAlignment? {
        guard let data = UserDefaults.standard.data(forKey: key(routeId: routeId, overlayId: overlayId)),
              let alignment = try? JSONDecoder().decode(RouteARAlignment.self, from: data) else {
            return nil
        }

        return alignment
    }

    static func save(_ alignment: RouteARAlignment, routeId: UUID, overlayId: UUID) {
        guard let data = try? JSONEncoder().encode(alignment) else {
            return
        }

        UserDefaults.standard.set(data, forKey: key(routeId: routeId, overlayId: overlayId))
    }

    private static func key(routeId: UUID, overlayId: UUID) -> String {
        "route-ar-alignment-\(routeId.uuidString)-\(overlayId.uuidString)"
    }
}

private struct StoredRouteStartPosition: Codable {
    let x: Float
    let y: Float
    let z: Float

    init(_ position: SIMD3<Float>) {
        x = position.x
        y = position.y
        z = position.z
    }

    var simdPosition: SIMD3<Float> {
        SIMD3<Float>(x, y, z)
    }
}

private enum RouteARPlacementStore {
    static func load(routeId: UUID, overlayId: UUID) -> SIMD3<Float>? {
        guard let data = UserDefaults.standard.data(forKey: key(routeId: routeId, overlayId: overlayId)),
              let position = try? JSONDecoder().decode(StoredRouteStartPosition.self, from: data) else {
            return nil
        }

        return position.simdPosition
    }

    static func save(_ position: SIMD3<Float>, routeId: UUID, overlayId: UUID) {
        guard let data = try? JSONEncoder().encode(StoredRouteStartPosition(position)) else {
            return
        }

        UserDefaults.standard.set(data, forKey: key(routeId: routeId, overlayId: overlayId))
    }

    static func delete(routeId: UUID, overlayId: UUID) {
        UserDefaults.standard.removeObject(forKey: key(routeId: routeId, overlayId: overlayId))
    }

    private static func key(routeId: UUID, overlayId: UUID) -> String {
        "route-ar-start-position-\(routeId.uuidString)-\(overlayId.uuidString)"
    }
}

private enum RouteARWorldMapStore {
    static func load(routeId: UUID, overlayId: UUID) -> ARWorldMap? {
        let url = worldMapURL(routeId: routeId, overlayId: overlayId)
        guard let data = try? Data(contentsOf: url) else {
            return nil
        }

        return try? NSKeyedUnarchiver.unarchivedObject(ofClass: ARWorldMap.self, from: data)
    }

    static func save(_ worldMap: ARWorldMap, routeId: UUID, overlayId: UUID) -> Bool {
        guard let data = try? NSKeyedArchiver.archivedData(
            withRootObject: worldMap,
            requiringSecureCoding: true
        ) else {
            return false
        }

        let directory = worldMapsDirectory
        try? FileManager.default.createDirectory(
            at: directory,
            withIntermediateDirectories: true
        )
        do {
            try data.write(to: worldMapURL(routeId: routeId, overlayId: overlayId), options: .atomic)
            return true
        } catch {
            return false
        }
    }

    static func delete(routeId: UUID, overlayId: UUID) {
        try? FileManager.default.removeItem(at: worldMapURL(routeId: routeId, overlayId: overlayId))
    }

    private static var worldMapsDirectory: URL {
        FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
            .appending(path: "ARWorldMaps", directoryHint: .isDirectory)
    }

    private static func worldMapURL(routeId: UUID, overlayId: UUID) -> URL {
        worldMapsDirectory.appending(path: "\(routeId.uuidString)-\(overlayId.uuidString).worldmap")
    }
}

private struct RouteCalibrationCapture: Codable, Identifiable, Equatable {
    let id: UUID
    let routeId: UUID
    let routeName: String
    let overlayId: UUID
    let overlayVersion: UInt32
    let anchorStrategy: ARAnchorStrategy
    let alignment: RouteARAlignment
    let capturedAt: Date

    init(
        id: UUID = UUID(),
        routeId: UUID,
        routeName: String,
        overlayId: UUID,
        overlayVersion: UInt32,
        anchorStrategy: ARAnchorStrategy,
        alignment: RouteARAlignment,
        capturedAt: Date = Date()
    ) {
        self.id = id
        self.routeId = routeId
        self.routeName = routeName
        self.overlayId = overlayId
        self.overlayVersion = overlayVersion
        self.anchorStrategy = anchorStrategy
        self.alignment = alignment
        self.capturedAt = capturedAt
    }
}

private enum RouteCalibrationCaptureStore {
    static func save(_ capture: RouteCalibrationCapture) -> Int {
        var captures = load(routeId: capture.routeId, overlayId: capture.overlayId)
        captures.append(capture)

        guard let data = try? JSONEncoder().encode(captures) else {
            return captures.count - 1
        }

        UserDefaults.standard.set(data, forKey: key(routeId: capture.routeId, overlayId: capture.overlayId))
        return captures.count
    }

    static func count(routeId: UUID, overlayId: UUID) -> Int {
        load(routeId: routeId, overlayId: overlayId).count
    }

    static func latest(routeId: UUID, overlayId: UUID) -> RouteCalibrationCapture? {
        load(routeId: routeId, overlayId: overlayId).last
    }

    static func latestJSON(routeId: UUID, overlayId: UUID) -> String? {
        load(routeId: routeId, overlayId: overlayId).last.flatMap(jsonString(for:))
    }

    static func jsonString(for capture: RouteCalibrationCapture) -> String? {
        let encoder = JSONEncoder()
        encoder.dateEncodingStrategy = .iso8601
        encoder.keyEncodingStrategy = .convertToSnakeCase
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys]

        guard let data = try? encoder.encode(capture) else {
            return nil
        }

        return String(data: data, encoding: .utf8)
    }

    private static func load(routeId: UUID, overlayId: UUID) -> [RouteCalibrationCapture] {
        guard let data = UserDefaults.standard.data(forKey: key(routeId: routeId, overlayId: overlayId)),
              let captures = try? JSONDecoder().decode([RouteCalibrationCapture].self, from: data) else {
            return []
        }

        return captures
    }

    private static func key(routeId: UUID, overlayId: UUID) -> String {
        "route-ar-calibration-captures-\(routeId.uuidString)-\(overlayId.uuidString)"
    }
}

private enum RouteTraceSegmentEntity {
    static func make(
        start: SIMD3<Float>,
        end: SIMD3<Float>,
        material: RealityKit.Material
    ) -> ModelEntity {
        let delta = end - start
        let length = simd_length(delta)
        let mesh = MeshResource.generateBox(size: SIMD3<Float>(0.05, length, 0.05))
        let entity = ModelEntity(mesh: mesh, materials: [material])
        entity.position = (start + end) / 2

        if length > 0 {
            entity.orientation = simd_quatf(from: SIMD3<Float>(0, 1, 0), to: simd_normalize(delta))
        }

        return entity
    }
}
