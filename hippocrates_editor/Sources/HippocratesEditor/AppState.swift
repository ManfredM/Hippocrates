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
    <Temperature> is a number:
      valid values: 35.0 ... 42.0
      question:
        ask "Please enter current temperature":
            validate answer once.

    <TreatmentPlan> is a plan:
      during plan:
        information to <patient> "Welcome to the treatment".
        ask for <Temperature>.
        
      <Monitor Fever> with change of <Temperature>:
        assess <Temperature>:
          38.0 ... 42.0:
            information to <patient> "Fever detected!".
          35.0 ... 38.0:
            information to <patient> "Temperature normal".
    """
    
    @Published var parseStatus: String = "Ready"
    @Published var currentErrors: [HippocratesParser.EngineError] = []
    
    @Published var currentFileURL: URL?
    @Published var currentExecutionLine: Int?
    @Published var executionLogs: [ExecutionEvent] = []
    @Published var pendingQuestion: AskRequest? = nil
    @Published var currentEngine: HippocratesEngine? = nil
    @Published var visualizationEngine: HippocratesEngine? = nil // For static analysis/viz
    
    func answerQuestion(value: String) {
        // This will be handled by the Wrapper/View bridging
        self.pendingQuestion = nil
    }
    
    func load(url: URL) {
        do {
            let data = try Data(contentsOf: url)
            if let content = String(data: data, encoding: .utf8) {
                self.planCode = content
                self.currentFileURL = url
                
                // Reset State
                self.currentExecutionLine = nil
                self.executionLogs = []
                self.pendingQuestion = nil
                self.currentEngine?.stop()
                self.currentEngine = nil
                
                // Clear errors on load
                self.parseStatus = "Ready"
                self.currentErrors = []
            }
        } catch {
            print("Failed to load file: \(error.localizedDescription)")
            // In a real app, we'd show an alert
        }
    }
}
