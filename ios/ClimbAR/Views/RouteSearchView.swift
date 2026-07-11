import SwiftUI
import CoreLocation

@MainActor
final class RouteSearchViewModel: ObservableObject {
    @Published var query = ""
    @Published var routes: [Route] = []
    @Published var isSearching = false
    @Published var statusMessage: String?
    @Published var nearbyRoutes: [NearbyRoute] = []
    @Published var isLoadingNearby = false

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

    func refresh() async {
        if query.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            await loadDownloadedRoutes()
        } else {
            await search()
        }
    }

    func loadNearby(location: CLLocation?) async {
        guard let location else {
            statusMessage = "Allow location access to find nearby routes."
            return
        }
        isLoadingNearby = true
        defer { isLoadingNearby = false }
        do {
            nearbyRoutes = try await api.nearbyRoutes(
                latitude: location.coordinate.latitude,
                longitude: location.coordinate.longitude
            ).sorted { $0.distanceMeters < $1.distanceMeters }
            statusMessage = nearbyRoutes.isEmpty ? "No routes were found nearby." : nil
        } catch {
            statusMessage = "Could not load nearby routes."
        }
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
    @StateObject private var locationService = LocationService()

    var body: some View {
        NavigationStack {
            List {
                if viewModel.isSearching || viewModel.isLoadingNearby {
                    HStack(spacing: 10) {
                        ProgressView()
                        Text(viewModel.isSearching ? "Searching routes..." : "Finding routes near you...")
                            .foregroundStyle(.secondary)
                    }
                }

                if let statusMessage = viewModel.statusMessage {
                    Text(statusMessage)
                        .foregroundStyle(.secondary)
                }

                ForEach(viewModel.routes) { route in
                    NavigationLink {
                        RouteDetailView(route: route)
                    } label: {
                        RouteRow(route: route)
                    }
                }

                if viewModel.routes.isEmpty,
                   !viewModel.isSearching,
                   viewModel.query.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty,
                   viewModel.nearbyRoutes.isEmpty {
                    EmptyStateCard(
                        title: "Search the guidebook",
                        message: "Look up a route by name, grade, or notes. Downloaded areas work offline too.",
                        systemImage: "magnifyingglass"
                    )
                    .listRowBackground(Color.clear)
                }

                if !viewModel.nearbyRoutes.isEmpty {
                    Section("Nearby Routes") {
                        ForEach(viewModel.nearbyRoutes) { nearby in
                            NavigationLink {
                                RouteDetailView(route: nearby.route)
                        } label: {
                            HStack {
                                RouteRow(route: nearby.route)
                                Spacer(minLength: 4)
                                Text(formattedDistance(nearby.distanceMeters))
                                    .font(.caption.weight(.medium))
                                    .foregroundStyle(.secondary)
                            }
                        }
                        }
                    }
                }
            }
            .listStyle(.insetGrouped)
            .navigationTitle("Find a route")
            .searchable(text: $viewModel.query, prompt: "Route name, grade, or notes")
            .onSubmit(of: .search) {
                Task { await viewModel.search() }
            }
            .toolbar {
                Button {
                    Task { await viewModel.search() }
                } label: {
                    Image(systemName: "magnifyingglass")
                }
                .accessibilityLabel("Search routes")
                .disabled(viewModel.query.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)

                Button {
                    locationService.requestLocation()
                    Task { await viewModel.loadNearby(location: locationService.userLocation) }
                } label: {
                    Label("Nearby", systemImage: "location.fill")
                }
                .disabled(viewModel.isLoadingNearby)
            }
            .tint(ClimbARStyle.tint)
            .refreshable {
                await viewModel.refresh()
            }
        }
        .task {
            await viewModel.loadDownloadedRoutes()
        }
        .onChange(of: locationService.userLocation) { _, location in
            guard location != nil else { return }
            Task { await viewModel.loadNearby(location: location) }
        }
    }

    private func formattedDistance(_ meters: Double) -> String {
        meters >= 1000 ? String(format: "%.1f km", meters / 1000) : "\(Int(meters.rounded())) m"
    }
}
