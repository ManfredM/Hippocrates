import SwiftUI

// Config
private let defaultHourHeight: CGFloat = 60
private let dayWidth: CGFloat = 350
private let timeColWidth: CGFloat = 60

// MARK: - Models

struct TimelineCluster: Identifiable {
    let id = UUID()
    let events: [ExecutionEvent]
    
    var time: Date {
        events.first?.time ?? Date()
    }
    
    var isGroup: Bool {
        events.count > 1
    }
}

// MARK: - Views

struct ExecutionTimelineView: View {
    @EnvironmentObject var appState: AppState
    
    // Zoom State
    @State private var hourHeight: CGFloat = defaultHourHeight
    @State private var lastHourHeight: CGFloat = defaultHourHeight
    
    var groupedEvents: [(Date, [TimelineCluster])] {
        let grouped = Dictionary(grouping: appState.executionLogs) { event in
            Calendar.current.startOfDay(for: event.time)
        }
        
        let sortedKeys = grouped.keys.sorted()
        return sortedKeys.map { date in
            let events = grouped[date]!.sorted { $0.time < $1.time }
            return (date, cluster(events: events))
        }
    }
    
    func cluster(events: [ExecutionEvent]) -> [TimelineCluster] {
        var clusters: [TimelineCluster] = []
        var currentBatch: [ExecutionEvent] = []
        
        for event in events {
            if let last = currentBatch.last {
                if event.time.timeIntervalSince(last.time) < 15 * 60 {
                    currentBatch.append(event)
                } else {
                    clusters.append(TimelineCluster(events: currentBatch))
                    currentBatch = [event]
                }
            } else {
                currentBatch.append(event)
            }
        }
        if !currentBatch.isEmpty {
            clusters.append(TimelineCluster(events: currentBatch))
        }
        return clusters
    }
    
    // Magnification Gesture
    var zoomGesture: some Gesture {
        MagnificationGesture()
            .onChanged { val in
                let newHeight = lastHourHeight * val
                // Constraint zoom: Min 30px/hr, Max 200px/hr
                hourHeight = min(max(newHeight, 30), 200)
            }
            .onEnded { _ in
                lastHourHeight = hourHeight
            }
    }
    
    var body: some View {
        ScrollView(.horizontal) {
            VStack(alignment: .leading, spacing: 0) {
                // Sticky Header Row (Outside Vertical Scroll)
                HStack(spacing: 1) {
                    // Placeholder for Time Column (Corner)
                    Color(nsColor: .controlBackgroundColor)
                        .frame(width: timeColWidth, height: 50)
                        .border(Color(nsColor: .separatorColor))
                        
                    ForEach(groupedEvents, id: \.0) { (date, _) in
                        DayHeader(date: date, width: dayWidth)
                    }
                }
                .background(Color(nsColor: .controlBackgroundColor))
                
                // Vertically Scrollable Content
                ScrollView(.vertical) {
                    HStack(alignment: .top, spacing: 1) {
                        // Time Labels
                        TimeLabelsColumn(hourHeight: hourHeight)
                            .frame(width: timeColWidth)
                            .background(Color(nsColor: .windowBackgroundColor))
                        
                        // Timelines
                        LazyHStack(alignment: .top, spacing: 1) {
                            ForEach(groupedEvents, id: \.0) { (date, clusters) in
                                DayTimeline(clusters: clusters, hourHeight: hourHeight, dayWidth: dayWidth)
                            }
                        }
                    }
                }
                .gesture(zoomGesture) // Apply gesture to the content area
            }
        }
        .background(Color(nsColor: .textBackgroundColor))
    }
}

struct DayHeader: View {
    let date: Date
    let width: CGFloat
    
    var body: some View {
        Text(date, format: .dateTime.weekday().day().month())
            .font(.headline)
            .frame(width: width, height: 50)
            .background(Color(nsColor: .controlBackgroundColor))
            .border(Color(nsColor: .separatorColor))
    }
}

struct TimeLabelsColumn: View {
    let hourHeight: CGFloat
    
    var body: some View {
        VStack(spacing: 0) {
            Color.clear.frame(height: 50) // Spacer if inside common scroll, but here distinct
            // Actually, in the nested scroll view approach, the TimeLabelsColumn is INSIDE the vertical scroll.
            // So it starts at 00:00 immediately.
            
            ForEach(0..<24) { hour in
                Text(String(format: "%02d:00", hour))
                    .font(.caption2.monospaced())
                    .foregroundStyle(.secondary)
                    .frame(height: hourHeight, alignment: .top)
                    .offset(y: -5) // Center text on the line
            }
        }
        .padding(.trailing, 4)
        .border(width: 1, edges: [.trailing], color: Color(nsColor: .separatorColor))
        .frame(height: hourHeight * 24)
    }
}

struct DayTimeline: View {
    let clusters: [TimelineCluster]
    let hourHeight: CGFloat
    let dayWidth: CGFloat
    
    var body: some View {
        ZStack(alignment: .top) {
            // Background Grid
            VStack(spacing: 0) {
                ForEach(0..<24) { _ in
                    Divider()
                        .frame(height: hourHeight, alignment: .top)
                }
            }
            
            // Clusters
            ForEach(clusters) { cluster in
                ClusterView(cluster: cluster, width: dayWidth, hourHeight: hourHeight)
                    .offset(y: yOffset(for: cluster.time))
            }
        }
        .frame(width: dayWidth, height: hourHeight * 24)
        .background(Color.white.opacity(0.02))
        .border(width: 1, edges: [.trailing], color: Color(nsColor: .separatorColor).opacity(0.5))
        .clipped()
    }
    
