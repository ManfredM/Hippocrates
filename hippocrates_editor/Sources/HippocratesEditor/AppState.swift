import SwiftUI
import Combine

struct ExecutionEvent: Identifiable {
    let id = UUID()
    let name: String
    let time: Date
    let category: String
    let type: Int
}

class AppState: ObservableObject {
    @Published var planCode: String = """
    TreatmentPlan is a plan:
      during plan:
        show message "Welcome to the treatment".
        
      context:
        timeframe: today
        
      MyEvent with change of Fever:
        assess Temperature > 38°C:
          show message "Fever detected".
    """
    
    @Published var currentFileURL: URL?
    @Published var currentExecutionLine: Int?
    @Published var executionLogs: [ExecutionEvent] = []
    
    func load(url: URL) {
        do {
            let data = try Data(contentsOf: url)
            if let content = String(data: data, encoding: .utf8) {
                self.planCode = content
                self.currentFileURL = url
            }
        } catch {
            print("Failed to load file: \(error.localizedDescription)")
            // In a real app, we'd show an alert
        }
    }
}
