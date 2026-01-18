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
    static func run(input: String, planName: String, onStep: @escaping (Int) -> Void, onLog: @escaping (String, Date) -> Void) {
        execute(input: input, planName: planName, onStep: onStep, onLog: onLog) { scriptC, planC, lineCb, logCb, context in
            hippocrates_run(scriptC, planC, lineCb, logCb, context)
        }
    }

    static func simulate(input: String, planName: String, days: Int, onStep: @escaping (Int) -> Void, onLog: @escaping (String, Date) -> Void) {
        execute(input: input, planName: planName, onStep: onStep, onLog: onLog) { scriptC, planC, lineCb, logCb, context in
            hippocrates_simulate(scriptC, planC, lineCb, logCb, context, Int32(days))
        }
    }
    
    private static func execute(input: String, planName: String, onStep: @escaping (Int) -> Void, onLog: @escaping (String, Date) -> Void,
                                executor: (UnsafePointer<CChar>, UnsafePointer<CChar>, LineCallback, LogCallback, UnsafeMutableRawPointer) -> Void) {
        guard let scriptC = input.cString(using: .utf8),
              let planC = planName.cString(using: .utf8) else { return }
        
        class RunContext {
            let onStep: (Int) -> Void
            let onLog: (String, Date) -> Void
            init(step: @escaping (Int) -> Void, log: @escaping (String, Date) -> Void) {
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
                 // Ensure UI updates on main thread if needed, but callback usually background.
                 // Swift side handles dispatch.
                 box.onStep(Int(line))
            }
        }
        
        let logCb: LogCallback = { msgPtr, timestamp, ctx in
            if let ctx = ctx, let msgPtr = msgPtr {
                let box = Unmanaged<RunContext>.fromOpaque(ctx).takeUnretainedValue()
                let msg = String(cString: msgPtr)
                // Convert timestamp (millis) to Date
                let date = Date(timeIntervalSince1970: TimeInterval(timestamp) / 1000.0)
                box.onLog(msg, date)
            }
        }
        
        executor(scriptC, planC, lineCb, logCb, context)
        
        Unmanaged<RunContext>.fromOpaque(context).release()
    }
}
