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

struct PeriodBand: Identifiable {
    let id = UUID()
    let name: String
    let start: Date
    let end: Date
}

// MARK: - Views

struct ExecutionTimelineView: View {
    @EnvironmentObject var appState: AppState
    
    // Zoom State
    @State private var hourHeight: CGFloat = defaultHourHeight
    @State private var lastHourHeight: CGFloat = defaultHourHeight
    
    @State private var periodBands: [Date: [PeriodBand]] = [:]
    
    // Days to show: Union of days with events AND days with bands
    var visibleDates: [Date] {
        let eventDates = Set(appState.executionLogs.map { Calendar.current.startOfDay(for: $0.time) })
        let bandDates = Set(periodBands.keys)
        let sorted = eventDates.union(bandDates).sorted()
        
        if appState.executionLogs.isEmpty && sorted.count > 7 {
            return Array(sorted.prefix(7))
        }
        return sorted
    }
    
    func clusters(for date: Date) -> [TimelineCluster] {
        let dayStart = Calendar.current.startOfDay(for: date)
        let nextDay = Calendar.current.date(byAdding: .day, value: 1, to: dayStart)!
        
        let events = appState.executionLogs.filter {
            $0.time >= dayStart && $0.time < nextDay
        }.sorted { $0.time < $1.time }
        
        return cluster(events: events)
    }
    
    func bands(for date: Date) -> [PeriodBand] {
        periodBands[date] ?? []
    }
    
    func refreshData() {
        guard let engine = appState.visualizationEngine ?? appState.currentEngine else { 
            periodBands = [:]
            return 
        }
        
        let periods = engine.getPeriods()
        guard !periods.isEmpty else { 
            periodBands = [:]
            return 
        }
        
        // Determine window
        // If static (no logs), use Generic Week
        let calendar = Calendar.current
        var anchorDate: Date
        var days: Int
        
        if appState.executionLogs.isEmpty {
            // Static Generic Week
            let today = Date()
            var components = calendar.dateComponents([.yearForWeekOfYear, .weekOfYear], from: today)
            components.weekday = calendar.firstWeekday
            anchorDate = calendar.date(from: components) ?? today
            days = 7
        } else {
            // Dynamic: Try to cover the logs range
            // We'll generate bands for the range of logs + buffer
            if let first = appState.executionLogs.min(by: { $0.time < $1.time })?.time,
               let last = appState.executionLogs.max(by: { $0.time < $1.time })?.time {
                anchorDate = calendar.startOfDay(for: first)
                let diff = calendar.dateComponents([.day], from: anchorDate, to: last).day ?? 0
                days = diff + 2 // Cover last day fully
            } else {
                // Fallback
                anchorDate = Date()
                days = 1
            }
        }
        
        var newBands: [Date: [PeriodBand]] = [:]
        
        for period in periods {
            let occs = engine.simulateOccurrences(periodName: period.name, days: days, startDate: anchorDate)
            for occ in occs {
                let band = PeriodBand(name: period.name, start: occ.start, end: occ.end)
                // Key by day
                let day = calendar.startOfDay(for: occ.start)
                newBands[day, default: []].append(band)
            }
        }
        
        periodBands = newBands
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
                        
                    ForEach(visibleDates, id: \.self) { date in
                        DayHeader(date: date, width: dayWidth, isStatic: appState.executionLogs.isEmpty)
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
                            ForEach(visibleDates, id: \.self) { date in
                                DayTimeline(
                                    clusters: clusters(for: date),
                                    bands: bands(for: date),
                                    hourHeight: hourHeight,
                                    dayWidth: dayWidth
                                )
                            }
                        }
                    }
                }
                .gesture(zoomGesture) // Apply gesture to the content area
            }
        }
        .background(Color(nsColor: .textBackgroundColor))
        .onAppear {
             refreshData()
        }
        .onChange(of: appState.executionLogs.count) { 
             refreshData()
        }
        .onChange(of: appState.visualizationEngine) { 
             refreshData()
        }
        .onChange(of: appState.parseStatus) { 
             if appState.parseStatus == "Ready" || appState.parseStatus.hasPrefix("Valid") {
                 refreshData()
             }
        }
    }
}

struct DayHeader: View {
    let date: Date
    let width: CGFloat
    let isStatic: Bool
    
    var body: some View {
        Text(date, format: isStatic ? .dateTime.weekday(.wide) : .dateTime.weekday().day().month())
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
            Color.clear.frame(height: 50) 
            
            ForEach(0..<24) { hour in
                Text(String(format: "%02d:00", hour))
                    .font(.caption2.monospaced())
                    .foregroundStyle(.secondary)
                    .frame(height: hourHeight, alignment: .top)
                    .offset(y: -5)
            }
        }
        .padding(.trailing, 4)
        .border(width: 1, edges: [.trailing], color: Color(nsColor: .separatorColor))
        .frame(height: hourHeight * 24)
    }
}

struct DayTimeline: View {
    let clusters: [TimelineCluster]
    let bands: [PeriodBand]
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
            
            // Bands Layer
            ForEach(bands) { band in
                PeriodBandView(band: band, width: dayWidth, hourHeight: hourHeight)
            }

            // Events Layer
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

struct PeriodBandView: View {
    let band: PeriodBand
    let width: CGFloat
    let hourHeight: CGFloat
    
    var body: some View {
        let startY = yOffset(for: band.start)
        let endY = yOffset(for: band.end)
        let height = max(endY - startY, 2) // Min height visibility
        
        ZStack(alignment: .topLeading) {
            Rectangle()
                .fill(Color.indigo.opacity(0.15))
            
            // Minimal Label inside the band
            Text(band.name)
                .font(.caption)
                .foregroundStyle(Color.indigo)
                .padding(4)
        }
        .frame(width: width - 2, height: height)
        .position(x: width / 2, y: startY + height / 2)
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
    let hourHeight: CGFloat
    
    @State private var isPresented = false
    
    var body: some View {
        HStack(alignment: .top, spacing: 0) {
            // Dot on the timeline (left edge)
            Circle()
                .fill(Color.accentColor)
                .frame(width: 8, height: 8)
                .background(Circle().stroke(Color(nsColor: .windowBackgroundColor), lineWidth: 2))
                .offset(y: 12) 
            
            // Connecting Line
            Rectangle()
                .fill(Color.accentColor.opacity(0.5))
                .frame(width: 20, height: 2)
                .offset(y: 15) 
            
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
        .padding(.leading, -4) 
        .frame(width: width - 8) 
        .contentShape(Rectangle()) 
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
