import CoreLocation
import Foundation

struct RouteProximityService {
    func state(for route: Route, userLocation: CLLocation?) -> RouteProximityState {
        guard let userLocation else {
            return .locationUnavailable
        }

        let routeLocation = CLLocation(
            latitude: route.location.latitude,
            longitude: route.location.longitude
        )
        let distance = userLocation.distance(from: routeLocation)

        switch distance {
        case 0..<35:
            return .atWall(distanceMeters: distance)
        case 35..<250:
            return .nearby(distanceMeters: distance)
        default:
            return .outOfRange(distanceMeters: distance)
        }
    }
}

enum RouteProximityState: Equatable {
    case locationUnavailable
    case outOfRange(distanceMeters: Double)
    case nearby(distanceMeters: Double)
    case atWall(distanceMeters: Double)

    var canFindOutside: Bool {
        switch self {
        case .atWall:
            return true
        case .locationUnavailable, .outOfRange, .nearby:
            return false
        }
    }
}

