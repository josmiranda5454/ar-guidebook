import SwiftUI

struct AreaDetailView: View {
    let area: Area

    var body: some View {
        List {
            Section("About") {
                Text(area.description)
                if let accessNotes = area.accessNotes {
                    Text(accessNotes)
                }
            }

            Section("Walls") {
                ForEach(area.walls) { wall in
                    NavigationLink(wall.name) {
                        WallDetailView(wall: wall)
                    }
                }
            }
        }
        .navigationTitle(area.name)
    }
}

