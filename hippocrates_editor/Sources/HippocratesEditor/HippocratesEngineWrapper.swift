import Foundation

struct HippocratesParser {
    
    struct EngineError: Decodable, Identifiable, Error, LocalizedError {
        var id: String { message }
        let message: String
        let line: Int
        let column: Int
        
        var errorDescription: String? {
            return message
        }
    }

    struct ParseResult: Decodable {
        let Ok: Plan?
        let Err: EngineError?
    }
    
    // Simplistic Plan model for now, expanded as needed for visualization
    struct Plan: Decodable {
        let definitions: [Definition]
    }
    
    struct Definition: Decodable {
        let Value: ValueDef?
        let Plan: PlanDef?
        let Period: PeriodDef?
    }
    
    struct ValueDef: Decodable {
        let name: String
    }
    
    struct PlanDef: Decodable {
        let name: String
    }
    
    struct PeriodDef: Decodable {
        let name: String
        let line: Int
        let timeframes: [[RangeSelector]]
    }
    

    static func parse(input: String) -> Result<Plan, EngineError> {
        // Convert Swift String to C String
        guard let cString = input.cString(using: .utf8) else {
            return .failure(EngineError(message: "Invalid UTF-8 input", line: 0, column: 0))
        }
        
        // Call Rust
        let resultPtr = hippocrates_parse_json(cString)
        
        guard let resultPtr = resultPtr else {
            return .failure(EngineError(message: "Received null pointer from engine", line: 0, column: 0))
        }
        
        defer {
             hippocrates_free_string(resultPtr)
        }
        
        // Convert Result C String to Swift String
        let resultString = String(cString: resultPtr)
        
        // Decode JSON
        guard let data = resultString.data(using: String.Encoding.utf8) else {
             return .failure(EngineError(message: "Invalid UTF-8 output", line: 0, column: 0))
        }
        
        do {
            let result = try JSONDecoder().decode(ParseResult.self, from: data)
            if let plan = result.Ok {
                return .success(plan)
            } else if let err = result.Err {
                return .failure(err)
            } else {
                 return .failure(EngineError(message: "Unknown result format", line: 0, column: 0))
            }
        } catch {
            return .failure(EngineError(message: "JSON Decode Error: \(error.localizedDescription)", line: 0, column: 0))
        }
    }

    static func validate(input: String) -> [EngineError] {
        guard let cString = input.cString(using: .utf8) else {
             return [EngineError(message: "Invalid UTF-8", line: 0, column: 0)]
        }
        let count = hippocrates_validate_file(cString)
        if count == 0 { return [] }
        
        var errors = [EngineError]()
        for i in 0..<count {
            if let ptr = hippocrates_get_error(Int32(i)) {
                 defer { hippocrates_free_string(ptr) }
                 let jsonStr = String(cString: ptr)
                 if let data = jsonStr.data(using: String.Encoding.utf8),
                    let err = try? JSONDecoder().decode(EngineError.self, from: data) {
                     errors.append(err)
                 }
            }
        }
        return errors
    }

    static func prepareEngine(_ source: String, simulate: Bool = false, simulationDays: Int = 30, onStep: @escaping (Int) -> Void, onLog: @escaping (String, Int, Date) -> Void, onAsk: @escaping (AskRequest) -> Void) -> HippocratesEngine? {
        let engine = HippocratesEngine()
        
        engine.onStep = onStep
        engine.onLog = onLog
        engine.onAsk = onAsk
        
        if simulate {
             // Use new Simulation mode with timelapse (speed_factor: None -> instant)
             engine.setSimulationMode(days: simulationDays)
        }
        
        if engine.load(source: source) {
            return engine
        }
        return nil
    }
}

// MARK: - New Engine Interface

