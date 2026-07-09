import SwiftUI

@MainActor
final class RouteSearchViewModel: ObservableObject {
    @Published var query = ""
    @Published var routes: [Route] = []
    @Published var isSearching = false
    @Published var statusMessage: String?

    private let api: ClimbARAPI
    private let packStore: OfflinePackStore

    init(api: ClimbARAPI, packStore: OfflinePackStore) {
        self.api = api
        self.packStore = packStore
    }

    func search() async {
        let trimmedQuery = query.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmedQuery.isEmpty else {
            routes = []
            statusMessage = nil
            return
        }

        isSearching = true
        defer { isSearching = false }

        do {
            routes = try await api.search(query: trimmedQuery)
            statusMessage = routes.isEmpty ? "No routes found." : nil
        } catch {
            let offlineRoutes = await searchDownloadedRoutes(query: trimmedQuery)
            routes = offlineRoutes
            statusMessage = offlineRoutes.isEmpty
                ? "No downloaded routes matched while offline."
                : "Showing downloaded route matches."
        }
    }

    func loadDownloadedRoutes() async {
        guard query.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else {
            return
        }

        let packs = (try? await packStore.loadAll()) ?? []
        routes = packs.flatMap { pack in
            pack.areas.flatMap { area in
                area.walls.flatMap(\.routes)
            }
        }
        statusMessage = routes.isEmpty ? "Downloaded routes will appear here." : "Showing downloaded routes."
    }

    private func searchDownloadedRoutes(query: String) async -> [Route] {
        let normalizedQuery = query.lowercased()
        let packs = (try? await packStore.loadAll()) ?? []

        return packs
            .flatMap { pack in
                pack.areas.flatMap { area in
                    area.walls.flatMap(\.routes)
                }
            }
            .filter { route in
                route.name.lowercased().contains(normalizedQuery)
                    || route.grade.lowercased().contains(normalizedQuery)
                    || route.description.lowercased().contains(normalizedQuery)
                    || route.locationNotes.lowercased().contains(normalizedQuery)
            }
    }
}

struct RouteSearchView: View {
    @StateObject var viewModel: RouteSearchViewModel

    var body: some View {
        NavigationStack {
            List {
                if viewModel.isSearching {
                    ProgressView("Searching...")
                }

                if let statusMessage = viewModel.statusMessage {
                    Text(statusMessage)
                        .foregroundStyle(.secondary)
                }

                ForEach(viewModel.routes) { route in
                    NavigationLink {
                        RouteDetailView(route: route)
                    } label: {
                        VStack(alignment: .leading, spacing: 4) {
                            Text(route.name)
                                .font(.headline)
                            Text("\(route.grade) • \(route.routeTypes.map(\.rawValue).joined(separator: ", "))")
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }
                    }
                }
            }
            .navigationTitle("Search")
            .searchable(text: $viewModel.query, prompt: "Route name, grade, or notes")
            .onSubmit(of: .search) {
                Task { await viewModel.search() }
            }
            .toolbar {
                Button("Search") {
                    Task { await viewModel.search() }
                }
                .disabled(viewModel.query.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
            }
        }
        .task {
            await viewModel.loadDownloadedRoutes()
        }
    }
}
