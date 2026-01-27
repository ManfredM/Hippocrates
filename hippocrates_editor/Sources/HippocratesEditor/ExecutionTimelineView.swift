import SwiftUI
#if os(macOS)
import AppKit
#endif

extension Calendar {
    static let gmt: Calendar = {
        var cal = Calendar(identifier: .gregorian)
        cal.timeZone = TimeZone(secondsFromGMT: 0)!
        return cal
    }()
}

// Config
private let defaultHourHeight: CGFloat = 60
private let dayWidth: CGFloat = 210
private let timeColWidth: CGFloat = 120
private let valueRowLineHeight: CGFloat = 16
private let valueRowPadding: CGFloat = 6
private let valueSectionHeaderHeight: CGFloat = 24
private let dividerHeight: CGFloat = 1
private let headerRowHeight: CGFloat = 50
private let minTimelineViewportHeight: CGFloat = 140
private let minValuesRowHeight: CGFloat = valueSectionHeaderHeight + valueRowLineHeight + valueRowPadding * 2 + dividerHeight * 2
private let headerRowColor = Color(nsColor: .systemYellow).opacity(0.18)
private let timelineRowColor = Color(nsColor: .textBackgroundColor)
private let valuesRowColor = Color(nsColor: .systemGreen).opacity(0.12)
private let timeLabelColor = Color(nsColor: .systemYellow).opacity(0.85)

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

struct ValueRow: Identifiable {
    let id: String
    let display: String
    let entriesByDay: [Date: [EngineValueEntry]]
    let maxEntries: Int
}

private func valueRowHeight(for row: ValueRow) -> CGFloat {
    let lines = max(row.maxEntries, 1)
    return CGFloat(lines) * valueRowLineHeight + valueRowPadding * 2
}

private func valuesSectionHeight(for rows: [ValueRow]) -> CGFloat {
    let rowHeights = rows.reduce(0) { $0 + valueRowHeight(for: $1) }
    let dividerCount = rows.count + 1
    return valueSectionHeaderHeight + rowHeights + CGFloat(dividerCount) * dividerHeight
}

final class HorizontalScrollSyncState: ObservableObject {
    @Published var offset: CGFloat = 0
}

// MARK: - Views

struct ExecutionTimelineView: View {
    @EnvironmentObject var appState: AppState
    
    // Zoom State
    @State private var hourHeight: CGFloat = defaultHourHeight
    @State private var lastHourHeight: CGFloat = defaultHourHeight
    @StateObject private var dayScrollSync = HorizontalScrollSyncState()
    
    @State private var periodBands: [Date: [PeriodBand]] = [:]
    @State private var valueRows: [ValueRow] = []
    
    // Days to show: Union of days with events AND days with bands
    var visibleDates: [Date] {
        let eventDates = Set(appState.executionLogs.map { Calendar.gmt.startOfDay(for: $0.time) })
        let bandDates = Set(periodBands.keys)
        let valueDates = Set(valueRows.flatMap { $0.entriesByDay.keys })
        let sorted = eventDates.union(bandDates).union(valueDates).sorted()
        
        if appState.executionLogs.isEmpty && sorted.count > 7 {
            return Array(sorted.prefix(7))
        }
        return sorted
    }
    
    func clusters(for date: Date) -> [TimelineCluster] {
        let dayStart = Calendar.gmt.startOfDay(for: date)
        let nextDay = Calendar.gmt.date(byAdding: .day, value: 1, to: dayStart)!
        
        let events = appState.executionLogs.filter {
            $0.time >= dayStart && $0.time < nextDay
        }.sorted { $0.time < $1.time }
        
        return cluster(events: events)
    }
    
    func bands(for date: Date) -> [PeriodBand] {
        periodBands[date] ?? []
    }
    
