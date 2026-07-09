import SwiftUI
import CoreLocation

struct RouteDetailView: View {
    let route: Route
    @StateObject private var locationService = LocationService()

    private var proximityState: RouteProximityState {
        RouteProximityService().state(for: route, userLocation: locationService.userLocation)
    }

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
                Section("Find It Outside") {
                    if proximityState.canFindOutside {
                        NavigationLink("Open AR route overlay") {
                            RouteARView(route: route, overlay: route.arOverlays[0])
                        }
                    } else {
                        Button("Open AR route overlay") {}
                            .disabled(true)
                    }

                    Text(proximityMessage)
                        .foregroundStyle(.secondary)

                    Button("Refresh Location") {
                        locationService.requestLocation()
                    }

                    #if DEBUG
                    Button("Simulate At Route") {
                        locationService.simulateLocation(
                            CLLocation(
                                latitude: route.location.latitude,
                                longitude: route.location.longitude
                            )
                        )
                    }
                    #endif
                }
            }
        }
        .navigationTitle(route.name)
        .task {
            locationService.requestLocation()
        }
    }

    private var proximityMessage: String {
        switch proximityState {
        case .locationUnavailable:
            "Location is unavailable. Allow location access to use outdoor AR guidance."
        case .outOfRange(let distance):
            "Available when you are at the wall. Current distance: \(formattedDistance(distance))."
        case .nearby(let distance):
            "You are nearby, but move closer to the wall to enable AR. Current distance: \(formattedDistance(distance))."
        case .atWall(let distance):
            "AR route overlay is available. Current distance: \(formattedDistance(distance))."
        }
    }

    private func formattedDistance(_ meters: Double) -> String {
        if meters >= 1000 {
            return String(format: "%.1f km", meters / 1000)
        }

        return "\(Int(meters.rounded())) m"
    }
}
