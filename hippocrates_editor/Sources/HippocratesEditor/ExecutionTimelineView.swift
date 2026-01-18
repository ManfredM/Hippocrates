import SwiftUI
import Charts

struct ExecutionEvent: Identifiable {
    let id = UUID()
    let name: String
    let time: Date
    let category: String
}

struct ExecutionTimelineView: View {
    // Mock Data spanning multiple days
    let events: [ExecutionEvent] = [
        // Day 1
        ExecutionEvent(name: "Start Plan", time: date(dayOffset: 0, hour: 8), category: "System"),
        ExecutionEvent(name: "Measure Temp", time: date(dayOffset: 0, hour: 9), category: "Input"),
        ExecutionEvent(name: "Take Pill", time: date(dayOffset: 0, hour: 9, minute: 10), category: "Action"),
        
        // Day 2
        ExecutionEvent(name: "Measure Temp", time: date(dayOffset: 1, hour: 9), category: "Input"),
        ExecutionEvent(name: "Take Pill", time: date(dayOffset: 1, hour: 9, minute: 5), category: "Action"),
        
        // Day 3
        ExecutionEvent(name: "Measure Temp", time: date(dayOffset: 2, hour: 9), category: "Input"),
        ExecutionEvent(name: "Fever!", time: date(dayOffset: 2, hour: 9, minute: 15), category: "Trigger"),
        ExecutionEvent(name: "Take Pill", time: date(dayOffset: 2, hour: 9, minute: 20), category: "Action")
    ]
    
    // Helper to generate dates relative to today
    static func date(dayOffset: Int, hour: Int, minute: Int = 0, second: Int = 0) -> Date {
        let cal = Calendar.current
        let day = cal.date(byAdding: .day, value: dayOffset, to: Date())!
        return cal.date(bySettingHour: hour, minute: minute, second: second, of: day) ?? day
    }
    
    // Helper to extract just the time component for the Y-axis (normalizing to a dummy date)
    func timeOnly(from date: Date) -> Date {
        let cal = Calendar.current
        // Set all dates to the same reference day (e.g., Jan 1, 2000) so they align on Y-axis
        return cal.date(bySettingHour: cal.component(.hour, from: date),
                        minute: cal.component(.minute, from: date),
                        second: 0,
                        of: Date(timeIntervalSince1970: 0)) ?? date
    }
    
    var body: some View {
        VStack(alignment: .leading) {
            Text("Execution Schedule")
                .font(.headline)
                .padding(.horizontal)
            
            Chart(events) { event in
                PointMark(
                    x: .value("Date", event.time, unit: .day),
                    y: .value("Time", timeOnly(from: event.time))
                )
                .symbol(by: .value("Category", event.category))
                .foregroundStyle(by: .value("Category", event.category))
                .annotation(position: .overlay, alignment: .bottom, spacing: 5) {
                   // Minimal annotation to avoid clutter
                }
            }
            .chartYScale(domain: .automatic(includesZero: false, reversed: true)) // Time typically runs top-down in schedules
            .chartXAxis {
                AxisMarks(values: .stride(by: .day)) { value in
                    AxisGridLine()
                    AxisTick()
                    AxisValueLabel(format: .dateTime.weekday().day())
                }
            }
            .chartYAxis {
                AxisMarks(values: .stride(by: .hour, count: 2)) { value in
                    AxisGridLine()
                    AxisTick()
                    AxisValueLabel(format: .dateTime.hour())
                }
            }
            .padding()
            .frame(minHeight: 400)
        }
    }
}
