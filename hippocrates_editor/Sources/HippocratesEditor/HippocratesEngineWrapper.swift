import Foundation


struct HippocratesParser {
    
    struct ParseResult: Decodable {
        let Ok: Plan?
        let Err: String?
    }
    
    // Simplistic Plan model for now, expanded as needed for visualization
    struct Plan: Decodable {
        let definitions: [Definition]
    }
    
    struct Definition: Decodable {
        // Todo: Map all variants. For now, we decode what we can.
        // Rust enum serialization: {"Value": {...}}
        let Value: ValueDef?
        let Plan: PlanDef?
    }
    
    struct ValueDef: Decodable {
        let name: String
    }
    
    struct PlanDef: Decodable {
        let name: String
        // let blocks: ...
    }
    
    static func parse(input: String) -> Result<Plan, Error> {
        // Convert Swift String to C String
        guard let cString = input.cString(using: .utf8) else {
            return .failure(NSError(domain: "HippocratesEngine", code: 1, userInfo: [NSLocalizedDescriptionKey: "Invalid UTF-8 input"]))
        }
        
        // Call Rust
        let resultPtr = hippocrates_parse_json(cString)
        
        guard let resultPtr = resultPtr else {
            return .failure(NSError(domain: "HippocratesEngine", code: 2, userInfo: [NSLocalizedDescriptionKey: "Received null pointer from engine"]))
        }
        
        defer {
             hippocrates_free_string(resultPtr)
        }
        
        // Convert Result C String to Swift String
        let resultString = String(cString: resultPtr)
        
        // Decode JSON
        guard let data = resultString.data(using: String.Encoding.utf8) else {
             return .failure(NSError(domain: "HippocratesEngine", code: 3, userInfo: [NSLocalizedDescriptionKey: "Invalid UTF-8 output"]))
        }
        
        do {
            let result = try JSONDecoder().decode(ParseResult.self, from: data)
            if let plan = result.Ok {
                return .success(plan)
            } else if let err = result.Err {
                return .failure(NSError(domain: "HippocratesEngine", code: 4, userInfo: [NSLocalizedDescriptionKey: err]))
            } else {
                 return .failure(NSError(domain: "HippocratesEngine", code: 5, userInfo: [NSLocalizedDescriptionKey: "Unknown result format"]))
            }
        } catch {
            return .failure(error)
        }
    }
    static func run(input: String, planName: String, onStep: @escaping (Int) -> Void, onLog: @escaping (String, Int, Date) -> Void) {
        execute(input: input, planName: planName, onStep: onStep, onLog: onLog) { scriptC, planC, lineCb, logCb, context in
            hippocrates_run(scriptC, planC, lineCb, logCb, context)
        }
    }

    static func simulate(input: String, planName: String, days: Int, onStep: @escaping (Int) -> Void, onLog: @escaping (String, Int, Date) -> Void) {
        execute(input: input, planName: planName, onStep: onStep, onLog: onLog) { scriptC, planC, lineCb, logCb, context in
            hippocrates_simulate(scriptC, planC, lineCb, logCb, context, Int32(days))
        }
    }
    
    private static func execute(input: String, planName: String, onStep: @escaping (Int) -> Void, onLog: @escaping (String, Int, Date) -> Void,
                                executor: (UnsafePointer<CChar>, UnsafePointer<CChar>, LineCallback, LogCallback, UnsafeMutableRawPointer) -> Void) {
        guard let scriptC = input.cString(using: .utf8),
              let planC = planName.cString(using: .utf8) else { return }
        
        class RunContext {
            let onStep: (Int) -> Void
            let onLog: (String, Int, Date) -> Void
            init(step: @escaping (Int) -> Void, log: @escaping (String, Int, Date) -> Void) {
                self.onStep = step
                self.onLog = log
            }
        }
        
        let box = RunContext(step: onStep, log: onLog)
        let context = Unmanaged.passRetained(box).toOpaque()
        
        let lineCb: LineCallback = { line, ctx in
            // Handle context and call onStep
             if let ctx = ctx {
                 let box = Unmanaged<RunContext>.fromOpaque(ctx).takeUnretainedValue()
                 box.onStep(Int(line))
            }
        }
        
        let logCb: LogCallback = { msgPtr, type, timestamp, ctx in
            if let ctx = ctx, let msgPtr = msgPtr {
                let box = Unmanaged<RunContext>.fromOpaque(ctx).takeUnretainedValue()
                let msg = String(cString: msgPtr)
                let date = Date(timeIntervalSince1970: TimeInterval(timestamp) / 1000.0)
                box.onLog(msg, Int(type), date)
            }
        }
        
        executor(scriptC, planC, lineCb, logCb, context)
        
        Unmanaged<RunContext>.fromOpaque(context).release()
    }
}

