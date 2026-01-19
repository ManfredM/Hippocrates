import SwiftUI
import AppKit

struct CodeVisualizerView: NSViewRepresentable {
    let code: String
    let highlightedLine: Int?
    
    func makeNSView(context: Context) -> NSScrollView {
        let scrollView = NSScrollView()
        scrollView.hasVerticalScroller = true
        scrollView.borderType = .noBorder
        
        // Setup text view
        let textView = NSTextView()
        textView.isEditable = false
        textView.isSelectable = true
        textView.font = .monospacedSystemFont(ofSize: 13, weight: .regular)
        textView.backgroundColor = .clear
        textView.textColor = .labelColor
        
        // Critical for wrapping
        textView.isHorizontallyResizable = false
        textView.autoresizingMask = [.width]
        textView.textContainer?.widthTracksTextView = true
        textView.textContainer?.containerSize = NSSize(width: scrollView.contentSize.width, height: CGFloat.greatestFiniteMagnitude)
        
        scrollView.documentView = textView
        return scrollView
    }
    
    func updateNSView(_ nsView: NSScrollView, context: Context) {
        guard let textView = nsView.documentView as? NSTextView else { return }
        
        let font = NSFont.monospacedSystemFont(ofSize: 13, weight: .regular)
        let boldFont = NSFont.monospacedSystemFont(ofSize: 13, weight: .bold)
        
        // Prepare storage
        let storage = textView.textStorage ?? NSTextStorage()
        // Always recreate the attributed string to ensure clean state
        
        // Create attributed string with syntax highlighting (Regex from previous implementation)
        let attributed = NSMutableAttributedString(string: code)
        attributed.addAttribute(.font, value: font, range: NSRange(location: 0, length: code.utf16.count))
        attributed.addAttribute(.foregroundColor, value: NSColor.labelColor, range: NSRange(location: 0, length: code.utf16.count))
        
        let nsString = NSString(string: code)
        let fullRange = NSRange(location: 0, length: nsString.length)
        
        let patterns: [(regex: String, color: NSColor)] = [
            // Keywords (Pink/Magenta in Xcode)
            ("\\b(is a plan|is a drug|is an addressee|context|timeframe|during plan|show message|ask|listen for|send information|assess|event progression|change of|begin of|end of|every|after|for|with|valid values|meaning|calculation|reuse|documentation|question)\\b", .systemPink),
            
            // Secondary Keywords / Operators
            ("\\b(is|during|between|begin|and|or|not)\\b", .systemPink),
            
            // Numbers & Units (Blue in Xcode)
            ("\\b\\d+(\\.\\d+)?\\s*(°C|°F|mg|kg|g|lb|oz|ml|l|m|cm|mm|km|days|weeks|hours|minutes|seconds)?\\b", .systemBlue),

            // Angled Variables (Teal/Cyan - Type-like)
            ("<[^>]+>", .systemTeal),
            
            // Strings (Red in Xcode)
            ("\"[^\"]*\"", .systemRed),
            
            // Comments (Gray)
            ("\\(\\*.*?\\*\\)", .secondaryLabelColor),
            
            // Identifiers being defined (Purple-ish for types/definitions?)
            // Just basic support for now.
        ]
        
        for (pattern, color) in patterns {
            guard let regex = try? NSRegularExpression(pattern: pattern, options: .caseInsensitive) else { continue }
            regex.enumerateMatches(in: code, options: [], range: fullRange) { match, _, _ in
                guard let matchRange = match?.range else { return }
                attributed.addAttribute(.foregroundColor, value: color, range: matchRange)
                attributed.addAttribute(.font, value: boldFont, range: matchRange)
            }
        }
        
        // --- SMART LINE WRAP LOGIC ---
        // Iterate through each line to apply paragraph style
        let spaceWidth = " ".size(withAttributes: [.font: font]).width
        let wrapExtraIndent: CGFloat = spaceWidth * 2 
        
        var currentLine = 1
        
        nsString.enumerateSubstrings(in: fullRange, options: .byLines) { (substring, substringRange, _, _) in
            guard let line = substring else { return }
            
            // Calculate indentation level (count leading spaces)
            let leadingSpaces = line.prefix(while: { $0 == " " }).count
            let indentPoints = CGFloat(leadingSpaces) * spaceWidth
            
            let paragraphStyle = NSMutableParagraphStyle()
            paragraphStyle.firstLineHeadIndent = 0 
            paragraphStyle.headIndent = indentPoints + wrapExtraIndent 
            
            attributed.addAttribute(.paragraphStyle, value: paragraphStyle, range: substringRange)
            
            // Execution Highlighting
            if let hl = highlightedLine, currentLine == hl {
                attributed.addAttribute(.backgroundColor, value: NSColor.systemYellow.withAlphaComponent(0.3), range: substringRange)
                
                // Auto-scroll to the highlighted line
                DispatchQueue.main.async {
                    textView.scrollRangeToVisible(substringRange)
                }
            }
            currentLine += 1
        }
        
        // Update storage
        if storage.string != code || highlightedLine != nil {
            storage.setAttributedString(attributed)
        }
    }
}
