import ARKit
import RealityKit
import SwiftUI

struct RouteARView: View {
    let route: Route
    let overlay: RouteAROverlay

    @State private var alignment: RouteARAlignment

    init(route: Route, overlay: RouteAROverlay) {
        self.route = route
        self.overlay = overlay
        _alignment = State(
            initialValue: RouteARAlignmentStore.load(routeId: route.id, overlayId: overlay.id)
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
