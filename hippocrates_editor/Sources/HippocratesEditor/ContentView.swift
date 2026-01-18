import SwiftUI

struct ContentView: View {
    @EnvironmentObject var appState: AppState
    
    @State private var parseStatus: String = "Ready"
    @State private var isError: Bool = false
    @State private var simulationDays: Int = 30
    
    func runPlan(simulate: Bool) {
        let code = appState.planCode
        parseStatus = simulate ? "Simulating..." : "Running..."
        
        DispatchQueue.global(qos: .userInitiated).async {
            // 1. Identify Plan Name
            var planName = "TreatmentPlan"
            let parseResult = HippocratesParser.parse(input: code)
            
            if case .success(let plan) = parseResult {
                // Find first plan definition
                for def in plan.definitions {
                    if let planDef = def.Plan {
                        planName = planDef.name
                        break
                    }
                }
            }
            
            DispatchQueue.main.async {
                appState.executionLogs.removeAll()
            }
            
            let onStep: (Int) -> Void = { line in
                 DispatchQueue.main.async {
                     appState.currentExecutionLine = line
                 }
            }
            
            let onLog: (String, Date) -> Void = { msg, date in
                 let event = ExecutionEvent(name: msg, time: date, category: "Log")
                 DispatchQueue.main.async {
                     appState.executionLogs.append(event)
                 }
            }
            
            if simulate {
                HippocratesEditor.HippocratesParser.simulate(input: code, planName: planName, days: simulationDays, onStep: onStep, onLog: onLog)
            } else {
                HippocratesEditor.HippocratesParser.run(input: code, planName: planName, onStep: onStep, onLog: onLog)
            }

            DispatchQueue.main.async {
                appState.currentExecutionLine = nil
                parseStatus = simulate ? "Simulation Finished" : "Execution Finished"
            }
        }
    }

    var body: some View {
        HSplitView {
            // Editor / Visualizer Area
            VStack(alignment: .leading) {
                HStack {
                    Text(appState.currentFileURL?.lastPathComponent ?? "Untitled Plan")
                        .font(.headline)
                    Spacer()
                    
                    HStack(spacing: 12) {
                        Button(action: { runPlan(simulate: false) }) {
                            Label("Run", systemImage: "play.fill")
                        }
                        .keyboardShortcut("r", modifiers: .command)
                        
                        Divider().frame(height: 20)
                        
                        Text("Simulate:")
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                        
                        Stepper("\(simulationDays) days", value: $simulationDays, in: 1...365)
                            .fixedSize()
                        
                        Button(action: { runPlan(simulate: true) }) {
                            Label("Go", systemImage: "clock.arrow.2.circlepath")
                        }
                    }
                }
                .padding(.horizontal)
                .padding(.top)
                
                CodeVisualizerView(code: appState.planCode, highlightedLine: appState.currentExecutionLine)
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
