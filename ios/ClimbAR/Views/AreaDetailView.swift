import SwiftUI

@MainActor
final class AreaDetailViewModel: ObservableObject {
    @Published var area: Area
    @Published var isDownloaded = false
    @Published var downloadedVersion: UInt32?
    @Published var isDownloading = false
    @Published var isRemoving = false
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
        if let pack = try? await packStore.load(areaId: area.id) {
            downloadedVersion = pack.version
        }

        do {
            area = try await api.area(id: area.id)
            return
        } catch {
            guard let pack = try? await packStore.load(areaId: area.id),
                  let cachedArea = pack.areas.first else { return }
            downloadedVersion = pack.version
            area = cachedArea
        }
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
            downloadedVersion = pack.version
            statusMessage = "Offline pack v\(pack.version) downloaded."
        } catch {
            statusMessage = "Could not download this area."
        }
    }

    func deleteArea() async {
        isRemoving = true
        statusMessage = nil
        defer { isRemoving = false }

        do {
            try await packStore.delete(areaId: area.id)
            isDownloaded = false
            downloadedVersion = nil
            statusMessage = "Removed from offline storage."
        } catch {
            statusMessage = "Could not remove this offline area."
        }
    }

    func refresh() async {
        if isDownloaded {
            await downloadArea()
        } else {
            do {
                area = try await api.area(id: area.id)
                statusMessage = "Area refreshed."
            } catch {
                await loadCachedPack()
                statusMessage = "Could not refresh this area."
            }
        }
    }
}

struct AreaDetailView: View {
    @StateObject private var viewModel: AreaDetailViewModel
    @State private var isDeleteConfirmationPresented = false

    init(area: Area, api: ClimbARAPI, packStore: OfflinePackStore) {
        _viewModel = StateObject(
            wrappedValue: AreaDetailViewModel(area: area, api: api, packStore: packStore)
        )
    }

    var body: some View {
        List {
            Section {
                VStack(alignment: .leading, spacing: 6) {
                    Text("Plan your day outside")
                        .font(.headline)
                    Text("Download this area before you leave service. Routes and walls will stay available offline.")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                }
                .padding(.vertical, 4)

                Button {
                    Task { await viewModel.downloadArea() }
                } label: {
                    Label(
                        viewModel.isDownloaded ? "Update Offline Area" : "Download Area",
                        systemImage: viewModel.isDownloaded ? "arrow.clockwise.circle" : "arrow.down.circle"
                    )
                }
                .disabled(viewModel.isDownloading || viewModel.isRemoving)

                if viewModel.isDownloaded {
                    Button(role: .destructive) {
                        isDeleteConfirmationPresented = true
                    } label: {
                        Label("Remove Offline Area", systemImage: "trash")
                    }
                    .disabled(viewModel.isDownloading || viewModel.isRemoving)
                }

                if viewModel.isDownloading || viewModel.isRemoving {
                    ProgressView(viewModel.isRemoving ? "Removing..." : "Downloading...")
                }

                if let statusMessage = viewModel.statusMessage {
                    Text(statusMessage)
                        .foregroundStyle(.secondary)
                }
                if let downloadedVersion = viewModel.downloadedVersion {
                    Label("Offline pack v\(downloadedVersion)", systemImage: "checkmark.circle.fill")
                        .font(.caption)
                        .foregroundStyle(.green)
                }
            }

            Section("About") {
                Text(viewModel.area.description)
                if let accessNotes = viewModel.area.accessNotes {
                    Text(accessNotes)
                }
            }

            Section("Walls · \(viewModel.area.walls.count)") {
                ForEach(viewModel.area.walls) { wall in
                    NavigationLink {
                        WallDetailView(wall: wall)
                    } label: {
                        VStack(alignment: .leading, spacing: 3) {
                            Text(wall.name)
                                .font(.body.weight(.semibold))
                            Text("\(wall.routes.count) route\(wall.routes.count == 1 ? "" : "s")")
                                .font(.subheadline)
                                .foregroundStyle(.secondary)
                        }
                    }
                }
            }
        }
        .listStyle(.insetGrouped)
        .tint(ClimbARStyle.tint)
        .navigationTitle(viewModel.area.name)
        .confirmationDialog(
            "Remove offline area?",
            isPresented: $isDeleteConfirmationPresented,
            titleVisibility: .visible
        ) {
            Button("Remove Offline Area", role: .destructive) {
                Task { await viewModel.deleteArea() }
            }
            Button("Cancel", role: .cancel) {}
        } message: {
            Text("This removes the downloaded walls and routes from this device. You can download the area again later.")
        }
        .refreshable {
            await viewModel.refresh()
        }
        .task {
            await viewModel.loadCachedPack()
        }
    }
}