enum QuestionStyle: Decodable, Equatable {
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

enum ValidationMode: String, Decodable {
    case Once
    case Twice
}

struct AskRequest: Decodable, Identifiable {
    var id: String { variable_name }
    let variable_name: String
    let question_text: String
    let style: QuestionStyle
    let options: [String]
    let range: [Double]?
    let validation_mode: ValidationMode?
    let validation_timeout: Int64?
    let timestamp: Int64
}

class HippocratesEngine: Equatable {
    static func == (lhs: HippocratesEngine, rhs: HippocratesEngine) -> Bool {
        return lhs === rhs
    }

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
             // Engine sends "Naive Time" as timestamp (millis).
             // Treated as GMT by Date(timeIntervalSince1970:) which corresponds to "Wall Clock Time" in UTC.
             // UI should display this with .timeZone(.gmt) to show the exact clock digits.
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
        
        let resultPtr = hippocrates_engine_load(ctx, cSource)
        guard let ptr = resultPtr else { return false }
        defer { hippocrates_free_string(ptr) }
        
        let jsonStr = String(cString: ptr)
        // We can check if it starts with {"Ok"
        if jsonStr.contains("\"Ok\"") {
            return true
        }
        return false
    }
    
    func setSimulationMode(days: Int = 30) {
        guard let ctx = ctx else { return }
        // Convert to minutes
        let mins = Int32(days * 24 * 60)
        hippocrates_engine_enable_simulation(ctx, mins)
    }
    
    func execute(planName: String) {
        guard let ctx = ctx, let cName = planName.cString(using: .utf8) else { return }
        hippocrates_engine_execute(ctx, cName)
    }
    
    func stop() {
        if let ctx = ctx {
            hippocrates_engine_stop(ctx)
        }
    }
    
    func getPeriods() -> [HippocratesParser.PeriodDef] {
        guard let ctx = ctx else { return [] }
        let ptr = hippocrates_get_periods(ctx)
        guard let p = ptr else { return [] }
        defer { hippocrates_free_string(p) }
        
        let jsonStr = String(cString: p)
        if let data = jsonStr.data(using: .utf8),
           let periods = try? JSONDecoder().decode([HippocratesParser.PeriodDef].self, from: data) {
            return periods
        }
        return []
    }

    struct PeriodOccurrence: Decodable {
        let start: Date
        let end: Date
        
        enum CodingKeys: String, CodingKey {
            case start, end
        }
        
        init(from decoder: Decoder) throws {
            let container = try decoder.container(keyedBy: CodingKeys.self)
            let startStr = try container.decode(String.self, forKey: .start)
            let endStr = try container.decode(String.self, forKey: .end)
            
            // Helper to parse dates with multiple potential formats (ISO8601 with/without fractional seconds)
            func parse(_ s: String) -> Date? {
                // Strict "Naive" matching: Input has no offset (e.g. "2026-01-01T12:00:00")
                // We treat this as Abstract Time = GMT.
                let f = DateFormatter()
                f.dateFormat = "yyyy-MM-dd'T'HH:mm:ss"
                f.timeZone = TimeZone(secondsFromGMT: 0)
                f.locale = Locale(identifier: "en_US_POSIX")
                if let d = f.date(from: s) { return d }
                
                // Fallback for fractional seconds if needed (e.g. .123)
                f.dateFormat = "yyyy-MM-dd'T'HH:mm:ss.SSS"
                if let d = f.date(from: s) { return d }
                
                // Fallback to ISO if serde adds Z (unexpected but safe)
                let fISO = ISO8601DateFormatter()
                return fISO.date(from: s)
            }
            
            guard let s = parse(startStr), let e = parse(endStr) else {
                throw DecodingError.dataCorruptedError(forKey: .start, in: container, debugDescription: "Invalid date format")
            }
            self.start = s
            self.end = e
        }
    }

    func simulateOccurrences(periodName: String, days: Int, startDate: Date? = nil) -> [PeriodOccurrence] {
        guard let ctx = ctx, let cName = periodName.cString(using: .utf8) else { return [] }
        // Use provided start time or current time
        let rawDate = startDate ?? Date()
        // Convert Wall Clock Time (Local) to Abstract Time (GMT-as-Local)
        // If Local is 15:00 (+1), rawDate is 14:00 GMT.
        // We want engine to see "15:00". So we shift it by offset.
        let tz = TimeZone.current
        let seconds = tz.secondsFromGMT(for: rawDate)
        let abstractDate = rawDate.addingTimeInterval(TimeInterval(seconds))
        
        let startTs = Int64(abstractDate.timeIntervalSince1970 * 1000)
        let ptr = hippocrates_simulate_occurrences(ctx, cName, startTs, Int32(days))
        guard let p = ptr else { return [] }
        defer { hippocrates_free_string(p) }
        
        let jsonStr = String(cString: p)
        if let data = jsonStr.data(using: String.Encoding.utf8),
           let occurrences = try? JSONDecoder().decode([PeriodOccurrence].self, from: data) {
            return occurrences
        }
        return []
    }
    
