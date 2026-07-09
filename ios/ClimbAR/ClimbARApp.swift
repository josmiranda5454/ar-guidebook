import SwiftUI

@main
struct ClimbARApp: App {
    var body: some Scene {
        WindowGroup {
            RootView(api: ClimbARAPI(), packStore: OfflinePackStore())
        }
    }
}

struct RootView: View {
    let api: ClimbARAPI
    let packStore: OfflinePackStore

    var body: some View {
        TabView {
            AreaListView(viewModel: AreaListViewModel(api: api, packStore: packStore))
                .tabItem {
                    Label("Areas", systemImage: "map")
                }

            RouteSearchView(viewModel: RouteSearchViewModel(api: api, packStore: packStore))
                .tabItem {
                    Label("Search", systemImage: "magnifyingglass")
                }
        }
    }
}
