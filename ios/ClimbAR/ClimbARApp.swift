import SwiftUI

@main
struct ClimbARApp: App {
    var body: some Scene {
        WindowGroup {
            RootView(api: ClimbARAPI(), packStore: OfflinePackStore())
        }
    }
}

enum ClimbARStyle {
    static let tint = Color(red: 0.08, green: 0.42, blue: 0.46)
}

struct RouteRow: View {
    let route: Route

    var body: some View {
        HStack(spacing: 12) {
            Image(systemName: "figure.climbing")
                .font(.title3.weight(.semibold))
                .foregroundStyle(ClimbARStyle.tint)
                .frame(width: 36, height: 36)
                .background(ClimbARStyle.tint.opacity(0.12), in: Circle())

            VStack(alignment: .leading, spacing: 4) {
                Text(route.name)
                    .font(.body.weight(.semibold))
                Text(routeSummary)
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }

            Spacer(minLength: 4)
        }
        .padding(.vertical, 4)
    }

    private var routeSummary: String {
        let types = route.routeTypes.map(\.rawValue).joined(separator: ", ")
        return types.isEmpty ? route.grade : "\(route.grade) • \(types)"
    }
}

struct EmptyStateCard: View {
    let title: String
    let message: String
    let systemImage: String

    var body: some View {
        ContentUnavailableView {
            Label(title, systemImage: systemImage)
        } description: {
            Text(message)
        }
        .frame(maxWidth: .infinity)
        .padding(.vertical, 24)
    }
}

struct RootView: View {
    let api: ClimbARAPI
    let packStore: OfflinePackStore

    var body: some View {
        TabView {
            AreaListView(viewModel: AreaListViewModel(api: api, packStore: packStore))
                .tabItem {
                    Label("Explore", systemImage: "map")
                }

            RouteSearchView(viewModel: RouteSearchViewModel(api: api, packStore: packStore))
                .tabItem {
                    Label("Search", systemImage: "magnifyingglass")
                }
        }
        .tint(ClimbARStyle.tint)
    }
}
