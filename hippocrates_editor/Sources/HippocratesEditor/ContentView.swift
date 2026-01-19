import SwiftUI

// Helper for file logging
func logToFile(_ message: String) {
    let logPath = "/tmp/hippocrates_debug.log"
    let fileURL = URL(fileURLWithPath: logPath)
    let timestamp = ISO8601DateFormatter().string(from: Date())
    let entry = "\(timestamp): \(message)\n"
    
    guard let data = entry.data(using: .utf8) else { return }
    
    if FileManager.default.fileExists(atPath: logPath) {
        if let fileHandle = try? FileHandle(forWritingTo: fileURL) {
            fileHandle.seekToEndOfFile()
            fileHandle.write(data)
            try? fileHandle.close()
        }
    } else {
        try? data.write(to: fileURL)
    }
}

struct ContentView: View {
    @EnvironmentObject var appState: AppState
    
    @State private var parseStatus: String = "Ready"
    @State private var isError: Bool = false
    @State private var simulationDays: Int = 30
    
    func runPlan(simulate: Bool) {
        let code = appState.planCode
        parseStatus = simulate ? "Simulating..." : "Running..."
        
        // Clear log file on new run
        try? FileManager.default.removeItem(atPath: "/tmp/hippocrates_debug.log")
        logToFile("Starting execution. Simulate: \(simulate)")
        
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
            
            let onLog: (String, Int, Date) -> Void = { msg, type, date in
                 let category: String
                 switch type {
                 case 1: category = "Message"
                 case 2: category = "Question"
                 case 3: category = "Answer"
                 default: category = "Log"
                 }
                 let event = ExecutionEvent(name: msg, time: date, category: category, type: type)
                 DispatchQueue.main.async {
                     appState.executionLogs.append(event)
                 }
            }
            
            let onAsk: (AskRequest) -> Void = { request in
                 if simulate {
                     // Auto-answer logic for simulation
                     let answerVal: String
                     logToFile("Simulate: Auto-answering question for \(request.variable_name). Options: \(request.options), Range: \(String(describing: request.range))")
                     
                     if !request.options.isEmpty {
                         // Pick random option? Or first? Let's pick random to simulate variation.
                         answerVal = request.options.randomElement() ?? request.options[0]
                     } else if let range = request.range {
                         // Pick random value in range
                         // Check style
                         if request.style == .Numeric || request.style == .Likert || request.style == .VisualAnalogueScale(min: 0, max: 0, min_label: "", max_label: "") { // Case matching is hard here due to associated values
                            // Just use range
                            if range.count >= 2 {
                                let val = Double.random(in: range[0]...range[1])
                                // Round if it looks integer-ish?
                                if range[0].truncatingRemainder(dividingBy: 1) == 0 && range[1].truncatingRemainder(dividingBy: 1) == 0 {
                                    answerVal = String(Int(val))
                                } else {
                                    answerVal = String(format: "%.1f", val)
                                }
                            } else {
                                answerVal = "0"
                            }
                         } else {
                             answerVal = "0"
                         }
                     } else {
                         // Fallback
                         logToFile("Simulate: No options or range found. Fallback to 10.")
                         answerVal = "10"
                     }
                     
                     logToFile("Simulate: Selected answer: \(answerVal)")
                     
                     // Delay slightly to mimic user/system latency and ensure loop doesn't spin too tight? 
                     // Or just answer immediately.
                     // We need to access engine. 
                     DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
                         guard let engine = appState.currentEngine else { 
                             logToFile("Simulate Error: Engine not found in appState")
                             return 
                         }
                         // valueJson: wrap strings in quotes, numbers as is?
                         // The engine expects JSON value string.
                         // If string, "val". If number, 10.
                         // My `answer` function does `let json = "\"\(value)\""` which forces string.
                         // We should fix `answer` function too or do it here.
                         // For now, let's look at `answer` function. It treats everything as string.
                         // But `InputMessage` expects `RuntimeValue`. FFI `hippocrates_engine_set_value` takes JSON.
                         // If we send `"10"`, it parses as String("10").
                         // If we send `10`, it parses as Number(10).
                         
                         // Determine JSON format
                         let json: String
                         // Try to parse as number?
                         if let _ = Double(answerVal) {
                             json = answerVal
                         } else {
                             json = "\"\(answerVal)\""
                         }
                         
                         logToFile("Simulate: Sending value \(json) for \(request.variable_name)")
                         let success = engine.setValue(name: request.variable_name, valueJson: json)
                         if success {
                             logToFile("Simulate: setValue successful")
                         } else {
                             logToFile("Simulate Error: setValue failed")
                         }
                         
                         // Log the auto-answer
                         let timestampDate = Date(timeIntervalSince1970: TimeInterval(request.timestamp) / 1000.0)
                         let event = ExecutionEvent(name: "Auto-Answer: \(answerVal)", time: timestampDate, category: "Answer", type: 3)
                         appState.executionLogs.append(event)
                     }
                 } else {
                     DispatchQueue.main.async {
                         appState.pendingQuestion = request
                     }
                 }
            }
            
            // Run/Simulate
            if let engine = HippocratesEditor.HippocratesParser.prepareEngine(code, simulate: simulate, simulationDays: simulationDays, onStep: onStep, onLog: onLog, onAsk: onAsk) {
                DispatchQueue.main.sync {
                    appState.currentEngine = engine
                }
                // Execute in background (already on background queue)
                engine.execute(planName: planName)
            }
            
            DispatchQueue.main.async {
                 // parseStatus = "Running..." // Already set
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
        .sheet(item: $appState.pendingQuestion) { question in
            VStack(spacing: 20) {
                Text(question.question_text)
                    .font(.headline)
                    .multilineTextAlignment(.center)
                
                switch question.style {
                case .Text:
                     TextField("Answer", text: .constant("")) // Placeholder
                        .textFieldStyle(.roundedBorder)
                case .Selection:
                    ForEach(question.options, id: \.self) { option in
                        Button(option) {
                             answer(question: question, value: option)
                        }
                    }
                case .Numeric:
                     // Simple numeric input
                     Text("Numeric Input here")
                default:
                    Text("Unsupported question type")
                }
                
                if question.style == .Text || question.style == .Numeric {
                    // Submit button needed for text/numeric
                    Button("Submit") {
                        // stub
                        answer(question: question, value: "10") // Test value
                    }
                }
            }
            .padding()
            .frame(minWidth: 300)
        }
    }
    
    func answer(question: AskRequest, value: String) {
        // We need access to the running engine instance to set value.
        // Currently `HippocratesParser.run` returns the engine but we don't store it.
        // Quick Fix: Make `appState` hold the active engine or use a singleton for this prototype.
        // Given `appState` is EnvironmentObject, let's store engine there?
        // But `HippocratesEngine` is not ObservableObject (yet).
        // Let's update `runPlan` to store engine in `appState` via a wrapper or direct reference if we make it public.
        
        // Actually, `HippocratesEngine` wrapper creates a new instance every run.
        // We need to store that instance to call `setValue`.
        
        // TODO: Refactor `runPlan` to store engine in `@State` or `appState`.
        // For now, I'll update `AppState` to hold `currentEngine`.
        if let engine = appState.currentEngine {
            // valueJson depends on type.
            // For boolean/text, we wrap in JSON.
            // Simple string for now:
            let json = "\"\(value)\"" // Wrap as JSON string
            _ = engine.setValue(name: question.variable_name, valueJson: json)
            appState.answerQuestion(value: value)
        }
    }

}