    func refreshData() {
        guard let engine = appState.currentEngine ?? appState.visualizationEngine else { 
            periodBands = [:]
            valueRows = []
            return 
        }
        
        let periods = engine.getPeriods()
        
        // Determine window
        // If static (no logs), use Generic Week
        let calendar = Calendar.gmt
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
        
        let endDate = calendar.date(byAdding: .day, value: days, to: anchorDate) ?? anchorDate
        let values = engine.getValues(start: anchorDate, end: endDate)
        var groupedValues: [Date: [String: [EngineValueEntry]]] = [:]
        
        for entry in values {
            let day = calendar.startOfDay(for: entry.date)
            groupedValues[day, default: [:]][entry.variable, default: []].append(entry)
        }
        
        struct ValueRowBuilder {
            var display: String
            var entriesByDay: [Date: [EngineValueEntry]]
            var maxEntries: Int
        }
        
        var rowsByVariable: [String: ValueRowBuilder] = [:]
        for (day, variables) in groupedValues {
            for (variable, entries) in variables {
                let sortedEntries = entries.sorted { $0.timestamp < $1.timestamp }
                let display = sortedEntries.first?.display ?? variable
                var row = rowsByVariable[variable] ?? ValueRowBuilder(display: display, entriesByDay: [:], maxEntries: 0)
                if row.display.isEmpty {
                    row.display = display
                }
                row.entriesByDay[day] = sortedEntries
                row.maxEntries = max(row.maxEntries, sortedEntries.count)
                rowsByVariable[variable] = row
            }
        }
        
        var rows: [ValueRow] = []
        for (variable, row) in rowsByVariable {
            let display = row.display.isEmpty ? variable : row.display
            rows.append(ValueRow(id: variable, display: display, entriesByDay: row.entriesByDay, maxEntries: row.maxEntries))
        }
        rows.sort { $0.display.localizedCaseInsensitiveCompare($1.display) == .orderedAscending }
        
        valueRows = rows
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
        let displayRows = valueRows.isEmpty
            ? [ValueRow(id: "no-values", display: "No values", entriesByDay: [:], maxEntries: 1)]
            : valueRows
        
        GeometryReader { geo in
            let dayViewportWidth = max(0, geo.size.width - timeColWidth - 1)
            let timelineHeight = hourHeight * 24
            let valuesContentHeight = valuesSectionHeight(for: displayRows)
            
            VStack(alignment: .leading, spacing: 0) {
                // Top Row: Days
                HStack(spacing: 1) {
                    Color.clear
                        .frame(width: timeColWidth, height: headerRowHeight)
                    
                    SyncedHorizontalScrollView(offset: $dayScrollSync.offset, showsIndicators: false) {
                        HStack(spacing: 1) {
                            ForEach(visibleDates, id: \.self) { date in
                                DayHeader(
                                    date: date,
                                    width: dayWidth,
                                    isStatic: appState.executionLogs.isEmpty,
                                    backgroundColor: headerRowColor
                                )
                            }
                        }
                    }
                    .frame(width: dayViewportWidth, height: headerRowHeight, alignment: .leading)
                }
                .frame(height: headerRowHeight)
                .background(Color.clear)
                
                Divider()
                
                VSplitView {
                    // Middle Row: Timeline
                    ScrollView(.vertical) {
                        HStack(alignment: .top, spacing: 1) {
                            TimeLabelsColumn(hourHeight: hourHeight, labelColor: timeLabelColor)
                                .frame(width: timeColWidth)
                            
                            SyncedHorizontalScrollView(offset: $dayScrollSync.offset) {
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
                                .frame(height: timelineHeight, alignment: .top)
                            }
                            .frame(width: dayViewportWidth, height: timelineHeight, alignment: .topLeading)
                        }
                        .frame(height: timelineHeight, alignment: .top)
                    }
                    .frame(minHeight: minTimelineViewportHeight)
                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
                    .background(timelineRowColor)
                    .gesture(zoomGesture)
                    
                    // Bottom Row: Values
                    let valuesContent = HStack(alignment: .top, spacing: 1) {
                        ValuesLabelColumn(rows: displayRows, width: timeColWidth)
                        
                        SyncedHorizontalScrollView(offset: $dayScrollSync.offset) {
                            ValuesDayColumnsView(rows: displayRows, dates: visibleDates, dayWidth: dayWidth)
                        }
                        .frame(width: dayViewportWidth, height: valuesContentHeight, alignment: .topLeading)
                    }
                    
                    ScrollView(.vertical) {
                        valuesContent
                            .frame(height: valuesContentHeight, alignment: .topLeading)
                    }
                    .frame(minHeight: minValuesRowHeight)
                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
                    .background(valuesRowColor)
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
            .environment(\.timeZone, TimeZone(secondsFromGMT: 0)!)
        }
    }
}

struct DayHeader: View {
    let date: Date
    let width: CGFloat
    let isStatic: Bool
    let backgroundColor: Color
    
    var body: some View {
        Text(date, format: (isStatic ? .dateTime.weekday(.wide) : .dateTime.weekday().day().month()))
            .font(.headline)
            .frame(width: width, height: headerRowHeight)
            .background(backgroundColor)
            .border(Color(nsColor: .separatorColor))
    }
}

struct TimeLabelsColumn: View {
    let hourHeight: CGFloat
    let labelColor: Color
    
    var body: some View {
        VStack(spacing: 0) {
            ForEach(0..<24) { hour in
                Text(String(format: "%02d:00", hour))
                    .font(.caption2.monospaced())
                    .foregroundStyle(labelColor)
                    .frame(height: hourHeight, alignment: .topTrailing)
                    .offset(y: hour == 0 ? 0 : -5)
            }
        }
        .padding(.trailing, 6)
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
        .clipped()
        .border(width: 1, edges: [.trailing], color: Color(nsColor: .separatorColor).opacity(0.5))
    }
    
    func yOffset(for time: Date) -> CGFloat {
        let cal = Calendar.gmt
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
        let cal = Calendar.gmt
        let hour = cal.component(.hour, from: time)
        let minute = cal.component(.minute, from: time)
        let totalMinutes = hour * 60 + minute
        return CGFloat(totalMinutes) / 60.0 * hourHeight
    }
}

struct ValuesDayColumnsView: View {
    let rows: [ValueRow]
    let dates: [Date]
    let dayWidth: CGFloat
    
    var body: some View {
        LazyHStack(alignment: .top, spacing: 1) {
            ForEach(dates, id: \.self) { date in
                ValuesDayColumn(date: date, rows: rows, width: dayWidth)
            }
        }
    }
}

struct ValuesLabelColumn: View {
    let rows: [ValueRow]
    let width: CGFloat
    
    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            Text("Values")
                .font(.caption2)
                .foregroundStyle(.secondary)
                .frame(height: valueSectionHeaderHeight, alignment: .bottomLeading)
                .padding(.leading, 6)
            
            Divider()
            
            ForEach(rows) { row in
                VStack(alignment: .leading, spacing: 2) {
                    Text(row.display)
                        .font(.caption2.weight(.semibold))
                        .lineLimit(2)
                    Spacer(minLength: 0)
                }
                .padding(.horizontal, 6)
                .padding(.top, valueRowPadding)
                .frame(height: valueRowHeight(for: row), alignment: .topLeading)
                
                Divider()
            }
        }
        .frame(width: width, alignment: .topLeading)
        .border(width: 1, edges: [.trailing], color: Color(nsColor: .separatorColor))
    }
}