// MARK: - New Engine Interface

enum QuestionStyle: Decodable {
    case Text
    case Selection
    case Likert
    case Numeric
    case Date
    case Unknown
    case VisualAnalogueScale(min: Double, max: Double, min_label: String, max_label: String)
    
    enum CodingKeys: String, CodingKey {
        case Text, Selection, Likert, Numeric, Date, Unknown, VisualAnalogueScale
    }
    
    struct VASData: Decodable {
        let min: Double
        let max: Double
        let min_label: String
        let max_label: String
    }
    
    init(from decoder: Decoder) throws {
        if let container = try? decoder.singleValueContainer(), let val = try? container.decode(String.self) {
             switch val {
             case "Text": self = .Text
             case "Selection": self = .Selection
             case "Likert": self = .Likert
             case "Numeric": self = .Numeric
             case "Date": self = .Date
             default: self = .Unknown
             }
             return
        }
        let container = try decoder.container(keyedBy: CodingKeys.self)
        if let vas = try? container.decode(VASData.self, forKey: .VisualAnalogueScale) {
            self = .VisualAnalogueScale(min: vas.min, max: vas.max, min_label: vas.min_label, max_label: vas.max_label)
            return
        }
        self = .Unknown
    }
}

struct AskRequest: Decodable {
    let variable_name: String
    let question_text: String
    let style: QuestionStyle
    let options: [String]
    let range: [Double]?
}

class HippocratesEngine {
    private var ctx: OpaquePointer?
    
    var onStep: ((Int) -> Void)?
    var onLog: ((String, Int, Date) -> Void)?
    var onAsk: ((AskRequest) -> Void)?
    
    init() {
        // Pass self as context? Using Unmanaged.passUnretained(self)
        // But callbacks are C functions. We need a way to route back to self.
        // We use a helper object or pass self pointer.
        // Note: 'self' must be stable in memory or retained.
        // hippocrates_engine_new calls:
        let contextPtr = Unmanaged.passUnretained(self).toOpaque()
        self.ctx = hippocrates_engine_new(contextPtr)
        
        setupCallbacks()
    }
    
    deinit {
        if let ctx = ctx {
            hippocrates_engine_free(ctx)
        }
    }
    
    private func setupCallbacks() {
        guard let ctx = ctx else { return }
        
        let lineCb: LineCallback = { line, userData in
            guard let userData = userData else { return }
            let engine = Unmanaged<HippocratesEngine>.fromOpaque(userData).takeUnretainedValue()
            engine.onStep?(Int(line))
        }
        
        let logCb: LogCallback = { msgPtr, type, ts, userData in
             guard let userData = userData, let msgPtr = msgPtr else { return }
             let engine = Unmanaged<HippocratesEngine>.fromOpaque(userData).takeUnretainedValue()
             let msg = String(cString: msgPtr)
             let date = Date(timeIntervalSince1970: TimeInterval(ts) / 1000.0)
             engine.onLog?(msg, Int(type), date)
        }
        
        let askCb: AskCallback = { reqJsonPtr, userData in
             guard let userData = userData, let reqJsonPtr = reqJsonPtr else { return }
             let engine = Unmanaged<HippocratesEngine>.fromOpaque(userData).takeUnretainedValue()
             let jsonStr = String(cString: reqJsonPtr)
             if let data = jsonStr.data(using: .utf8),
                let req = try? JSONDecoder().decode(AskRequest.self, from: data) {
                 engine.onAsk?(req)
             }
        }
        
        hippocrates_engine_set_callbacks(ctx, lineCb, logCb, askCb)
    }
    
    func load(source: String) -> Bool {
        guard let ctx = ctx, let cSource = source.cString(using: .utf8) else { return false }
        return hippocrates_engine_load(ctx, cSource) == 0
    }
    
    func execute(planName: String) {
        guard let ctx = ctx, let cName = planName.cString(using: .utf8) else { return }
        hippocrates_engine_execute(ctx, cName)
    }
    
    func setValue(name: String, valueJson: String) -> Bool {
        guard let ctx = ctx, 
              let cName = name.cString(using: .utf8),
              let cVal = valueJson.cString(using: .utf8) else { return false }
        return hippocrates_engine_set_value(ctx, cName, cVal) == 0
    }
}
