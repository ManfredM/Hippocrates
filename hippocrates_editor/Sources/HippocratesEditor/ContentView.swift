import SwiftUI
import AppKit

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

private struct MessagePayload: Decodable {
    let kind: String
    let text: String
    let addressees: [MessageAddressee]?
}

private struct MessageAddressee: Decodable {
    let name: String
}

private func messageDisplayText(from msg: String, type: Int) -> String {
    guard type == 1, let data = msg.data(using: .utf8) else { return msg }
    guard let payload = try? JSONDecoder().decode(MessagePayload.self, from: data) else { return msg }
    let suffix = (payload.addressees ?? []).map { $0.name }.joined(separator: ", ")
    if suffix.isEmpty {
        return payload.text
    }
    return "\(payload.text) (to: \(suffix))"
}

private struct SimulationAnswerEntry {
    let valueJson: String
    let valueDisplay: String
    let delaySeconds: TimeInterval?
}

private struct SimulationAnswerFile {
    let answersByVariable: [String: [SimulationAnswerEntry]]
    let simulationMinutes: Int
    let simulationDays: Int
}

private enum SimulationAnswerError: LocalizedError {
    case invalidFormat(String)
    case missingEntries

    var errorDescription: String? {
        switch self {
        case .invalidFormat(let message):
            return message
        case .missingEntries:
            return "Simulation answers file does not contain any answers."
        }
    }
}

private final class SimulationAnswerQueue {
    private var queues: [String: [SimulationAnswerEntry]]
    private let lock = NSLock()

    init(answersByVariable: [String: [SimulationAnswerEntry]]) {
        var normalized: [String: [SimulationAnswerEntry]] = [:]
        for (variable, entries) in answersByVariable {
            let key = formatVariableName(variable)
            normalized[key] = entries
        }
        self.queues = normalized
    }

    func nextAnswer(for variable: String) -> SimulationAnswerEntry? {
        let key = formatVariableName(variable)
        lock.lock()
        defer { lock.unlock() }
        guard var list = queues[key], !list.isEmpty else { return nil }
        let entry = list.removeFirst()
        queues[key] = list
        return entry
    }
}

private func formatVariableName(_ name: String) -> String {
    let trimmed = name.trimmingCharacters(in: .whitespacesAndNewlines)
    if trimmed.hasPrefix("<"), trimmed.hasSuffix(">"), trimmed.count > 2 {
        return trimmed
    }
    return "<\(trimmed)>"
}

private func promptSimulationAnswersFile() -> URL? {
    let panel = NSOpenPanel()
    panel.canChooseFiles = true
    panel.canChooseDirectories = false
    panel.allowsMultipleSelection = false
    panel.allowedContentTypes = [.json]
    return panel.runModal() == .OK ? panel.url : nil
}

private func jsonString(from value: Any) -> String? {
    if JSONSerialization.isValidJSONObject(value) {
        if let data = try? JSONSerialization.data(withJSONObject: value, options: []) {
            return String(data: data, encoding: .utf8)
        }
        return nil
    }
    if JSONSerialization.isValidJSONObject([value]) {
        if let data = try? JSONSerialization.data(withJSONObject: [value], options: []),
           let str = String(data: data, encoding: .utf8),
           str.hasPrefix("["),
           str.hasSuffix("]") {
            return String(str.dropFirst().dropLast())
        }
    }
    return nil
}

private func displayString(from value: Any) -> String {
    if let s = value as? String { return s }
    if let n = value as? NSNumber { return n.stringValue }
    if value is NSNull { return "null" }
    if let json = jsonString(from: value) { return json }
    return "\(value)"
}

private func parseQuantityString(_ input: String) -> (Double, String)? {
    let trimmed = input.trimmingCharacters(in: .whitespacesAndNewlines)
    if trimmed.isEmpty { return nil }

    let parts = trimmed.split(separator: " ")
    if parts.count >= 2, let value = Double(parts[0]) {
        let unit = parts.dropFirst().joined(separator: " ")
        return (value, unit)
    }

    var endIndex = trimmed.startIndex
    var sawDigit = false
    for idx in trimmed.indices {
        let ch = trimmed[idx]
        if idx == trimmed.startIndex && (ch == "+" || ch == "-") {
            endIndex = trimmed.index(after: idx)
            continue
        }
        if ch.isNumber || ch == "." {
            endIndex = trimmed.index(after: idx)
            if ch.isNumber { sawDigit = true }
            continue
        }
        break
    }

    guard sawDigit else { return nil }
    let numStr = String(trimmed[..<endIndex])
    let unitStr = String(trimmed[endIndex...]).trimmingCharacters(in: .whitespacesAndNewlines)
    guard let value = Double(numStr), !unitStr.isEmpty else { return nil }
    return (value, unitStr)
}

