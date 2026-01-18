import SwiftUI
import Charts

struct ExecutionEvent: Identifiable {
    let id = UUID()
    let name: String
    let time: Date
    let category: String
}

struct ExecutionTimelineView: View {
    // Mock Data simulating a plan execution trace
    let events: [ExecutionEvent] = [
        ExecutionEvent(name: "Start Plan", time: date(hour: 8), category: "System"),
        ExecutionEvent(name: "Welcome Message", time: date(hour: 8, minute: 1), category: "Message"),
        ExecutionEvent(name: "Measure Temperature", time: date(hour: 9), category: "Input"),
        ExecutionEvent(name: "Fever Detected (Trigger)", time: date(hour: 9, minute: 5), category: "Trigger"),
        ExecutionEvent(name: "Fever Alert", time: date(hour: 9, minute: 5, second: 1), category: "Message"),
        ExecutionEvent(name: "Take Paracetamol", time: date(hour: 9, minute: 10), category: "Action"),
        ExecutionEvent(name: "End Plan", time: date(hour: 20), category: "System")
    ]
    
    static func date(hour: Int, minute: Int = 0, second: Int = 0) -> Date {
        let cal = Calendar.current
        // Just use today for visualization
        return cal.date(bySettingHour: hour, minute: minute, second: second, of: Date()) ?? Date()
    }
    
    var body: some View {
        VStack(alignment: .leading) {
            Text("Simulated Run: Today")
                .font(.caption)
                .foregroundStyle(.secondary)
                .padding(.horizontal)
            
            Chart(events) { event in
                PointMark(
                    x: .value("Time", event.time),
                    y: .value("Event", event.name)
                )
                .symbol(by: .value("Category", event.category))
                .foregroundStyle(by: .value("Category", event.category))
                .annotation(position: .overlay, alignment: .bottom, spacing: 10) {
                    // Optional annotation
                }
            }
            .chartXAxis {
                AxisMarks(values: .automatic) { value in
                    AxisGridLine()
                    AxisTick()
                    AxisValueLabel(format: .dateTime.hour().minute())
                }
            }
            .chartYAxis {
                AxisMarks { value in
                    AxisGridLine()
                    AxisTick()
                    AxisValueLabel() // Shows absolute event names
                }
            }
            .padding()
            .frame(minHeight: 300)
        }
    }
}