    func setValue(name: String, valueJson: String) -> Bool {
        guard let ctx = ctx, 
              let cName = name.cString(using: .utf8),
              let cVal = valueJson.cString(using: .utf8) else { return false }
        return hippocrates_engine_set_value(ctx, cName, cVal) == 0
    }
    
    func setTime(_ date: Date) {
        guard let ctx = ctx else { return }
        // Convert Wall Clock Time to Abstract Time
        let tz = TimeZone.current
        let seconds = tz.secondsFromGMT(for: date)
        let abstractDate = date.addingTimeInterval(TimeInterval(seconds))
        let ts = Int64(abstractDate.timeIntervalSince1970 * 1000)
        
        hippocrates_engine_set_time(ctx, ts)
    }
}


extension HippocratesParser {
    // Helper Structures for Visualization
    enum RangeSelector: Decodable {
        case Between(Expression, Expression)
        case Range(Expression, Expression)
        case Equals(Expression)
        case List([Expression])
        case Unknown
        
        enum CodingKeys: String, CodingKey {
            case Between, Range, Equals, List
        }
        
        init(from decoder: Decoder) throws {
            if let container = try? decoder.container(keyedBy: CodingKeys.self) {
                if let arr = try? container.decode([Expression].self, forKey: .Between), arr.count == 2 {
                    self = .Between(arr[0], arr[1])
                    return
                }
                if let arr = try? container.decode([Expression].self, forKey: .Range), arr.count == 2 {
                    self = .Range(arr[0], arr[1])
                    return
                }
                if let expr = try? container.decode(Expression.self, forKey: .Equals) {
                    self = .Equals(expr)
                    return
                }
                if let list = try? container.decode([Expression].self, forKey: .List) {
                    self = .List(list)
                    return
                }
            }
            self = .Unknown
        }
    }

    enum Expression: Decodable {
        case Literal(LiteralValue)
        case Variable(String)
        case Unknown
        
        enum CodingKeys: String, CodingKey {
            case Literal, Variable
        }
        
        init(from decoder: Decoder) throws {
            if let container = try? decoder.container(keyedBy: CodingKeys.self) {
                if let lit = try? container.decode(LiteralValue.self, forKey: .Literal) {
                    self = .Literal(lit)
                    return
                }
                if let varName = try? container.decode(String.self, forKey: .Variable) {
                    self = .Variable(varName)
                    return
                }
            }
            // Fallback for string-enum based expression variants (e.g. Statistical) or complex
            self = .Unknown
        }
    }

    enum LiteralValue: Decodable {
        case TimeOfDay(String)
        case StringVal(String)
        case Number(Double)
        case Unknown
        
        enum CodingKeys: String, CodingKey {
            case TimeOfDay, String, Number
        }
        
        init(from decoder: Decoder) throws {
            if let container = try? decoder.container(keyedBy: CodingKeys.self) {
                if let s = try? container.decode(String.self, forKey: .TimeOfDay) {
                    self = .TimeOfDay(s)
                    return
                }
                if let s = try? container.decode(String.self, forKey: .String) {
                    self = .StringVal(s)
                    return
                }
                 // Number in Rust JSON might be Number(f64, Option<usize>) which maps to [f64, ?] or just simple struct
                 // Wait, Rust serialization for tuple variants: {"Number":[10.0, null]}
                 if let arr = try? container.decode([Double?].self, forKey: .Number), let first = arr.first, let val = first {
                     self = .Number(val)
                     return
                 }
            }
            self = .Unknown
        }
    }
}
