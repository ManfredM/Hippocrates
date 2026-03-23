import SwiftUI
import AppKit

struct CodeVisualizerView: NSViewRepresentable {
    let code: String
    let highlightedLine: Int?
    let errors: [HippocratesParser.EngineError]
    let engine: HippocratesEngine?
    let simulationDays: Int
    
    init(code: String, highlightedLine: Int?, errors: [HippocratesParser.EngineError] = [], engine: HippocratesEngine? = nil, simulationDays: Int = 30) {
        self.code = code
        self.highlightedLine = highlightedLine
        self.errors = errors
        self.engine = engine
        self.simulationDays = simulationDays
    }
    
    func makeCoordinator() -> Coordinator {
        Coordinator()
    }
    
    func makeNSView(context: Context) -> NSScrollView {
        let scrollView = NSScrollView()
        scrollView.hasVerticalScroller = true
        scrollView.borderType = .noBorder
        
        let textView = NSTextView()
        textView.isEditable = false
        textView.isSelectable = true
        textView.isContinuousSpellCheckingEnabled = false
        textView.isGrammarCheckingEnabled = false
        textView.font = .monospacedSystemFont(ofSize: 13, weight: .regular)
        textView.backgroundColor = .clear
        textView.textColor = .labelColor
        
        textView.isHorizontallyResizable = false
        textView.autoresizingMask = [.width]
        textView.textContainer?.widthTracksTextView = true
        textView.textContainer?.containerSize = NSSize(width: scrollView.contentSize.width, height: CGFloat.greatestFiniteMagnitude)
        
        scrollView.documentView = textView
        
        // Setup Coordinator to listen for frame changes
        context.coordinator.textView = textView
        textView.postsFrameChangedNotifications = true
        NotificationCenter.default.addObserver(context.coordinator,
                                             selector: #selector(Coordinator.updateLayout),
                                             name: NSView.frameDidChangeNotification,
                                             object: textView)
        
        return scrollView
    }
    