private func secondsForUnit(_ unit: String) -> Double? {
    let trimmed = unit.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
    switch trimmed {
    case "s", "sec", "secs", "second", "seconds":
        return 1
    case "m", "min", "mins", "minute", "minutes":
        return 60
    case "h", "hr", "hrs", "hour", "hours":
        return 3600
    case "d", "day", "days":
        return 86400
    case "w", "week", "weeks":
        return 604800
    case "month", "months":
        return 2592000
    case "y", "yr", "yrs", "year", "years":
        return 31536000
    default:
        return nil
    }
}

private func parseDelaySeconds(_ value: Any) throws -> TimeInterval {
    if let number = value as? NSNumber {
        let secs = number.doubleValue
        if secs <= 0 { throw SimulationAnswerError.invalidFormat("Delay must be greater than zero.") }
        return secs
    }
    if let string = value as? String {
        let trimmed = string.trimmingCharacters(in: .whitespacesAndNewlines)
        if let number = Double(trimmed) {
            if number <= 0 { throw SimulationAnswerError.invalidFormat("Delay must be greater than zero.") }
            return number
        }
        guard let (amount, unit) = parseQuantityString(trimmed) else {
            throw SimulationAnswerError.invalidFormat("Expected a duration like '30 days' or '12 hours'.")
        }
        if amount <= 0 { throw SimulationAnswerError.invalidFormat("Delay must be greater than zero.") }
        guard let factor = secondsForUnit(unit) else {
            throw SimulationAnswerError.invalidFormat("Unsupported delay unit '\(unit)'. Use seconds, minutes, hours, days, weeks, months, or years.")
        }
        return amount * factor
    }
    if let obj = value as? [String: Any] {
        guard let amountValue = obj["value"] else {
            throw SimulationAnswerError.invalidFormat("Delay object must include 'value'.")
        }
        guard let unitValue = (obj["unit"] ?? obj["units"]) as? String else {
            throw SimulationAnswerError.invalidFormat("Delay object must include a string 'unit'.")
        }
        let amount: Double?
        if let num = amountValue as? NSNumber {
            amount = num.doubleValue
        } else if let str = amountValue as? String {
            amount = Double(str)
        } else {
            amount = nil
        }
        guard let amt = amount else {
            throw SimulationAnswerError.invalidFormat("Delay value must be a valid number.")
        }
        if amt <= 0 { throw SimulationAnswerError.invalidFormat("Delay must be greater than zero.") }
        guard let factor = secondsForUnit(unitValue) else {
            throw SimulationAnswerError.invalidFormat("Unsupported delay unit '\(unitValue)'. Use seconds, minutes, hours, days, weeks, months, or years.")
        }
        return amt * factor
    }
    throw SimulationAnswerError.invalidFormat("Delay must be a string or object.")
}

private func quantityString(from value: Any) -> String? {
    guard let obj = value as? [String: Any] else { return nil }
    guard let unit = (obj["unit"] ?? obj["units"]) as? String else { return nil }
    guard let amountValue = obj["value"] else { return nil }
    let amount: String
    if let num = amountValue as? NSNumber {
        amount = num.stringValue
    } else if let str = amountValue as? String {
        amount = str
    } else {
        return nil
    }
    let trimmedUnit = unit.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !trimmedUnit.isEmpty else { return nil }
    return "\(amount) \(trimmedUnit)"
}

private func parseAnswerEntry(_ value: Any) throws -> (value: Any, delaySeconds: TimeInterval?) {
    if var obj = value as? [String: Any] {
        var delay: TimeInterval? = nil
        if let delayValue = obj.removeValue(forKey: "delay") {
            delay = try parseDelaySeconds(delayValue)
        }

        if obj.keys.contains("value"),
           obj["unit"] == nil,
           obj["units"] == nil,
           obj.count == 1 {
            let inner = obj["value"] ?? NSNull()
            return (inner, delay)
        }
        return (obj, delay)
    }
    return (value, nil)
}

