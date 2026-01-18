import SwiftUI

struct ContentView: View {
    @EnvironmentObject var appState: AppState
    
    @State private var parseStatus: String = "Ready"
    @State private var isError: Bool = false
    
    var body: some View {
        HSplitView {
            // Editor / Visualizer Area
            VStack(alignment: .leading) {
                Text(appState.currentFileURL?.lastPathComponent ?? "Untitled Plan")
                    .font(.headline)
                    .padding(.horizontal)
                    .padding(.top)
                
                CodeVisualizerView(code: appState.planCode)
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                    .background(Color(nsColor: .textBackgroundColor))
                    .onChange(of: appState.planCode, initial: true) { _, newValue in
                         let result = HippocratesParser.parse(input: newValue)
                         switch result {
                         case .success(let plan):
                             parseStatus = "Valid Plan: \(plan.definitions.count) definitions"
                             isError = false
                         case .failure(let error):
                             parseStatus = "Error: \(error.localizedDescription)"
                             isError = true
                         }
                    }
                
                Text(parseStatus)
                    .font(.caption)
                    .foregroundStyle(isError ? .red : .green)
                    .padding()
            }
            .layoutPriority(1)
            
            // Execution Visualizer Area
            VStack(alignment: .leading) {
                Text("Execution Timeline")
                    .font(.headline)
                    .padding(.horizontal)
                    .padding(.top)
                    
                ExecutionTimelineView()
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            }
            .layoutPriority(1)
        }
        .frame(minWidth: 900, minHeight: 600)
    }
}