struct ValuesDayColumn: View {
    let date: Date
    let rows: [ValueRow]
    let width: CGFloat
    
    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            Rectangle()
                .fill(Color.clear)
                .frame(height: valueSectionHeaderHeight)
            
            Divider()
            
            ForEach(rows) { row in
                ValuesDayCell(entries: row.entriesByDay[date] ?? [], height: valueRowHeight(for: row))
                Divider()
            }
        }
        .frame(width: width, alignment: .topLeading)
        .border(width: 1, edges: [.trailing], color: Color(nsColor: .separatorColor).opacity(0.5))
    }
}

struct ValuesDayCell: View {
    let entries: [EngineValueEntry]
    let height: CGFloat
    
    var body: some View {
        VStack(alignment: .leading, spacing: 2) {
            if entries.isEmpty {
                Text("—")
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            } else {
                ForEach(entries.indices, id: \.self) { idx in
                    let entry = entries[idx]
                    HStack(alignment: .firstTextBaseline, spacing: 6) {
                        Text(entry.date, format: .dateTime.hour().minute())
                            .font(.caption2.monospaced())
                            .foregroundStyle(.secondary)
                        
                        Text(entry.value)
                            .font(.caption2)
                            .lineLimit(2)
                        Spacer()
                    }
                }
            }
            Spacer(minLength: 0)
        }
        .padding(.horizontal, 6)
        .padding(.top, valueRowPadding)
        .frame(height: height, alignment: .topLeading)
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
                .frame(width: 400, height: 500)
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

    var isMessage: Bool {
        event.type == 1
    }
    
    var pillColor: Color {
        if isQuestion { return .orange }
        if isAnswer { return .green }
        if isMessage { return .purple }
        return .blue
    }
    
    var body: some View {
        HStack(alignment: .top, spacing: 6) {
            Text(event.time, format: .dateTime.hour().minute())
                .font(.caption2.monospaced())
                .foregroundStyle(.white.opacity(0.9))
                .padding(.top, 2)
            
            VStack(alignment: .leading, spacing: 2) {
                Text(isMessage ? "[Message] \(event.name)" : event.name)
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

// MARK: - Synced Horizontal Scroll

#if os(macOS)
struct SyncedHorizontalScrollView<Content: View>: NSViewRepresentable {
    @Binding var offset: CGFloat
    let showsIndicators: Bool
    let content: Content
    
    init(offset: Binding<CGFloat>, showsIndicators: Bool = true, @ViewBuilder content: () -> Content) {
        _offset = offset
        self.showsIndicators = showsIndicators
        self.content = content()
    }
    
    func makeCoordinator() -> Coordinator {
        Coordinator(offset: $offset)
    }
    
    func makeNSView(context: Context) -> NSScrollView {
        let scrollView = NSScrollView()
        scrollView.drawsBackground = false
        scrollView.hasHorizontalScroller = showsIndicators
        scrollView.hasVerticalScroller = false
        scrollView.autohidesScrollers = true
        scrollView.horizontalScrollElasticity = .allowed
        scrollView.verticalScrollElasticity = .none
        
        let hostingView = NSHostingView(rootView: content)
        hostingView.translatesAutoresizingMaskIntoConstraints = true
        scrollView.documentView = hostingView
        
        context.coordinator.scrollView = scrollView
        context.coordinator.hostingView = hostingView
        
        scrollView.contentView.postsBoundsChangedNotifications = true
        NotificationCenter.default.addObserver(
            context.coordinator,
            selector: #selector(Coordinator.boundsDidChange(_:)),
            name: NSView.boundsDidChangeNotification,
            object: scrollView.contentView
        )
        
        return scrollView
    }
    
    func updateNSView(_ nsView: NSScrollView, context: Context) {
        nsView.hasHorizontalScroller = showsIndicators
        
        if let hostingView = context.coordinator.hostingView {
            hostingView.rootView = content
            
            let fittingSize = hostingView.fittingSize
            let clipSize = nsView.contentView.bounds.size
            let targetSize = NSSize(
                width: max(fittingSize.width, clipSize.width),
                height: clipSize.height
            )
            if hostingView.frame.size != targetSize {
                hostingView.frame.size = targetSize
            }
            
            let maxOffset = max(0, targetSize.width - clipSize.width)
            let clampedOffset = min(max(offset, 0), maxOffset)
            if abs(clampedOffset - offset) > 0.5 {
                DispatchQueue.main.async {
                    offset = clampedOffset
                }
            }
            context.coordinator.applyOffset(clampedOffset)
        }
    }
    
    static func dismantleNSView(_ nsView: NSScrollView, coordinator: Coordinator) {
        NotificationCenter.default.removeObserver(
            coordinator,
            name: NSView.boundsDidChangeNotification,
            object: nsView.contentView
        )
    }
    
    final class Coordinator: NSObject {
        @Binding var offset: CGFloat
        weak var scrollView: NSScrollView?
        weak var hostingView: NSHostingView<Content>?
        private var isApplyingExternalOffset = false
        
        init(offset: Binding<CGFloat>) {
            _offset = offset
        }
        
        @objc func boundsDidChange(_ notification: Notification) {
            guard let clipView = notification.object as? NSClipView else { return }
            guard !isApplyingExternalOffset else { return }
            let newOffset = clipView.bounds.origin.x
            if abs(newOffset - offset) > 0.5 {
                offset = newOffset
            }
        }
        
        func applyOffset(_ newOffset: CGFloat) {
            guard let scrollView = scrollView else { return }
            let clipView = scrollView.contentView
            if abs(clipView.bounds.origin.x - newOffset) < 0.5 { return }
            isApplyingExternalOffset = true
            var point = clipView.bounds.origin
            point.x = newOffset
            clipView.setBoundsOrigin(point)
            scrollView.reflectScrolledClipView(clipView)
            isApplyingExternalOffset = false
        }
    }
}
#else
struct SyncedHorizontalScrollView<Content: View>: View {
    @Binding var offset: CGFloat
    let showsIndicators: Bool
    let content: Content
    
    init(offset: Binding<CGFloat>, showsIndicators: Bool = true, @ViewBuilder content: () -> Content) {
        _offset = offset
        self.showsIndicators = showsIndicators
        self.content = content()
    }
    
    var body: some View {
        ScrollView(.horizontal, showsIndicators: showsIndicators) {
            content
        }
    }
}
#endif

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