private func makeSimulationAnswerEntry(value: Any, delaySeconds: TimeInterval?) throws -> SimulationAnswerEntry {
    if let quantity = quantityString(from: value) {
        guard let valueJson = jsonString(from: quantity) else {
            throw SimulationAnswerError.invalidFormat("Unable to encode quantity value.")
        }
        return SimulationAnswerEntry(valueJson: valueJson, valueDisplay: quantity, delaySeconds: delaySeconds)
    }
    guard let valueJson = jsonString(from: value) else {
        throw SimulationAnswerError.invalidFormat("Unable to encode answer value.")
    }
    return SimulationAnswerEntry(valueJson: valueJson, valueDisplay: displayString(from: value), delaySeconds: delaySeconds)
}

private func pushAnswer(_ map: inout [String: [SimulationAnswerEntry]], variable: String, value: Any) throws {
    let parsed = try parseAnswerEntry(value)
    let entry = try makeSimulationAnswerEntry(value: parsed.value, delaySeconds: parsed.delaySeconds)
    map[variable, default: []].append(entry)
}

private func pushAnswerWithDelay(
    _ map: inout [String: [SimulationAnswerEntry]],
    variable: String,
    value: Any,
    delay: Any?
) throws {
    var parsed = try parseAnswerEntry(value)
    if let delayValue = delay {
        let extraDelay = try parseDelaySeconds(delayValue)
        if parsed.delaySeconds != nil {
            throw SimulationAnswerError.invalidFormat("Delay specified multiple times for an answer.")
        }
        parsed.delaySeconds = extraDelay
    }
    let entry = try makeSimulationAnswerEntry(value: parsed.value, delaySeconds: parsed.delaySeconds)
    map[variable, default: []].append(entry)
}

private func loadSimulationAnswers(from url: URL) throws -> SimulationAnswerFile {
    let data = try Data(contentsOf: url)
    let json = try JSONSerialization.jsonObject(with: data, options: [])

    var answersByVariable: [String: [SimulationAnswerEntry]] = [:]

    if let obj = json as? [String: Any] {
        for (key, value) in obj {
            let name = formatVariableName(key)
            if let arr = value as? [Any] {
                for item in arr {
                    try pushAnswer(&answersByVariable, variable: name, value: item)
                }
            } else {
                try pushAnswer(&answersByVariable, variable: name, value: value)
            }
        }
    } else if let items = json as? [Any] {
        for item in items {
            guard let obj = item as? [String: Any] else {
                throw SimulationAnswerError.invalidFormat("Answer list items must be objects.")
            }
            guard let variable = obj["variable"] as? String else {
                throw SimulationAnswerError.invalidFormat("Answer items must include a string 'variable' field.")
            }
            guard let value = obj["value"] else {
                throw SimulationAnswerError.invalidFormat("Answer items must include a 'value' field.")
            }
            let delay = obj["delay"]
            let name = formatVariableName(variable)
            try pushAnswerWithDelay(&answersByVariable, variable: name, value: value, delay: delay)
        }
    } else {
        throw SimulationAnswerError.invalidFormat("Answers file must be a JSON object or array.")
    }

    guard !answersByVariable.isEmpty else {
        throw SimulationAnswerError.missingEntries
    }

    let maxCount = answersByVariable.values.map { $0.count }.max() ?? 0
    let maxDelay = answersByVariable.values
        .flatMap { $0.compactMap { $0.delaySeconds } }
        .max() ?? 0
    let baseMinutes = max(1, maxCount) * 24 * 60
    let extraMinutes = Int(ceil(maxDelay / 60.0))
    let totalMinutes = max(1, baseMinutes + extraMinutes)
    let totalDays = max(1, Int(ceil(Double(totalMinutes) / 1440.0)))

    return SimulationAnswerFile(
        answersByVariable: answersByVariable,
        simulationMinutes: totalMinutes,
        simulationDays: totalDays
    )
}

struct ContentView: View {
    @EnvironmentObject var appState: AppState
    
    @State private var simulationDays: Int = 30
    
