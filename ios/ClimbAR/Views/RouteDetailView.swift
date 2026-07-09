import SwiftUI

struct RouteDetailView: View {
    let route: Route

    var body: some View {
        List {
            Section("Overview") {
                LabeledContent("Grade", value: route.grade)
                if let lengthFeet = route.lengthFeet {
                    LabeledContent("Length", value: "\(lengthFeet) ft")
                }
                if let pitches = route.pitches {
                    LabeledContent("Pitches", value: "\(pitches)")
                }
            }

            Section("Description") {
                Text(route.description)
            }

            Section("Location") {
                Text(route.locationNotes)
            }

            if let protectionNotes = route.protectionNotes {
                Section("Protection") {
                    Text(protectionNotes)
                }
            }

            if let safetyNotes = route.safetyNotes {
                Section("Safety") {
                    Text(safetyNotes)
                }
            }

            if !route.arOverlays.isEmpty {
                Section {
                    NavigationLink("Find it outside") {
                        RouteARView(route: route, overlay: route.arOverlays[0])
                    }
                }
            }
        }
        .navigationTitle(route.name)
    }
}

