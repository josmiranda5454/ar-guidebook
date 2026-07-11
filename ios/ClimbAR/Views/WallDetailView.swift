import SwiftUI

struct WallDetailView: View {
    let wall: Wall

    var body: some View {
        List {
            Section("Approach") {
                Text(wall.description)
                if let approachNotes = wall.approachNotes {
                    Text(approachNotes)
                        .foregroundStyle(.secondary)
                }
            }

            Section("Routes · \(wall.routes.count)") {
                ForEach(wall.routes) { route in
                    NavigationLink {
                        RouteDetailView(route: route)
                    } label: {
                        RouteRow(route: route)
                    }
                }
            }
        }
        .listStyle(.insetGrouped)
        .tint(ClimbARStyle.tint)
        .navigationTitle(wall.name)
    }
}
