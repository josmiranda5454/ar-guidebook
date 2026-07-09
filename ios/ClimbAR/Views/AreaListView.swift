import SwiftUI

@MainActor
final class AreaListViewModel: ObservableObject {
    @Published var areas: [Area] = []
    @Published var errorMessage: String?

    private let api: ClimbARAPI

    init(api: ClimbARAPI) {
        self.api = api
    }

    func load() async {
        do {
            areas = try await api.areas()
        } catch {
            errorMessage = "Could not load climbing areas."
        }
    }
}

struct AreaListView: View {
    @StateObject var viewModel: AreaListViewModel

    var body: some View {
        NavigationStack {
            List(viewModel.areas) { area in
                NavigationLink(area.name) {
                    AreaDetailView(area: area)
                }
            }
            .navigationTitle("Climbing Areas")
            .overlay {
                if let errorMessage = viewModel.errorMessage {
                    ContentUnavailableView("Offline or Unavailable", systemImage: "wifi.slash", description: Text(errorMessage))
                }
            }
        }
        .task {
            await viewModel.load()
        }
    }
}

