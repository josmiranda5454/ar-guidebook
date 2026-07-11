import SwiftUI

@MainActor
final class AreaListViewModel: ObservableObject {
    @Published var areas: [Area] = []
    @Published var downloadedAreaIds: Set<UUID> = []
    @Published var errorMessage: String?

    let api: ClimbARAPI
    let packStore: OfflinePackStore

    init(api: ClimbARAPI, packStore: OfflinePackStore = OfflinePackStore()) {
        self.api = api
        self.packStore = packStore
    }

    func load() async {
        let cachedPacks = (try? await packStore.loadAll()) ?? []
        let cachedAreas = cachedPacks.flatMap(\.areas)
        downloadedAreaIds = Set(cachedPacks.map(\.areaId))

        do {
            let remoteAreas = try await api.areas()
            areas = mergedAreas(remoteAreas: remoteAreas, cachedAreas: cachedAreas)
            errorMessage = nil
        } catch {
            areas = cachedAreas
            errorMessage = cachedAreas.isEmpty ? "Could not load climbing areas." : "Showing downloaded areas."
        }
    }

    private func mergedAreas(remoteAreas: [Area], cachedAreas: [Area]) -> [Area] {
        let remoteIds = Set(remoteAreas.map(\.id))
        let cachedOnly = cachedAreas.filter { !remoteIds.contains($0.id) }
        return remoteAreas + cachedOnly
    }
}

struct AreaListView: View {
    @StateObject var viewModel: AreaListViewModel

    var body: some View {
        NavigationStack {
            List(viewModel.areas) { area in
                NavigationLink {
                    AreaDetailView(area: area, api: viewModel.api, packStore: viewModel.packStore)
                } label: {
                    HStack {
                        Text(area.name)
                        Spacer()
                        if viewModel.downloadedAreaIds.contains(area.id) {
                            Image(systemName: "arrow.down.circle.fill")
                                .foregroundStyle(.green)
                                .accessibilityLabel("Downloaded")
                        }
                    }
                }
            }
            .navigationTitle("Climbing Areas")
            .overlay {
                if let errorMessage = viewModel.errorMessage {
                    ContentUnavailableView("Offline or Unavailable", systemImage: "wifi.slash", description: Text(errorMessage))
                }
            }
            .refreshable {
                await viewModel.load()
            }
        }
        .task {
            await viewModel.load()
        }
    }
}
