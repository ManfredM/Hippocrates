import SwiftUI
import AppKit

struct CodeVisualizerView: NSViewRepresentable {
    let code: String
    let highlightedLine: Int?
    let errors: [HippocratesParser.EngineError]
    
    init(code: String, highlightedLine: Int?, errors: [HippocratesParser.EngineError] = []) {
        self.code = code
        self.highlightedLine = highlightedLine
        self.errors = errors
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
        return scrollView
    }
    
    func updateNSView(_ nsView: NSScrollView, context: Context) {
        guard let textView = nsView.documentView as? NSTextView else { return }
        
        // Cleanup old views
        context.coordinator.errorViews.forEach { $0.removeFromSuperview() }
        context.coordinator.errorViews.removeAll()
        
        let font = NSFont.monospacedSystemFont(ofSize: 13, weight: .regular)
        let storage = textView.textStorage ?? NSTextStorage()
        
        let attributed = NSMutableAttributedString(string: code)
        let fullRange = NSRange(location: 0, length: code.utf16.count)
        
        attributed.addAttribute(.font, value: font, range: fullRange)
        attributed.addAttribute(.foregroundColor, value: NSColor.labelColor, range: fullRange)
        
        let patterns: [(regex: String, color: NSColor)] = [
            ("\\b(is|during|between|begin|and|or|not)\\b", .systemPink),
            ("\\b(with|for|every|after|to|once|of)\\b", .systemOrange),
            ("\\b(show message|ask|listen for|send information|validate answer)\\b", .systemTeal),
            ("\\b(context|timeframe|during plan|valid values|meaning|calculation|reuse|documentation|question|assess|event progression|change of|begin of|end of)\\b", .systemIndigo),
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
        
        // Multiple Errors Overlay
        let errorsByLine = Dictionary(grouping: errors, by: { $0.line })
        
        for (line, lineErrors) in errorsByLine {
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
                 
                if let rng = targetRange, let layoutManager = textView.layoutManager, let textContainer = textView.textContainer {
                    layoutManager.ensureLayout(for: textContainer)
                    
                    let glyphRange = layoutManager.glyphRange(forCharacterRange: rng, actualCharacterRange: nil)
                    let boundingRect = layoutManager.boundingRect(forGlyphRange: glyphRange, in: textContainer)
                    
                    let uniqueMessages = Array(Set(lineErrors.map { $0.message })).sorted()
                    
                    let swiftUIView = ErrorPill(errors: uniqueMessages)
                    let hostingView = NSHostingView(rootView: swiftUIView)
                    
                    let xPos = max(boundingRect.maxX + 10, 300) 
                    let yPos = boundingRect.origin.y
                    
                    hostingView.frame = NSRect(x: xPos, y: yPos, width: 200, height: 26)
                    hostingView.translatesAutoresizingMaskIntoConstraints = true 
                    
                    let size = hostingView.fittingSize
                    hostingView.frame.size = size
                    
                    textView.addSubview(hostingView)
                    context.coordinator.errorViews.append(hostingView)
                }
            }
        }
    }
    
    // Updated Coordinator to hold generic NSView
    class Coordinator {
        var errorViews: [NSView] = []
    }
}

// SwiftUI Components to replicate Timeline style
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
