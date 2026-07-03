import MadobeClientCore
import SwiftUI

@main
struct MadobeMacApp: App {
    var body: some Scene {
        WindowGroup {
            ContentView(identity: MadobeClientCore.identity())
        }
    }
}

struct ContentView: View {
    let identity: MadobeIdentity

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Madobe")
                .font(.title)
                .fontWeight(.semibold)
            Text(identity.statusLine)
                .font(.system(.body, design: .monospaced))
                .textSelection(.enabled)
        }
        .padding(24)
        .frame(minWidth: 520, minHeight: 180)
    }
}
