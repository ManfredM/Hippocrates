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
    
    @State private var simulationDays: Int = 30
    
    func runPlan(simulate: Bool) {
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
                 let event = ExecutionEvent(name: msg, time: date, category: category, type: type)
                 DispatchQueue.main.async {
                     appState.executionLogs.append(event)
                 }
            }
            
            let onAsk: (AskRequest) -> Void = { request in
                 if simulate {
                     // Auto-answer logic for simulation
                     let answerVal: String
                     logToFile("Simulate: Auto-answering question for \(request.variable_name). Options: \(request.options), Range: \(String(describing: request.range)), DateRange: \(String(describing: request.dateTimeRange)), TimeRange: \(String(describing: request.timeRange))")
                     
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
                     
                     if !request.options.isEmpty {
                         // Pick random option? Or first? Let's pick random to simulate variation.
                         answerVal = request.options.randomElement() ?? request.options[0]
                     } else if let timeRange = request.timeRange, timeRange.count >= 2 {
                         answerVal = timeRange[0]
                     } else if let dateRange = request.dateTimeRange, dateRange.count >= 2 {
                         let start = Date(timeIntervalSince1970: TimeInterval(dateRange[0]) / 1000.0)
                         if request.dateOnly == true {
                             answerVal = formatDate(start)
                         } else {
                             answerVal = formatDateTime(start)
                         }
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
                         if request.style == .Date {
                             let now = Date()
                             if request.dateOnly == true {
                                 answerVal = formatDate(now)
                             } else {
                                 answerVal = formatDateTime(now)
                             }
                         } else {
                             answerVal = "10"
                         }
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
                // Set Abstract Local Time to synchronize Engine with Wall Clock
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
