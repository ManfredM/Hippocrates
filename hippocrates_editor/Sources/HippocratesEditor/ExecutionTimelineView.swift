import SwiftUI
import Charts

struct ExecutionTimelineView: View {
    @EnvironmentObject var appState: AppState
    
    var groupedEvents: [(Date, [ExecutionEvent])] {
        let grouped = Dictionary(grouping: appState.executionLogs) { event in
            Calendar.current.startOfDay(for: event.time)
        }
        return grouped.sorted { $0.key < $1.key }
    }
    
    var body: some View {
        ScrollViewReader { proxy in
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 20, pinnedViews: [.sectionHeaders]) {
                    if appState.executionLogs.isEmpty {
                        ContentUnavailableView("No Execution Data", systemImage: "clock.arrow.circlepath", description: Text("Run a plan to see the timeline here."))
                            .padding(.top, 50)
                    } else {
                        ForEach(groupedEvents, id: \.0) { (date, events) in
                            Section(header: DayHeader(date: date)) {
                                ForEach(Array(events.enumerated()), id: \.element.id) { index, event in
                                    TimelineRow(event: event, isLast: index == events.count - 1)
                                        .id(event.id) // Identify row for potential specific scrolling
                                }
                            }
                        }
                        
                        // Invisible footer to scroll to
                        Color.clear
                            .frame(height: 1)
                            .id("bottom")
                    }
                }
                .padding()
            }
            .background(Color(nsColor: .textBackgroundColor))
            .onChange(of: appState.executionLogs.count) { _, _ in
                // Auto-scroll to bottom when new logs arrive
                if !appState.executionLogs.isEmpty {
                    withAnimation {
                        proxy.scrollTo("bottom", anchor: .bottom)
                    }
                }
            }
        }
    }
}

struct DayHeader: View {
    let date: Date
    
    var body: some View {
        HStack {
            Text(date, format: .dateTime.weekday().day().month().year())
                .font(.headline)
                .padding(.vertical, 8)
                .padding(.horizontal, 12)
                .background(.ultraThinMaterial, in: Capsule())
            Spacer()
        }
        .padding(.vertical, 4)
    }
}

struct TimelineRow: View {
    let event: ExecutionEvent
    let isLast: Bool
    
    var body: some View {
        HStack(alignment: .top, spacing: 12) {
            // Timestamp
            Text(event.time, format: .dateTime.hour().minute().second())
                .font(.caption.monospaced())
                .foregroundStyle(.secondary)
                .frame(width: 65, alignment: .trailing)
                .padding(.top, 2)
            
            // Visual Line & Dot
            VStack(spacing: 0) {
                Circle()
                    .fill(Color.accentColor)
                    .frame(width: 10, height: 10)
                    .background(
                        Circle()
                            .stroke(Color(nsColor: .windowBackgroundColor), lineWidth: 2)
                    )
                
                if !isLast {
                    Rectangle()
                        .fill(Color.secondary.opacity(0.3))
                        .frame(width: 2)
                        .frame(maxWidth: .infinity, maxHeight: .infinity) // Fill vertical space to next row
                        .padding(.top, -2) // Overlap slightly to connect
                }
            }
            .frame(width: 14)
            
            // Message Content
            VStack(alignment: .leading, spacing: 4) {
                Text(event.name)
                    .font(.body)
                    .fixedSize(horizontal: false, vertical: true) // Allow wrapping
                
                if !event.category.isEmpty && event.category != "Log" {
                    Text(event.category)
                        .font(.caption2)
                        .padding(.horizontal, 6)
                        .padding(.vertical, 2)
                        .background(Color.secondary.opacity(0.1), in: Capsule())
                }
            }
            .padding(.bottom, 16) // Spacing between events
            
            Spacer()
        }
    }
}
