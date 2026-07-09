import SwiftUI

struct WallDetailView: View {
    let wall: Wall

    var body: some View {
        List {
            Section("Approach") {
                Text(wall.description)
                if let approachNotes = wall.approachNotes {
                    Text(approachNotes)
                }
            }

            Section("Routes") {
                ForEach(wall.routes) { route in
                    NavigationLink {
                        RouteDetailView(route: route)
                    } label: {
                        VStack(alignment: .leading) {
                            Text(route.name)
                            Text("\(route.grade) • \(route.routeTypes.map(\.rawValue).joined(separator: ", "))")
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }
                    }
                }
            }
        }
        .navigationTitle(wall.name)
    }
}

