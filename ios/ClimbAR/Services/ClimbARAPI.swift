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

    func offlinePack(areaId: UUID) async throws -> OfflinePack {
        try await get(path: "offline-packs/areas/\(areaId.uuidString)")
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
