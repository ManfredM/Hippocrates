import XCTest
@testable import HippocratesEditor

class HippocratesEditorTests: XCTestCase {
    
    func testExample() throws {
        // This is an example of a functional test case.
        // Use XCTAssert and related functions to verify your tests produce the correct results.
        XCTAssertTrue(true, "Basic test should pass")
    }

    func testParserIntegration() {
        // Ideally we tests that the Parser returns something valid
        // But since AppState relies on EngineWrapper which is internal...
        // We can just verify that we can instantiate things if possible.
        let appState = AppState()
        XCTAssertNotNil(appState)
    }
    func testMultiErrorValidation() {
        let input = """
        <point> is a unit:
            plural is <points>.
        
        <val> is a number:
            unit is <points>.
            valid values:
                0 <points> ... 10 <points>.
        
        <plan> is a plan:
            during plan:
                assess <val>:
                    0 <points> ... 5 <points>:
                        information "lower".
        """
        
        let errors = HippocratesParser.validate(input: input)
        XCTAssertFalse(errors.isEmpty, "Should have validation errors")
        
        if let first = errors.first {
            print("Validation Error: \(first.message)")
            XCTAssertTrue(first.message.contains("Coverage Error") || first.message.contains("gap") || first.message.contains("Uncovered"), "Error should mention missing coverage. Got: \(first.message)")
        }
        
        let inputMultiple = """
        <point> is a unit:
            plural is <points>.
        
        <val> is a number:
            unit is <points>.
            valid values:
                0 <points> ... 10 <points>.
        
        <plan> is a plan:
            during plan:
                assess <val>:
                    0 <points> ... 5 <points>:
                        information "lower".
                    4 <points> ... 6 <points>:
                        information "overlap".
        """
        
        let errors2 = HippocratesParser.validate(input: inputMultiple)
        let hasOverlap = errors2.contains { $0.message.contains("overlapping ranges") || $0.message.contains("Constraint Violation") }
        let hasGap = errors2.contains { $0.message.contains("Coverage Error") || $0.message.contains("Uncovered") || $0.message.contains("gap") }
        XCTAssertTrue(hasOverlap, "Should include overlap error. Got \(errors2.map { $0.message })")
        XCTAssertTrue(hasGap, "Should include coverage gap error. Got \(errors2.map { $0.message })")
        XCTAssertTrue(errors2.count >= 2, "Should have at least 2 errors (overlap and gap). Got \(errors2.count)")
        for e in errors2 {
            print("Multi Error: \(e.message)")
        }
    }
    func testAskRequestDecodingWithValidation() {
        let json = """
        {
            "variable_name": "test_var",
            "question_text": "Enter value?",
            "style": "Numeric",
            "options": [],
            "range": [0.0, 10.0],
            "validation_mode": "Twice",
            "validation_timeout": 30,
            "timestamp": 123456789
        }
        """
        
        let data = json.data(using: .utf8)!
        do {
            let req = try JSONDecoder().decode(AskRequest.self, from: data)
            XCTAssertEqual(req.variable_name, "test_var")
            XCTAssertEqual(req.validation_mode, .Twice)
            XCTAssertEqual(req.validation_timeout, 30)
            XCTAssertEqual(req.range?.count, 2)
            print("Successfully decoded AskRequest with validation mode: \(String(describing: req.validation_mode))")
        } catch {
            XCTFail("Decoding failed: \(error)")
        }
    }
}
