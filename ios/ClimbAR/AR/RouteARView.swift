import SwiftUI

struct RouteARView: View {
    let route: Route
    let overlay: RouteAROverlay

    var body: some View {
        VStack(spacing: 16) {
            Image(systemName: "arkit")
                .font(.system(size: 56))
                .foregroundStyle(.blue)

            Text("AR route overlay")
                .font(.title2)
                .fontWeight(.semibold)

            Text("This screen will host the ARKit and RealityKit route tracing experience for \(route.name).")
                .multilineTextAlignment(.center)
                .foregroundStyle(.secondary)

            Text("Overlay v\(overlay.version) • \(overlay.confidence.rawValue)")
                .font(.caption)
                .foregroundStyle(.secondary)
        }
        .padding()
        .navigationTitle("Find It Outside")
    }
}

