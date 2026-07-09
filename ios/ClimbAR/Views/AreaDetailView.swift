import SwiftUI

@MainActor
final class AreaDetailViewModel: ObservableObject {
    @Published var area: Area
    @Published var isDownloaded = false
    @Published var isDownloading = false
    @Published var statusMessage: String?

    private let api: ClimbARAPI
    private let packStore: OfflinePackStore

    init(area: Area, api: ClimbARAPI, packStore: OfflinePackStore) {
        self.area = area
        self.api = api
        self.packStore = packStore
    }

    func loadCachedPack() async {
        isDownloaded = await packStore.isDownloaded(areaId: area.id)
        guard let pack = try? await packStore.load(areaId: area.id),
              let cachedArea = pack.areas.first else {
            return
        }

        area = cachedArea
    }

    func downloadArea() async {
        isDownloading = true
        statusMessage = nil
        defer { isDownloading = false }

        do {
            let pack = try await api.offlinePack(areaId: area.id)
            try await packStore.save(pack)
            if let downloadedArea = pack.areas.first {
                area = downloadedArea
            }
            isDownloaded = true
            statusMessage = "Downloaded for offline use."
        } catch {
            statusMessage = "Could not download this area."
        }
    }
}

struct AreaDetailView: View {
    @StateObject private var viewModel: AreaDetailViewModel

    init(area: Area, api: ClimbARAPI, packStore: OfflinePackStore) {
        _viewModel = StateObject(
            wrappedValue: AreaDetailViewModel(area: area, api: api, packStore: packStore)
        )
    }

    var body: some View {
        List {
            Section {
                Button {
                    Task { await viewModel.downloadArea() }
                } label: {
                    Label(
                        viewModel.isDownloaded ? "Update Offline Area" : "Download Area",
                        systemImage: viewModel.isDownloaded ? "arrow.clockwise.circle" : "arrow.down.circle"
                    )
                }
                .disabled(viewModel.isDownloading)

                if viewModel.isDownloading {
                    ProgressView("Downloading...")
                }

                if let statusMessage = viewModel.statusMessage {
                    Text(statusMessage)
                        .foregroundStyle(.secondary)
                }
            }

            Section("About") {
                Text(viewModel.area.description)
                if let accessNotes = viewModel.area.accessNotes {
                    Text(accessNotes)
                }
            }

            Section("Walls") {
                ForEach(viewModel.area.walls) { wall in
                    NavigationLink(wall.name) {
                        WallDetailView(wall: wall)
                    }
                }
            }
        }
        .navigationTitle(viewModel.area.name)
        .task {
            await viewModel.loadCachedPack()
        }
    }
}
