import Foundation

actor OfflinePackStore {
    private let fileManager: FileManager

    init(fileManager: FileManager = .default) {
        self.fileManager = fileManager
    }

    func save(_ pack: OfflinePack) throws {
        let data = try encoder.encode(pack)
        try fileManager.createDirectory(at: packsDirectory, withIntermediateDirectories: true)
        try data.write(to: packURL(areaId: pack.areaId), options: [.atomic])
    }

    func load(areaId: UUID) throws -> OfflinePack? {
        let url = packURL(areaId: areaId)
        guard fileManager.fileExists(atPath: url.path) else {
            return nil
        }

        let data = try Data(contentsOf: url)
        return try decoder.decode(OfflinePack.self, from: data)
    }

    private var packsDirectory: URL {
        fileManager.urls(for: .documentDirectory, in: .userDomainMask)[0]
            .appending(path: "OfflinePacks", directoryHint: .isDirectory)
    }

    private func packURL(areaId: UUID) -> URL {
        packsDirectory.appending(path: "\(areaId.uuidString).json")
    }

    private var encoder: JSONEncoder {
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        encoder.dateEncodingStrategy = .iso8601
        return encoder
    }

    private var decoder: JSONDecoder {
        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        decoder.dateDecodingStrategy = .iso8601
        return decoder
    }
}

