import SwiftUI
import Charts

struct ExecutionTimelineView: View {
    @EnvironmentObject var appState: AppState
    
    // Helper to extract just the time component for the Y-axis
    func timeOnly(from date: Date) -> Date {
        let cal = Calendar.current
        return cal.date(bySettingHour: cal.component(.hour, from: date),
                        minute: cal.component(.minute, from: date),
                        second: cal.component(.second, from: date), // Added seconds for precision
                        of: Date(timeIntervalSince1970: 0)) ?? date
    }
    
    var body: some View {
        VStack(alignment: .leading) {
            Text("Execution Live Feed")
                .font(.headline)
                .padding(.horizontal)
            
            if appState.executionLogs.isEmpty {
                ContentUnavailableView("No Execution Data", systemImage: "chart.xyaxis.line", description: Text("Run a plan to see events here."))
            } else {
                // Split view: Top Chart, Bottom Log List
                VSplitView {
                    VStack {
                        Chart(appState.executionLogs) { event in
                            PointMark(
                                x: .value("Date", event.time, unit: .day),
                                y: .value("Time", timeOnly(from: event.time))
                            )
                            .symbol(by: .value("Category", event.category))
                            .foregroundStyle(by: .value("Category", event.category))
                        }
                        .chartYScale(domain: .automatic(includesZero: false, reversed: true))
                        .chartXAxis {
                            AxisMarks(values: .stride(by: .day)) { value in
                                AxisGridLine()
                                AxisTick()
                                AxisValueLabel(format: .dateTime.weekday().day())
                            }
                        }
                        .chartYAxis {
                            AxisMarks { value in
                                AxisGridLine()
                                AxisTick()
                                AxisValueLabel(format: .dateTime.hour().minute().second())
                            }
                        }
                        .padding()
                    }
                    .frame(minHeight: 200)
                    
                    VStack(alignment: .leading) {
                        Text("Log History")
                            .font(.headline)
                            .padding(.horizontal)
                            .padding(.top, 5)
                            
                        List(appState.executionLogs) { event in
                            HStack(alignment: .top) {
                                Text(event.time, format: .dateTime.hour().minute().second())
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                                    .frame(width: 80)
                                Text(event.name)
                                    .font(.system(.body, design: .monospaced))
                            }
                        }
                        .listStyle(.inset)
                    }
                    .frame(minHeight: 150)
                }
            }
        }
    }
}
