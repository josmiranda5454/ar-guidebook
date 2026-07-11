import Foundation

struct ClimbARAPI {
    var baseURL = AppConfiguration.apiBaseURL

    func areas() async throws -> [Area] {
        try await get(path: "areas")
    }

    func area(id: UUID) async throws -> Area {
        try await get(path: "areas/\(id.uuidString)")
    }

    func wall(id: UUID) async throws -> Wall {
        try await get(path: "walls/\(id.uuidString)")
    }

    func route(id: UUID) async throws -> Route {
        try await get(path: "routes/\(id.uuidString)")
    }

    func search(query: String) async throws -> [Route] {
        var components = URLComponents(url: baseURL.appending(path: "search"), resolvingAgainstBaseURL: false)!
        components.queryItems = [URLQueryItem(name: "q", value: query)]

        guard let url = components.url else {
            throw APIError.invalidURL
        }

        return try await get(url: url)
    }

    func nearbyRoutes(latitude: Double, longitude: Double, radiusMeters: Double = 2_000) async throws -> [NearbyRoute] {
        var components = URLComponents(url: baseURL.appending(path: "nearby/routes"), resolvingAgainstBaseURL: false)!
        components.queryItems = [
            URLQueryItem(name: "latitude", value: String(latitude)),
            URLQueryItem(name: "longitude", value: String(longitude)),
            URLQueryItem(name: "radius_meters", value: String(radiusMeters)),
        ]
        guard let url = components.url else { throw APIError.invalidURL }
        return try await get(url: url)
    }

    func offlinePack(areaId: UUID) async throws -> OfflinePack {
        try await get(path: "offline-packs/areas/\(areaId.uuidString)")
    }

    func post<T: Encodable>(path: String, body: T) async throws {
        let url = baseURL.appending(path: path)
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")

        let encoder = JSONEncoder()
        encoder.dateEncodingStrategy = .iso8601
        encoder.keyEncodingStrategy = .convertToSnakeCase
        request.httpBody = try encoder.encode(body)

        let (_, response) = try await URLSession.shared.data(for: request)

        guard let httpResponse = response as? HTTPURLResponse,
              (200..<300).contains(httpResponse.statusCode) else {
            throw APIError.requestFailed
        }
    }

    private func get<T: Decodable>(path: String) async throws -> T {
        let url = baseURL.appending(path: path)
        return try await get(url: url)
    }

    private func get<T: Decodable>(url: URL) async throws -> T {
        let (data, response) = try await URLSession.shared.data(from: url)

        guard let httpResponse = response as? HTTPURLResponse,
              (200..<300).contains(httpResponse.statusCode) else {
            throw APIError.requestFailed
        }

        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        decoder.dateDecodingStrategy = .iso8601
        return try decoder.decode(T.self, from: data)
    }
}

enum APIError: Error {
    case invalidURL
    case requestFailed
}

enum AppConfiguration {
    static var apiBaseURL: URL {
        guard let value = Bundle.main.object(forInfoDictionaryKey: "CLIMBAR_API_BASE_URL") as? String,
              !value.isEmpty,
              let url = URL(string: value) else {
            return URL(string: "http://127.0.0.1:8080/api/v1")!
        }

        return url
    }
}