    func yOffset(for time: Date) -> CGFloat {
        let cal = Calendar.current
        let hour = cal.component(.hour, from: time)
        let minute = cal.component(.minute, from: time)
        let totalMinutes = hour * 60 + minute
        return CGFloat(totalMinutes) / 60.0 * hourHeight
    }
}

struct ClusterView: View {
    let cluster: TimelineCluster
    let width: CGFloat
    let hourHeight: CGFloat // Passed down if needed for scaling inner elements? Not strictly needed for logic, but good for context.
    
    @State private var isPresented = false
    
    var body: some View {
        HStack(alignment: .top, spacing: 0) {
            // Dot on the timeline (left edge)
            Circle()
                .fill(Color.accentColor)
                .frame(width: 8, height: 8)
                .background(Circle().stroke(Color(nsColor: .windowBackgroundColor), lineWidth: 2))
                .offset(y: 12) // Align roughly with pill center or top. Constant offset might feel off if pill shrinks?
                // Visual consistency: The dot marks the TIME.
                // The pill is visually connected.
            
            // Connecting Line
            Rectangle()
                .fill(Color.accentColor.opacity(0.5))
                .frame(width: 20, height: 2)
                .offset(y: 15) // Align with dot center
            
            // Pill
            Group {
                if cluster.isGroup {
                    GroupPill(cluster: cluster)
                } else if let event = cluster.events.first {
                    EventPillContent(event: event)
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(.leading, -4) // Offset dot to sit on the line
        .frame(width: width - 8) // Padding
        .contentShape(Rectangle()) // Make interaction area solid
        .onTapGesture {
            if cluster.isGroup {
                isPresented = true
            }
        }
        .popover(isPresented: $isPresented, arrowEdge: .trailing) {
            ClusterDetailView(cluster: cluster)
                .frame(width: 400, height: 300)
        }
    }
}

struct GroupPill: View {
    let cluster: TimelineCluster
    
    var body: some View {
        HStack(spacing: 8) {
            Image(systemName: "square.stack.3d.up.fill")
            VStack(alignment: .leading) {
                Text("\(cluster.events.count) messages")
                    .font(.headline)
                if let first = cluster.events.first {
                    Text(first.time, format: .dateTime.hour().minute())
                        .font(.caption)
                }
            }
            Spacer()
        }
        .padding(10)
        .background(Color.accentColor.opacity(0.2), in: RoundedRectangle(cornerRadius: 12))
        .overlay(
            RoundedRectangle(cornerRadius: 12)
                .stroke(Color.accentColor.opacity(0.5), lineWidth: 1)
        )
    }
}

struct EventPillContent: View {
    let event: ExecutionEvent
    
    var isQuestion: Bool {
        event.type == 2
    }
    
    var isAnswer: Bool {
        event.type == 3
    }
    
    var pillColor: Color {
        if isQuestion { return .orange }
        if isAnswer { return .green }
        return .blue
    }
    
    var body: some View {
        HStack(alignment: .top, spacing: 6) {
            Text(event.time, format: .dateTime.hour().minute())
                .font(.caption2.monospaced())
                .foregroundStyle(.white.opacity(0.9))
                .padding(.top, 2)
            
            VStack(alignment: .leading, spacing: 2) {
                Text(event.name)
                    .font(.callout)
                    .foregroundStyle(.white)
            }
            Spacer()
        }
        .padding(8)
        .background(pillColor.gradient, in: RoundedRectangle(cornerRadius: 8))
        .shadow(color: .black.opacity(0.1), radius: 2, x: 0, y: 1)
    }
}

struct ClusterDetailView: View {
    let cluster: TimelineCluster
    
    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            Text("Events at \(cluster.time, format: .dateTime.hour().minute())")
                .font(.headline)
                .padding()
                .frame(maxWidth: .infinity, alignment: .leading)
                .background(Color(nsColor: .controlBackgroundColor))
            
            ScrollView {
                VStack(spacing: 8) {
                    ForEach(cluster.events) { event in
                        EventPillContent(event: event)
                    }
                }
                .padding()
            }
        }
    }
}

// Helpers
extension View {
    func border(width: CGFloat, edges: [Edge], color: Color) -> some View {
        overlay(edgeBorder(width: width, edges: edges).foregroundColor(color))
    }
}

struct EdgeBorder: Shape {
    var width: CGFloat
    var edges: [Edge]
    
    func path(in rect: CGRect) -> Path {
        var path = Path()
        for edge in edges {
            var x: CGFloat {
                switch edge {
                case .top, .bottom, .leading: return rect.minX
                case .trailing: return rect.maxX - width
                }
            }
            var y: CGFloat {
                switch edge {
                case .top, .leading, .trailing: return rect.minY
                case .bottom: return rect.maxY - width
                }
            }
            var w: CGFloat {
                switch edge {
                case .top, .bottom: return rect.width
                case .leading, .trailing: return width
                }
            }
            var h: CGFloat {
                switch edge {
                case .top, .bottom: return width
                case .leading, .trailing: return rect.height
                }
            }
            path.addRect(CGRect(x: x, y: y, width: w, height: h))
        }
        return path
    }
}

extension View {
    func edgeBorder(width: CGFloat, edges: [Edge]) -> some Shape {
        EdgeBorder(width: width, edges: edges)
    }
}