    func runPlan(simulate: Bool) {
        var simulationFile: SimulationAnswerFile?
        if simulate {
            guard let url = promptSimulationAnswersFile() else {
                appState.parseStatus = "Simulation canceled"
                return
            }
            do {
                simulationFile = try loadSimulationAnswers(from: url)
                if let days = simulationFile?.simulationDays {
                    simulationDays = days
                }
            } catch {
                appState.parseStatus = "Simulation Error: \(error.localizedDescription)"
                return
            }
        }

        let code = appState.planCode
        appState.parseStatus = simulate ? "Simulating..." : "Running..."
        appState.currentErrors = []
        
        // Clear log file on new run
        try? FileManager.default.removeItem(atPath: "/tmp/hippocrates_debug.log")
        logToFile("Starting execution. Simulate: \(simulate)")
        
        // Stop previous engine immediately (Main Thread)
        if let current = appState.currentEngine {
            current.stop()
        }
        
        DispatchQueue.global(qos: .userInitiated).async {
            // 2. Identify Plan Name
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
                
                // 2. Validate (Semantics) - Multi-error
                let validationErrors = HippocratesParser.validate(input: code)
                if !validationErrors.isEmpty {
                    DispatchQueue.main.async {
                        self.appState.parseStatus = "Validation Failed: \(validationErrors.count) error(s)"
                        self.appState.currentErrors = validationErrors
                    }
                    return // Stop if invalid
                }
                
            } else if case .failure(let error) = parseResult {
                DispatchQueue.main.async {
                    self.appState.parseStatus = "Syntax Error: 1 error(s)"
                    self.appState.currentErrors = [error]
                }
                return
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
                 let displayMsg = messageDisplayText(from: msg, type: type)
                 let event = ExecutionEvent(name: displayMsg, time: date, category: category, type: type)
                 DispatchQueue.main.async {
                     appState.executionLogs.append(event)
                }
            }
            
            let answerQueue = simulationFile.map { SimulationAnswerQueue(answersByVariable: $0.answersByVariable) }
            let simulationDurationMinutes = simulationFile?.simulationMinutes
            
            let onAsk: (AskRequest) -> Void = { request in
                 if simulate {
                     // Auto-answer logic for simulation
                     logToFile("Simulate: Auto-answering question for \(request.variable_name). Options: \(request.options), Range: \(String(describing: request.range)), DateRange: \(String(describing: request.dateTimeRange)), TimeRange: \(String(describing: request.timeRange))")
                     
                     guard let entry = answerQueue?.nextAnswer(for: request.variable_name) else {
                         logToFile("Simulate Error: No answer available for \(request.variable_name)")
                         DispatchQueue.main.async {
                             appState.parseStatus = "Simulation Error: No answer for \(request.variable_name)"
                             appState.currentEngine?.stop()
                         }
                         return
                     }
                     
                     let answerVal = entry.valueDisplay
                     let delayMs = Int64((entry.delaySeconds ?? 0) * 1000.0)
                     let targetTimestamp = request.timestamp + delayMs
                     
                     DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
                         guard let engine = appState.currentEngine else {
                             logToFile("Simulate Error: Engine not found in appState")
                             return
                         }
                         
                         logToFile("Simulate: Sending value \(entry.valueJson) for \(request.variable_name) at \(targetTimestamp)")
                         let success = engine.setValueAt(name: request.variable_name, valueJson: entry.valueJson, timestampMs: targetTimestamp)
                         if success {
                             logToFile("Simulate: setValueAt successful")
                         } else {
                             logToFile("Simulate Error: setValueAt failed")
                         }
                         
                         let eventDate = Date(timeIntervalSince1970: TimeInterval(targetTimestamp) / 1000.0)
                         let event = ExecutionEvent(name: "Auto-Answer: \(answerVal)", time: eventDate, category: "Answer", type: 3)
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
                if let durationMinutes = simulationDurationMinutes {
                    engine.setSimulationMode(durationMinutes: durationMinutes)
                }
                // Set Abstract Local Time to current wall clock (matches CLI default).
                engine.setTime(Date())
                
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
                
                CodeVisualizerView(code: appState.planCode, highlightedLine: appState.currentExecutionLine, errors: appState.currentErrors, engine: appState.currentEngine, simulationDays: simulationDays)
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                    .background(Color(nsColor: .textBackgroundColor))
                    .onChange(of: appState.planCode, initial: true) { _, newValue in
                         // Live Syntax Check
                         let result = HippocratesParser.parse(input: newValue)
                         switch result {
                         case .success(let plan):
                             // Live Validation
                             let validationErrors = HippocratesParser.validate(input: newValue)
                             if !validationErrors.isEmpty {
                                 appState.parseStatus = "Validation Failed: \(validationErrors.count) error(s)"
                                 appState.currentErrors = validationErrors
                             } else {
                                 appState.parseStatus = "Valid Syntax: \(plan.definitions.count) definitions"
                                 appState.currentErrors = []
                                 
                                 // Initialize Visualization Engine
                                 if let vizEngine = HippocratesParser.prepareEngine(newValue, simulate: true, simulationDays: 7, onStep: { _ in }, onLog: { _,_,_ in }, onAsk: { _ in }) {
                                     // We use simulation mode to allow prediction functions to work if needed, 
                                     // though simulateOccurrences manages its own transient state usually.
                                     appState.visualizationEngine = vizEngine
                                 }
                             }
                         case .failure(let error):
                             appState.parseStatus = "Syntax Error"
                             appState.currentErrors = [error]
                         }
                    }
                
                Text(appState.parseStatus)
                    .font(.caption)
                    .foregroundStyle(appState.currentErrors.isEmpty ? .green : .red)
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
            QuestionSheetView(question: question) { answerValue in
                answer(question: question, value: answerValue)
            }
        }
    }
    
    @discardableResult
    func answer(question: AskRequest, value: String) -> Bool {
        guard let engine = appState.currentEngine else { return false }

        // Determine if value is number or string and format JSON accordingly
        // For now, if it parses as double, treat as number (raw), else string (quoted)
        let json: String
        if let _ = Double(value) {
            json = value
        } else {
            json = "\"\(value)\""
        }

        let success = engine.setValue(name: question.variable_name, valueJson: json)
        if success {
            appState.answerQuestion(value: value)
        }
        return success
    }
}

struct QuestionSheetView: View {
    let question: AskRequest
    let onAnswer: (String) -> Bool
    