    func updateNSView(_ nsView: NSScrollView, context: Context) {
        guard let textView = nsView.documentView as? NSTextView else { return }
        
        let font = NSFont.monospacedSystemFont(ofSize: 13, weight: .regular)
        let storage = textView.textStorage ?? NSTextStorage()
        
        let attributed = NSMutableAttributedString(string: code)
        let fullRange = NSRange(location: 0, length: code.utf16.count)
        
        attributed.addAttribute(.font, value: font, range: fullRange)
        attributed.addAttribute(.foregroundColor, value: NSColor.labelColor, range: fullRange)
        
        // ... (Regex patterns code omitted for brevity, assume strictly same logic as before) ...
        let patterns: [(regex: String, color: NSColor)] = [
            ("\\b(is|during|between|begin|and|or|not)\\b", .systemPink),
            ("\\b(with|for|every|after|to|once|of)\\b", .systemOrange),
            ("\\b(information|warning|urgent warning|ask|listen for|send information|validate answer)\\b", .systemTeal),
            ("\\b(context|timeframe|before plan|after plan|valid values|meaning|calculation|reuse|documentation|question|assess|event progression|change of|begin of|end of)\\b", .systemIndigo),
            ("\\b(is a plan|is a drug|is an addressee|is a number|is a boolean|is a string)\\b", .systemPurple),
            ("\\b\\d+(\\.\\d+)?\\s*(°C|°F|mg|kg|g|lb|oz|ml|l|m|cm|mm|km|days|weeks|hours|minutes|seconds)?\\b", .systemYellow),
            ("<[^>]+>", .systemGreen),
            ("\"[^\"]*\"", .systemRed),
            ("\\(\\*.*?\\*\\)", .secondaryLabelColor),
        ]
        
        for (pattern, color) in patterns {
            guard let regex = try? NSRegularExpression(pattern: pattern, options: .caseInsensitive) else { continue }
            regex.enumerateMatches(in: code, options: [], range: fullRange) { match, _, _ in
                guard let matchRange = match?.range else { return }
                attributed.addAttribute(.foregroundColor, value: color, range: matchRange)
            }
        }
        
        let spaceWidth = " ".size(withAttributes: [.font: font]).width
        let wrapExtraIndent: CGFloat = spaceWidth * 2 
        
        var currentLine = 1
        
        let nsString = NSString(string: code)
        nsString.enumerateSubstrings(in: fullRange, options: .byLines) { (substring, substringRange, _, _) in
            guard let line = substring else { return }
            
            let leadingSpaces = line.prefix(while: { $0 == " " }).count
            let indentPoints = CGFloat(leadingSpaces) * spaceWidth
            
            let paragraphStyle = NSMutableParagraphStyle()
            paragraphStyle.firstLineHeadIndent = 0 
            paragraphStyle.headIndent = indentPoints + wrapExtraIndent 
            
            attributed.addAttribute(.paragraphStyle, value: paragraphStyle, range: substringRange)
            
            if let hl = highlightedLine, currentLine == hl {
                attributed.addAttribute(.backgroundColor, value: NSColor.systemYellow.withAlphaComponent(0.3), range: substringRange)
                DispatchQueue.main.async { textView.scrollRangeToVisible(substringRange) }
            }
            
            if errors.contains(where: { $0.line == currentLine }) {
                attributed.addAttribute(.backgroundColor, value: NSColor.red.withAlphaComponent(0.2), range: substringRange)
                attributed.addAttribute(.underlineStyle, value: NSUnderlineStyle.thick.rawValue, range: substringRange)
                attributed.addAttribute(.underlineColor, value: NSColor.red, range: substringRange)
            }
            
            currentLine += 1
        }
        
        if storage.string != code || highlightedLine != nil || !errors.isEmpty {
             storage.setAttributedString(attributed)
        }
        
        // Update coordinator state
        context.coordinator.errors = errors
        context.coordinator.engine = engine
        context.coordinator.simulationDays = simulationDays
        context.coordinator.updateLayout()
    }
    
    class Coordinator: NSObject {
        var errorViews: [NSView] = []
        weak var textView: NSTextView?
        var errors: [HippocratesParser.EngineError] = []
        var engine: HippocratesEngine?
        var simulationDays: Int = 30
        
        @objc func updateLayout() {
            guard let textView = textView, let layoutManager = textView.layoutManager, let textContainer = textView.textContainer else { return }
            
            // Remove old views
            errorViews.forEach { $0.removeFromSuperview() }
            errorViews.removeAll()
            
            // Parse periods
            let result = HippocratesParser.parse(input: textView.string)
            var periodsByLine: [Int: [HippocratesParser.PeriodDef]] = [:]
            
            if case .success(let plan) = result {
                for def in plan.definitions {
                     if let p = def.Period {
                         periodsByLine[p.line, default: []].append(p)
                     }
                }
            }
            
            let nsString = NSString(string: textView.string)
            let fullRange = NSRange(location: 0, length: textView.string.utf16.count)
            let errorsByLine = Dictionary(grouping: errors, by: { $0.line })
            
            let allLines = Set(errorsByLine.keys).union(periodsByLine.keys).sorted()
            
            for line in allLines {
                if line > 0 {
                    var lineCount = 1
                    var targetRange: NSRange?
                    nsString.enumerateSubstrings(in: fullRange, options: .byLines) { (_, rng, _, stop) in
                        if lineCount == line {
                            targetRange = rng
                            stop.pointee = true
                        }
                        lineCount += 1
                    }
                    
                    if let rng = targetRange {
                        layoutManager.ensureLayout(for: textContainer)
                        let glyphRange = layoutManager.glyphRange(forCharacterRange: rng, actualCharacterRange: nil)
                        let boundingRect = layoutManager.boundingRect(forGlyphRange: glyphRange, in: textContainer)
                        
                        let xPos = max(boundingRect.maxX + 10, 400) // Slightly wider indent
                        var currentX = xPos
                        let yPos = boundingRect.origin.y
                        
                        // 1. Errors
                        if let lineErrors = errorsByLine[line] {
                            let uniqueMessages = Array(Set(lineErrors.map { $0.message })).sorted()
                            let swiftUIView = ErrorPill(errors: uniqueMessages)
                            let hostingView = NSHostingView(rootView: swiftUIView)
                            
                            hostingView.frame = NSRect(x: currentX, y: yPos, width: 200, height: 26)
                            hostingView.translatesAutoresizingMaskIntoConstraints = true
                            let size = hostingView.fittingSize
                            hostingView.frame.size = size
                            
                            textView.addSubview(hostingView)
                            errorViews.append(hostingView)
                            
                            currentX += size.width + 8
                        }
                        
                        // 2. Periods
                        if let periods = periodsByLine[line] {
                             for p in periods {
                                 let swiftUIView = PeriodPill(period: p, engine: engine, simulationDays: simulationDays)
                                 let hostingView = NSHostingView(rootView: swiftUIView)
                                 
                                 hostingView.frame = NSRect(x: currentX, y: yPos, width: 200, height: 26)
                                 hostingView.translatesAutoresizingMaskIntoConstraints = true
                                 let size = hostingView.fittingSize
                                 hostingView.frame.size = size
                                 
                                 textView.addSubview(hostingView)
                                 errorViews.append(hostingView)
                                 
                                 currentX += size.width + 8
                             }
                        }
                    }
                }
            }
        }
    }
}

