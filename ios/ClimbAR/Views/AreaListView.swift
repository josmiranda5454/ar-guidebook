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
            List {
                Section {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Find your next line")
                            .font(.title2.weight(.bold))
                        Text("Explore climbing areas, download them for the wall, and use AR when you arrive.")
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                    }
                    .padding(.vertical, 8)
                    .listRowBackground(Color.clear)
                    .listRowInsets(EdgeInsets(top: 8, leading: 20, bottom: 12, trailing: 20))
                }

                Section("Climbing areas") {
                    ForEach(viewModel.areas) { area in
                        NavigationLink {
                            AreaDetailView(area: area, api: viewModel.api, packStore: viewModel.packStore)
                        } label: {
                            HStack(spacing: 12) {
                                Image(systemName: "mountain.2.fill")
                                    .foregroundStyle(ClimbARStyle.tint)
                                    .frame(width: 28)

                                VStack(alignment: .leading, spacing: 3) {
                                    Text(area.name)
                                        .font(.body.weight(.semibold))
                                    Text("\(area.walls.count) wall\(area.walls.count == 1 ? "" : "s")")
                                        .font(.subheadline)
                                        .foregroundStyle(.secondary)
                                }

                                Spacer()
                                if viewModel.downloadedAreaIds.contains(area.id) {
                                    Image(systemName: "checkmark.circle.fill")
                                        .foregroundStyle(.green)
                                        .accessibilityLabel("Downloaded for offline use")
                                }
                            }
                            .padding(.vertical, 5)
                        }
                    }
                }

                if viewModel.areas.isEmpty, viewModel.errorMessage == nil {
                    EmptyStateCard(
                        title: "No areas yet",
                        message: "Pull down to refresh and look for climbing areas.",
                        systemImage: "map"
                    )
                    .listRowBackground(Color.clear)
                }
            }
            .listStyle(.insetGrouped)
            .navigationTitle("Explore")
            .overlay {
                if let errorMessage = viewModel.errorMessage {
                    EmptyStateCard(title: "Offline or unavailable", message: errorMessage, systemImage: "wifi.slash")
                        .padding(.horizontal)
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
