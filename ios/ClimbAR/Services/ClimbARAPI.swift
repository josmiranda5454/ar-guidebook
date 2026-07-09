import Foundation

struct ClimbARAPI {
    var baseURL = URL(string: "http://127.0.0.1:8080/api/v1")!

    func areas() async throws -> [Area] {
        try await get(path: "areas")
    }

    func area(id: UUID) async throws -> Area {
        try await get(path: "areas/\(id.uuidString)")
    }

    func offlinePack(areaId: UUID) async throws -> OfflinePack {
        try await get(path: "offline-packs/areas/\(areaId.uuidString)")
    }

    private func get<T: Decodable>(path: String) async throws -> T {
        let url = baseURL.appending(path: path)
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
    case requestFailed
}