struct PeriodPill: View {
    let period: HippocratesParser.PeriodDef
    let engine: HippocratesEngine?
    let simulationDays: Int
    
    @State private var isPresented = false
    @State private var occurrences: [HippocratesEngine.PeriodOccurrence]? = nil
    
    var body: some View {
        Button(action: { 
            if engine != nil {
                loadOccurrences()
            }
            isPresented.toggle() 
        }) {
            HStack(spacing: 6) {
                Image(systemName: "clock.fill")
                    .foregroundStyle(.indigo)
                Text(period.name)
                    .font(.system(size: 11, weight: .medium))
                    .lineLimit(1)
            }
            .padding(.horizontal, 8)
            .padding(.vertical, 4)
            .background(Color(nsColor: .windowBackgroundColor))
            .cornerRadius(12)
            .overlay(
                RoundedRectangle(cornerRadius: 12)
                    .stroke(Color.indigo.opacity(0.3), lineWidth: 1)
            )
            .shadow(color: .black.opacity(0.1), radius: 1, x: 0, y: 1)
        }
        .buttonStyle(.plain)
        .popover(isPresented: $isPresented, arrowEdge: .bottom) {
            VStack(alignment: .leading) {
                if let _ = engine {
                     // Dynamic View
                     if let occ = occurrences {
                         Text("Occurrences (Next \(simulationDays) days)")
                            .font(.headline)
                            .padding(.bottom, 4)
                         
                         List(occ, id: \.start) { item in
                             VStack(alignment: .leading) {
                                 Text(item.start.formatted(date: .abbreviated, time: .shortened))
                                    .font(.callout)
                                 Text("to " + item.end.formatted(date: .omitted, time: .shortened))
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                             }
                         }
                         .frame(minWidth: 250, minHeight: 200)
                     } else {
                         ProgressView("Simulating...")
                     }
                } else {
                     // Static View
                     Text("Weekly Schedule")
                        .font(.headline)
                        .padding(.bottom, 4)
                     
                     // Visualize Timeframes
                     ForEach(Array(period.timeframes.enumerated()), id: \.offset) { _, group in
                         HStack(alignment: .top) {
                             Image(systemName: "calendar")
                             VStack(alignment: .leading) {
                                 ForEach(Array(group.enumerated()), id: \.offset) { _, selector in
                                     Text(selectorDescription(selector))
                                        .font(.caption)
                                 }
                             }
                         }
                         .padding(4)
                         .background(Color.gray.opacity(0.1))
                         .cornerRadius(4)
                     }
                     .frame(minWidth: 250)
                }
            }
            .padding()
        }
    }
    
