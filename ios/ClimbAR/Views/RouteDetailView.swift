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
            Section {
                VStack(alignment: .leading, spacing: 10) {
                    HStack {
                        Image(systemName: "figure.climbing")
                            .font(.title2.weight(.semibold))
                            .foregroundStyle(ClimbARStyle.tint)
                            .frame(width: 44, height: 44)
                            .background(ClimbARStyle.tint.opacity(0.12), in: Circle())

                        VStack(alignment: .leading, spacing: 3) {
                            Text(route.grade)
                                .font(.title3.weight(.bold))
                            Text(route.routeTypes.map(\.rawValue).joined(separator: " · "))
                                .font(.subheadline)
                                .foregroundStyle(.secondary)
                        }
                        Spacer()
                    }

                    if let stars = route.starsAverage {
                        Label(String(format: "%.1f community rating", stars), systemImage: "star.fill")
                            .font(.subheadline.weight(.medium))
                            .foregroundStyle(.orange)
                    }
                }
                .padding(.vertical, 4)
            }

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
                        NavigationLink {
                            RouteARView(route: route, overlay: route.arOverlays[0])
                        } label: {
                            Label("Open AR route overlay", systemImage: "arkit")
                                .font(.body.weight(.semibold))
                        }
                        .buttonStyle(.borderedProminent)
                    } else {
                        Label("AR unlocks when you reach the wall", systemImage: "location.circle")
                            .font(.body.weight(.semibold))
                        Text(proximityMessage)
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                    }

                    Button {
                        locationService.requestLocation()
                    } label: {
                        Label("Refresh location", systemImage: "location.fill")
                    }
                    .font(.subheadline.weight(.medium))

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
        .listStyle(.insetGrouped)
        .tint(ClimbARStyle.tint)
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
