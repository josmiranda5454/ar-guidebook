import Foundation

struct RouteAROverlay: Codable, Identifiable, Hashable {
    let id: UUID
    let routeId: UUID
    let version: UInt32
    let anchorStrategy: ARAnchorStrategy
    let gpsHint: GeoPoint
    let compassBearingDegrees: Float?
    let wallPlane: WallPlaneEstimate?
    let routeTrace: RouteTrace
    let confidence: OverlayConfidence
    let reviewedAt: Date?
}

enum ARAnchorStrategy: String, Codable, Hashable {
    case manualAlignment = "manual_alignment"
    case referenceImage = "reference_image"
    case wallPlaneAndBearing = "wall_plane_and_bearing"
}

struct WallPlaneEstimate: Codable, Hashable {
    let normal: [Float]
    let center: [Float]
    let widthMeters: Float
    let heightMeters: Float
}

struct RouteTrace: Codable, Hashable {
    let coordinateSpace: TraceCoordinateSpace
    let points: [TracePoint]
}

enum TraceCoordinateSpace: String, Codable, Hashable {
    case normalizedWallImage = "normalized_wall_image"
    case localWallMeters = "local_wall_meters"
}

struct TracePoint: Codable, Hashable {
    let x: Float
    let y: Float
    let z: Float?
}

enum OverlayConfidence: String, Codable, Hashable {
    case draft
    case fieldTested = "field_tested"
    case reviewed
}