    func loadOccurrences() {
        guard let engine = engine else { return }
        // Run in background if complex, but FFI is fast enough for small N usually
        // But UI should not block.
        DispatchQueue.global(qos: .userInitiated).async {
             let dates = engine.simulateOccurrences(periodName: period.name, days: simulationDays)
             DispatchQueue.main.async {
                 self.occurrences = dates
             }
        }
    }
    
    func selectorDescription(_ selector: HippocratesParser.RangeSelector) -> String {
        switch selector {
        case .Between(let start, let end):
            return "Between \(exprDesc(start)) and \(exprDesc(end))"
        case .Range(let start, let end):
            return "\(exprDesc(start)) ... \(exprDesc(end))"
        case .Equals(let val):
            return "At \(exprDesc(val))"
        case .List(let items):
            return "List: " + items.map(exprDesc).joined(separator: ", ")
        case .Unknown:
            return "Complex/Unknown Rule"
        }
    }
    
    func exprDesc(_ expr: HippocratesParser.Expression) -> String {
        switch expr {
        case .Literal(let lit):
            switch lit {
            case .TimeOfDay(let s): return s
            case .StringVal(let s): return s
            case .Number(let n): return "\(n)"
            case .Unknown: return "?"
            }
        case .Variable(let v): return "<\(v)>"
        case .Unknown: return "?"
        }
    }
}

// ... ErrorPill components ...
struct ErrorPill: View {
    let errors: [String]
    
    @State private var isPresented = false
    
    var body: some View {
        Button(action: { isPresented.toggle() }) {
            HStack(spacing: 6) {
                Image(systemName: "exclamationmark.triangle.fill")
                    .foregroundStyle(.yellow)
                if errors.count > 1 {
                    Text("\(errors.count) Errors")
                        .font(.system(size: 11, weight: .bold))
                } else {
                    Text(errors.first ?? "")
                        .font(.system(size: 11, weight: .medium))
                        .lineLimit(1)
                }
            }
            .padding(.horizontal, 8)
            .padding(.vertical, 4)
            .background(Color(nsColor: .windowBackgroundColor))
            .cornerRadius(12)
            .overlay(
                RoundedRectangle(cornerRadius: 12)
                    .stroke(Color.red.opacity(0.3), lineWidth: 1)
            )
            .shadow(color: .black.opacity(0.1), radius: 1, x: 0, y: 1)
        }
        .buttonStyle(.plain)
        .popover(isPresented: $isPresented, arrowEdge: .bottom) {
            ErrorDetailList(errors: errors)
                .frame(width: 300, height: 200)
        }
    }
}

struct ErrorDetailList: View {
    let errors: [String]
    
    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack {
                Text("Validation Errors")
                    .font(.headline)
                Spacer()
                Text("\(errors.count)")
                    .font(.caption)
                    .padding(4)
                    .background(Color.red.opacity(0.1), in: Circle())
            }
            .padding()
            .background(Color(nsColor: .controlBackgroundColor))
            
            ScrollView {
                VStack(alignment: .leading, spacing: 8) {
                    ForEach(Array(errors.enumerated()), id: \.offset) { _, error in
                        HStack(alignment: .top) {
                            Image(systemName: "exclamationmark.circle.fill")
                                .foregroundStyle(.red)
                                .font(.caption)
                                .padding(.top, 2)
                            Text(error)
                                .font(.callout)
                                .multilineTextAlignment(.leading)
                        }
                        .padding(8)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .background(Color(nsColor: .controlBackgroundColor).opacity(0.5), in: RoundedRectangle(cornerRadius: 6))
                    }
                }
                .padding()
            }
        }
    }
}
