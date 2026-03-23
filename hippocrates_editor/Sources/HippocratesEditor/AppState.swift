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
(* Hippocrates — Post-Dental Implant Care Plan *)

<point> is a unit:
    plural is <points>.

<patient> is an addressee:
    contact information:
        email is "patient@email.com".

<dentist> is an addressee:
    contact information:
        email is "dr_mueller@dental.com".

<body temperature> is a number:
    valid values:
        35.0 °C ... 42.0 °C.
    unit: °C.
    question:
        ask "What is your body temperature?".

<pain level> is a number:
    valid values:
        0 <points> ... 10 <points>.
    question:
        ask "How severe is your pain? (0 = none, 10 = worst)":
            validate answer once.

<implant care> is a plan:
    before plan:
        information to <patient> "Welcome to your post-implant care plan.".
        information to <patient> "Monitoring pain and temperature for 9 days.".
        ask for <body temperature>.

    every day at 08:00 for 9 days:
        ask for <pain level>.
        assess <pain level>:
            0 <points> ... 3 <points>:
                information to <patient> "Pain is manageable. Keep it up!".
            4 <points> ... 7 <points>:
                information to <patient> "Moderate pain. Take prescribed medication.".
            8 <points> ... 10 <points>:
                information to <patient> "Severe pain detected.".
                information to <dentist> "Patient reports severe pain.".
        ask for <body temperature>.
        assess <body temperature>:
            35.0 °C ... 37.5 °C:
                information to <patient> "Temperature is normal.".
            37.6 °C ... 42.0 °C:
                information to <patient> "Elevated temperature. Contact your dentist.".
                information to <dentist> "Patient has elevated temperature.".

    after plan:
        information to <patient> "Your post-implant care plan is now complete.".
        information to <patient> "Please schedule a follow-up visit with your dentist.".
        information to <dentist> "Patient care plan completed.".
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
