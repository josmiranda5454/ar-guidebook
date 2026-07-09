import ARKit
import RealityKit
import SwiftUI

struct RouteARView: View {
    let route: Route
    let overlay: RouteAROverlay

    @State private var alignment: RouteARAlignment
    @State private var captureCount: Int
    @State private var captureMessage: String?
    @State private var latestCaptureJSON: String?

    init(route: Route, overlay: RouteAROverlay) {
        self.route = route
        self.overlay = overlay
        let savedAlignment = RouteARAlignmentStore.load(routeId: route.id, overlayId: overlay.id)
        _alignment = State(
            initialValue: savedAlignment
        )
        _captureCount = State(
            initialValue: RouteCalibrationCaptureStore.count(routeId: route.id, overlayId: overlay.id)
        )
        _latestCaptureJSON = State(
            initialValue: RouteCalibrationCaptureStore.latestJSON(routeId: route.id, overlayId: overlay.id)
        )
    }

    var body: some View {
        ZStack(alignment: .bottom) {
            RouteARSceneView(overlay: overlay, alignment: alignment)
                .ignoresSafeArea()

            VStack(alignment: .leading, spacing: 12) {
                Text(route.name)
                    .font(.headline)

                Text("Overlay v\(overlay.version) • \(overlay.confidence.rawValue)")
                    .font(.caption)
                    .foregroundStyle(.secondary)

                Text(alignmentHint)
                    .font(.caption)
                    .foregroundStyle(.secondary)

                RouteAlignmentControls(alignment: $alignment)

                Button {
                    saveCalibrationCapture()
                } label: {
                    Label("Save Calibration Snapshot", systemImage: "square.and.arrow.down")
                        .frame(maxWidth: .infinity)
                }
                .buttonStyle(.borderedProminent)

                Text(captureStatus)
                    .font(.caption2)
                    .foregroundStyle(.secondary)

                if let latestCaptureJSON {
                    ShareLink(item: latestCaptureJSON) {
                        Label("Share Latest Snapshot JSON", systemImage: "square.and.arrow.up")
                            .frame(maxWidth: .infinity)
                    }
                    .buttonStyle(.bordered)
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding()
            .background(.regularMaterial)
        }
        .navigationTitle("Find It Outside")
        .navigationBarTitleDisplayMode(.inline)
        .onChange(of: alignment) { _, newValue in
            RouteARAlignmentStore.save(newValue, routeId: route.id, overlayId: overlay.id)
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
        latestCaptureJSON = RouteCalibrationCaptureStore.jsonString(for: capture)
        captureMessage = "Saved calibration snapshot \(capture.capturedAt.formatted(date: .abbreviated, time: .shortened))."
    }
}

private struct RouteAlignmentControls: View {
    @Binding var alignment: RouteARAlignment

    var body: some View {
        VStack(spacing: 12) {
            HStack {
                Text(alignment.summary)
                    .font(.caption2)
                    .foregroundStyle(.secondary)

                Spacer()

                Button("Reset") {
                    alignment = .zero
                }
                .font(.caption)
            }

            HStack(spacing: 12) {
                Spacer()

                Button {
                    alignment.verticalOffsetMeters += 0.1
                } label: {
                    Image(systemName: "arrow.up")
                }
                .buttonStyle(.bordered)

                Spacer()
            }

            HStack(spacing: 12) {
                Button {
                    alignment.horizontalOffsetMeters -= 0.1
                } label: {
                    Image(systemName: "arrow.left")
                }
                .buttonStyle(.bordered)

                Spacer()

                Button {
                    alignment.horizontalOffsetMeters += 0.1
                } label: {
                    Image(systemName: "arrow.right")
                }
                .buttonStyle(.bordered)
            }

            HStack(spacing: 12) {
                Spacer()

                Button {
                    alignment.verticalOffsetMeters -= 0.1
                } label: {
                    Image(systemName: "arrow.down")
                }
                .buttonStyle(.bordered)

                Spacer()
            }

            VStack(alignment: .leading) {
                Text("Depth")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                Slider(
                    value: $alignment.depthOffsetMeters,
                    in: -3...3,
                    step: 0.1
                )
            }

            VStack(alignment: .leading) {
                Text("Scale")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                Slider(
                    value: $alignment.scale,
                    in: 0.5...1.75,
                    step: 0.05
                )
            }
        }
        .padding(.top, 4)
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

            return overlay.routeTrace.points.map { point in
                let x = (point.x - 0.5) * wallPlane.widthMeters
                let y = (0.5 - point.y) * wallPlane.heightMeters
                let z = point.z ?? 0
                return center + SIMD3<Float>(x, y, z)
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

private struct RouteARAlignment: Codable, Equatable {
    var horizontalOffsetMeters: Float
    var verticalOffsetMeters: Float
    var depthOffsetMeters: Float
    var scale: Float

    static let zero = RouteARAlignment(
        horizontalOffsetMeters: 0,
        verticalOffsetMeters: 0,
        depthOffsetMeters: 0,
        scale: 1
    )

    var summary: String {
        "x \(formatted(horizontalOffsetMeters))m  y \(formatted(verticalOffsetMeters))m  z \(formatted(depthOffsetMeters))m  scale \(String(format: "%.2f", scale))x"
    }

    private func formatted(_ value: Float) -> String {
        String(format: "%+.1f", value)
    }
}

private enum RouteARAlignmentStore {
    static func load(routeId: UUID, overlayId: UUID) -> RouteARAlignment {
        guard let data = UserDefaults.standard.data(forKey: key(routeId: routeId, overlayId: overlayId)),
              let alignment = try? JSONDecoder().decode(RouteARAlignment.self, from: data) else {
            return .zero
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

    static func latestJSON(routeId: UUID, overlayId: UUID) -> String? {
        load(routeId: routeId, overlayId: overlayId).last.flatMap(jsonString(for:))
    }

    static func jsonString(for capture: RouteCalibrationCapture) -> String? {
        let encoder = JSONEncoder()
        encoder.dateEncodingStrategy = .iso8601
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
