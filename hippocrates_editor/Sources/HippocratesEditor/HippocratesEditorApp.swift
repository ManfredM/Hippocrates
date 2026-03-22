import SwiftUI
import UniformTypeIdentifiers

@main
struct HippocratesEditorApp: App {
    @StateObject private var appState = AppState()

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(appState)
        }
        .windowStyle(.hiddenTitleBar)
        .commands {
            CommandGroup(replacing: .newItem) {
                Button("Open...") {
                    openFile()
                }
                .keyboardShortcut("o", modifiers: .command)
            }
        }
    }
    
    func openFile() {
        let panel = NSOpenPanel()
        panel.allowsMultipleSelection = false
        panel.canChooseDirectories = false
        panel.canCreateDirectories = false
        
        // Allow .hipp files and plain text
        var allowedTypes: [UTType] = [.plainText]
        if let hippType = UTType(filenameExtension: "hipp") {
            allowedTypes.append(hippType)
        }
        panel.allowedContentTypes = allowedTypes
        
        if panel.runModal() == .OK {
            if let url = panel.url {
                 appState.load(url: url)
            }
        }
    }
}