    @State private var textInput: String = ""
    @State private var errorMessage: String?
    @State private var selectedDateTime: Date = Date()
    @State private var selectedTime: Date = Date()
    @State private var didInitializeDateSelection: Bool = false
    
    // Double-entry validation state
    @State private var previousValue: String?
    @State private var confirmationMode: Bool = false
    
    var body: some View {
        VStack(spacing: 20) {
            Text(confirmationMode ? "Please confirm your answer" : question.question_text)
                .font(.headline)
                .multilineTextAlignment(.center)
            
            if confirmationMode {
                Text("(Enter value again)")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            
            switch question.style {
            case .Text:
                TextField("Your Answer", text: $textInput)
                    .textFieldStyle(.roundedBorder)
                    .frame(width: 200)
                    .onChange(of: textInput) { _, _ in errorMessage = nil }
                    
            case .Selection, .Likert, .VisualAnalogueScale:
                // If options are available, use Selection style
                if !question.options.isEmpty {
                    VStack(spacing: 8) {
                        ForEach(question.options, id: \.self) { option in
                            Button(option) {
                                handleSelection(option)
                            }
                            .controlSize(.large)
                        }
                    }
                    .frame(maxHeight: 300)
                } else if question.style == .VisualAnalogueScale(min: 0, max: 0, min_label: "", max_label: "") || question.style == .Numeric || question.style == .Likert {
                         // Fallback to numeric input if VAS/Likert has no options but is numeric based
                         TextField("Number", text: $textInput)
                            #if os(iOS)
                            .keyboardType(.decimalPad)
                            #endif
                            .textFieldStyle(.roundedBorder)
                            .frame(width: 120)
                            .onChange(of: textInput) { _, _ in errorMessage = nil }
                } else {
                    Text("No options available for selection")
                }
                
            case .Numeric:
                 TextField("Number", text: $textInput)
                    #if os(iOS)
                    .keyboardType(.decimalPad)
                    #endif
                    .textFieldStyle(.roundedBorder)
                    .frame(width: 120)
                    .onChange(of: textInput) { _, _ in errorMessage = nil }
                
            case .Date:
                if let timeRange = question.timeRange, timeRange.count >= 2 {
                    if let range = timeRangeBounds() {
                        DatePicker(
                            "Time",
                            selection: $selectedTime,
                            in: range,
                            displayedComponents: [.hourAndMinute]
                        )
                        .datePickerStyle(.compact)
                        .onChange(of: selectedTime) { _, _ in errorMessage = nil }
                    } else {
                        DatePicker(
                            "Time",
                            selection: $selectedTime,
                            displayedComponents: [.hourAndMinute]
                        )
                        .datePickerStyle(.compact)
                        .onChange(of: selectedTime) { _, _ in errorMessage = nil }
                    }
                } else {
                    let components: DatePickerComponents = (question.dateOnly ?? false) ? [.date] : [.date, .hourAndMinute]
                    if let range = dateRangeBounds() {
                        DatePicker(
                            "Date",
                            selection: $selectedDateTime,
                            in: range,
                            displayedComponents: components
                        )
                        .datePickerStyle(.compact)
                        .onChange(of: selectedDateTime) { _, _ in errorMessage = nil }
                    } else {
                        DatePicker(
                            "Date",
                            selection: $selectedDateTime,
                            displayedComponents: components
                        )
                        .datePickerStyle(.compact)
                        .onChange(of: selectedDateTime) { _, _ in errorMessage = nil }
                    }
                }
                
            default:
                Text("Unsupported question type")
            }
            
            if let error = errorMessage {
                Text(error)
                    .foregroundStyle(.red)
                    .font(.caption)
            }
            
            if question.style == .Text || question.style == .Numeric || question.style == .Date {
                Button(confirmationMode ? "Confirm" : "Submit") {
                    validateAndSubmit()
                }
                .keyboardShortcut(.defaultAction)
                .disabled((question.style == .Text || question.style == .Numeric) && textInput.isEmpty)
            }
        }
        .padding()
        .frame(minWidth: 300, minHeight: 200)
        .onAppear {
            if !didInitializeDateSelection {
                initializeDateSelection()
                didInitializeDateSelection = true
            }
        }
    }
    
    func handleSelection(_ value: String) {
        if question.validation_mode == .Twice {
            if confirmationMode {
                if value == previousValue {
                    if !onAnswer(value) {
                        errorMessage = "Answer rejected. Please check the allowed format."
                        confirmationMode = false
                        previousValue = nil
                    }
                } else {
                    errorMessage = "Value mismatch! Please try again."
                    confirmationMode = false
                    previousValue = nil
                }
            } else {
                previousValue = value
                confirmationMode = true
                errorMessage = nil 
                // Visual feedback?
            }
        } else {
            if !onAnswer(value) {
                errorMessage = "Answer rejected. Please check the allowed format."
            }
        }
    }
    
    func validateAndSubmit() {
        var valueToSubmit = textInput

        // 1. Basic Type/Range Validation
        if question.style == .Numeric {
            if let value = Double(textInput) {
                if let range = question.range, range.count >= 2 {
                    let min = range[0]
                    let max = range[1]
                    if value < min || value > max {
                        errorMessage = "Value must be between \(String(format: "%.1f", min)) and \(String(format: "%.1f", max))"
                        return
                    }
                }
            } else {
                errorMessage = "Please enter a valid number"
                return
            }
        } else if question.style == .Date {
            if let dateValue = validatedDateValue() {
                valueToSubmit = dateValue
            } else {
                return
            }
        }
        
        // 2. Double Entry Logic
        if question.validation_mode == .Twice {
             if confirmationMode {
                 if valueToSubmit == previousValue {
                     if !onAnswer(valueToSubmit) {
                         errorMessage = "Answer rejected. Please check the allowed format."
                         confirmationMode = false
                         previousValue = nil
                         if question.style == .Text || question.style == .Numeric {
                             textInput = ""
                         }
                     }
                 } else {
                     errorMessage = "Values do not match. Please start over."
                     confirmationMode = false
                     previousValue = nil
                     if question.style == .Text || question.style == .Numeric {
                         textInput = ""
                     }
                 }
             } else {
                 previousValue = valueToSubmit
                 confirmationMode = true
                 if question.style == .Text || question.style == .Numeric {
                     textInput = ""
                 }
                 errorMessage = nil
                 // Maybe focus field again? Swift UI automatically keeps focus usually.
             }
        } else {
            if !onAnswer(valueToSubmit) {
                errorMessage = "Answer rejected. Please check the allowed format."
            }
        }
    }

    func validatedDateValue() -> String? {
        if let timeRange = question.timeRange, timeRange.count >= 2 {
            guard let start = timeDate(from: timeRange[0]),
                  let end = timeDate(from: timeRange[1]) else {
                errorMessage = "Invalid time range configuration"
                return nil
            }
            let minutes = minutesSinceMidnight(selectedTime)
            let startMinutes = minutesSinceMidnight(start)
            let endMinutes = minutesSinceMidnight(end)
            if !timeInRange(value: minutes, start: startMinutes, end: endMinutes) {
                errorMessage = "Time must be between \(timeRange[0]) and \(timeRange[1])"
                return nil
            }
            return formatTime(selectedTime)
        }

        if let dateRange = question.dateTimeRange, dateRange.count >= 2 {
            let start = Date(timeIntervalSince1970: TimeInterval(dateRange[0]) / 1000.0)
            let end = Date(timeIntervalSince1970: TimeInterval(dateRange[1]) / 1000.0)

            if question.dateOnly == true {
                let day = Calendar.current.startOfDay(for: selectedDateTime)
                let minDay = Calendar.current.startOfDay(for: start)
                let maxDay = Calendar.current.startOfDay(for: end)
                if day < minDay || day > maxDay {
                    errorMessage = "Date must be between \(formatDate(minDay)) and \(formatDate(maxDay))"
                    return nil
                }
                return formatDate(selectedDateTime)
            } else {
                if selectedDateTime < start || selectedDateTime > end {
                    errorMessage = "Date/time must be within the allowed range"
                    return nil
                }
                return formatDateTime(selectedDateTime)
            }
        }

        if question.dateOnly == true {
            return formatDate(selectedDateTime)
        }
        return formatDateTime(selectedDateTime)
    }

    func initializeDateSelection() {
        if let timeRange = question.timeRange, timeRange.count >= 2 {
            if let start = timeDate(from: timeRange[0]) {
                selectedTime = start
            }
            return
        }
        if let dateRange = question.dateTimeRange, dateRange.count >= 2 {
            let start = Date(timeIntervalSince1970: TimeInterval(dateRange[0]) / 1000.0)
            selectedDateTime = start
            return
        }
        selectedDateTime = Date()
    }

    func dateRangeBounds() -> ClosedRange<Date>? {
        guard let dateRange = question.dateTimeRange, dateRange.count >= 2 else { return nil }
        let start = Date(timeIntervalSince1970: TimeInterval(dateRange[0]) / 1000.0)
        let end = Date(timeIntervalSince1970: TimeInterval(dateRange[1]) / 1000.0)
        if question.dateOnly == true {
            let minDay = Calendar.current.startOfDay(for: start)
            let maxDay = Calendar.current.startOfDay(for: end)
            return minDay...maxDay
        }
        return start...end
    }

    func timeRangeBounds() -> ClosedRange<Date>? {
        guard let timeRange = question.timeRange, timeRange.count >= 2 else { return nil }
        guard let start = timeDate(from: timeRange[0]),
              let end = timeDate(from: timeRange[1]) else { return nil }

        let startMinutes = minutesSinceMidnight(start)
        let endMinutes = minutesSinceMidnight(end)
        if startMinutes <= endMinutes {
            return start...end
        }
        return nil
    }

    func timeDate(from value: String) -> Date? {
        let parts = value.split(separator: ":")
        if parts.count != 2 { return nil }
        guard let hour = Int(parts[0]), let minute = Int(parts[1]) else { return nil }
        var components = Calendar.current.dateComponents([.year, .month, .day], from: Date())
        components.hour = hour
        components.minute = minute
        components.second = 0
        return Calendar.current.date(from: components)
    }

    func minutesSinceMidnight(_ date: Date) -> Int {
        let comps = Calendar.current.dateComponents([.hour, .minute], from: date)
        return (comps.hour ?? 0) * 60 + (comps.minute ?? 0)
    }

    func timeInRange(value: Int, start: Int, end: Int) -> Bool {
        if start <= end {
            return value >= start && value <= end
        }
        return value >= start || value <= end
    }

    func formatDate(_ date: Date) -> String {
        let formatter = DateFormatter()
        formatter.locale = Locale(identifier: "en_US_POSIX")
        formatter.dateFormat = "yyyy-MM-dd"
        return formatter.string(from: date)
    }

    func formatDateTime(_ date: Date) -> String {
        let formatter = DateFormatter()
        formatter.locale = Locale(identifier: "en_US_POSIX")
        formatter.dateFormat = "yyyy-MM-dd HH:mm"
        return formatter.string(from: date)
    }

    func formatTime(_ date: Date) -> String {
        let formatter = DateFormatter()
        formatter.locale = Locale(identifier: "en_US_POSIX")
        formatter.dateFormat = "HH:mm"
        return formatter.string(from: date)
    }
}
