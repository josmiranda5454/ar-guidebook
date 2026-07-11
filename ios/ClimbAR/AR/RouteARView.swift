import ARKit
import RealityKit
import SwiftUI

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
    }

    var body: some View {
        ZStack(alignment: .bottom) {
            RouteARSceneView(overlay: overlay, alignment: alignment)
                .ignoresSafeArea()

            RouteARControlPanel(
                routeName: route.name,
                overlay: overlay,
                alignmentHint: alignmentHint,
                alignmentStatus: alignmentStatus,
                alignment: $alignment,
                isExpanded: $isControlPanelExpanded,
                captureStatus: captureStatus,
                uploadMessage: uploadMessage,
                latestCaptureJSON: latestCaptureJSON,
                hasLatestCapture: latestCapture != nil,
                hasRecorderSession: hasRecorderSession,
                isUploading: isUploading,
                saveCalibrationCapture: saveCalibrationCapture,
                uploadLatestCapture: uploadLatestCapture,
                presentRecorderLogin: { isRecorderLoginPresented = true }
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
            uploadMessage = "Uploaded latest calibration snapshot."
        } catch {
            uploadMessage = "Upload failed. Check that the backend is reachable."
        }
    }
}

private struct RouteARControlPanel: View {
    let routeName: String
    let overlay: RouteAROverlay
    let alignmentHint: String
    let alignmentStatus: String
    @Binding var alignment: RouteARAlignment
    @Binding var isExpanded: Bool
    let captureStatus: String
    let uploadMessage: String?
    let latestCaptureJSON: String?
    let hasLatestCapture: Bool
    let hasRecorderSession: Bool
    let isUploading: Bool
    let saveCalibrationCapture: () -> Void
    let uploadLatestCapture: () async -> Void
    let presentRecorderLogin: () -> Void

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
    let overlay: RouteAROverlay
    let alignment: RouteARAlignment

    func makeUIView(context: Context) -> ARView {
        let arView = ARView(frame: .zero)
        arView.environment.sceneUnderstanding.options.insert(.occlusion)
        arView.automaticallyConfigureSession = false

        if ARWorldTrackingConfiguration.isSupported {
            let configuration = ARWorldTrackingConfiguration()
            configuration.planeDetection = [.vertical]
            configuration.environmentTexturing = .automatic
            arView.session.run(configuration)
        }

        renderTrace(in: arView)
        return arView
    }

    func updateUIView(_ arView: ARView, context: Context) {
        arView.scene.anchors.removeAll()
        renderTrace(in: arView)
    }

    private func renderTrace(in arView: ARView) {
        let points = RouteTraceProjector().project(overlay: overlay, alignment: alignment)
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

        arView.scene.addAnchor(anchor)
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
