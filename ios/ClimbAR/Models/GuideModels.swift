import Foundation

struct GeoPoint: Codable, Hashable {
    let latitude: Double
    let longitude: Double
    let elevationMeters: Double?
}

struct Area: Codable, Identifiable, Hashable {
    let id: UUID
    let parentAreaId: UUID?
    let name: String
    let slug: String
    let description: String
    let accessNotes: String?
    let location: GeoPoint
    let walls: [Wall]
}

struct Wall: Codable, Identifiable, Hashable {
    let id: UUID
    let areaId: UUID
    let name: String
    let slug: String
    let description: String
    let approachNotes: String?
    let aspect: String?
    let location: GeoPoint
    let routes: [Route]
}

struct Route: Codable, Identifiable, Hashable {
    let id: UUID
    let wallId: UUID
    let name: String
    let slug: String
    let grade: String
    let gradeSystem: GradeSystem
    let routeTypes: [RouteType]
    let lengthFeet: UInt16?
    let pitches: UInt8?
    let starsAverage: Float?
    let ratingVotes: UInt32
    let firstAscent: String?
    let description: String
    let locationNotes: String
    let protectionNotes: String?
    let safetyNotes: String?
    let location: GeoPoint
    let media: [MediaAsset]
    let arOverlays: [RouteAROverlay]
}

enum GradeSystem: String, Codable, Hashable {
    case yosemiteDecimal = "yosemite_decimal"
    case hueco
    case french
}

enum RouteType: String, Codable, Hashable {
    case sport
    case trad
    case boulder
    case mixed
    case topRope = "top_rope"
    case aid
    case ice
    case alpine
}

struct MediaAsset: Codable, Identifiable, Hashable {
    let id: UUID
    let kind: MediaKind
    let title: String
    let url: URL
    let offlinePath: String?
}

enum MediaKind: String, Codable, Hashable {
    case photo
    case topo
    case video
}

struct OfflinePack: Codable, Identifiable, Hashable {
    let id: UUID
    let areaId: UUID
    let version: UInt32
    let generatedAt: Date
    let areas: [Area]
    let assets: [MediaAsset]
}

