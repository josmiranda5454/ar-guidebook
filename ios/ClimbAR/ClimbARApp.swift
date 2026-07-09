import SwiftUI

@main
struct ClimbARApp: App {
    var body: some Scene {
        WindowGroup {
            AreaListView(viewModel: AreaListViewModel(api: ClimbARAPI()))
        }
    }
}

