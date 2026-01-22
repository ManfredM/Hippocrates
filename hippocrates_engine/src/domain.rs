use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum EventType {
    Log = 0,
    Message = 1,
    Question = 2,
    Answer = 3,
    Decision = 4,
    StateChange = 5,
    EventTrigger = 6,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: NaiveDateTime,
    pub event_type: EventType,
    pub details: String, // JSON payload or text
    pub context: Option<String>, // Context name or Rule name
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Unit {
    // Temperature
    Fahrenheit,
    Celsius,
    Percent,
    // Weight
    Milligram,
    Gram,
    Kilogram,
    Pound,
    Ounce,
    // Length
    Meter,
    Centimeter,
    Millimeter,
    Kilometer,
    Inch,
    Foot,
    Mile,
    // Volume
    Liter,
    Milliliter,
    FluidOunce,
    Gallon,
    // Time
    Year,
    Month,
    Week,
    Day,
    Hour,
    Minute,
    Second,
    // Pressure
    MillimeterOfMercury,
    // Clinical
    Bpm,
    MgPerDl,
    MmolPerL,
    // Custom
    Custom(String),
}

impl Unit {
    pub fn convert(&self, value: f64, target: &Unit) -> Result<f64, String> {
        if self == target {
            return Ok(value);
        }

        match (self, target) {
            // Temperature
            (Unit::Celsius, Unit::Fahrenheit) => Ok(value * 9.0 / 5.0 + 32.0),
            (Unit::Fahrenheit, Unit::Celsius) => Ok((value - 32.0) * 5.0 / 9.0),

            // Weight (to kg base)
            (Unit::Kilogram, Unit::Gram) => Ok(value * 1000.0),
            (Unit::Gram, Unit::Kilogram) => Ok(value / 1000.0),
            (Unit::Milligram, Unit::Gram) => Ok(value / 1000.0),
            (Unit::Gram, Unit::Milligram) => Ok(value * 1000.0),
            (Unit::Milligram, Unit::Kilogram) => Ok(value / 1_000_000.0),
            (Unit::Kilogram, Unit::Milligram) => Ok(value * 1_000_000.0),
            (Unit::Pound, Unit::Kilogram) => Ok(value * 0.45359237),
            (Unit::Kilogram, Unit::Pound) => Ok(value / 0.45359237),
            (Unit::Ounce, Unit::Gram) => Ok(value * 28.34952),
            (Unit::Gram, Unit::Ounce) => Ok(value / 28.34952),

            // Length (to meter base)
            (Unit::Meter, Unit::Centimeter) => Ok(value * 100.0),
            (Unit::Centimeter, Unit::Meter) => Ok(value / 100.0),
            (Unit::Meter, Unit::Millimeter) => Ok(value * 1000.0),
            (Unit::Millimeter, Unit::Meter) => Ok(value / 1000.0),
            (Unit::Kilometer, Unit::Meter) => Ok(value * 1000.0),
            (Unit::Meter, Unit::Kilometer) => Ok(value / 1000.0),
            (Unit::Inch, Unit::Centimeter) => Ok(value * 2.54),
            (Unit::Centimeter, Unit::Inch) => Ok(value / 2.54),
            (Unit::Foot, Unit::Meter) => Ok(value * 0.3048),
            (Unit::Meter, Unit::Foot) => Ok(value / 0.3048),
            (Unit::Mile, Unit::Kilometer) => Ok(value * 1.60934),
            (Unit::Kilometer, Unit::Mile) => Ok(value / 1.60934),

            // Volume (to Liter base)
            (Unit::Liter, Unit::Milliliter) => Ok(value * 1000.0),
            (Unit::Milliliter, Unit::Liter) => Ok(value / 1000.0),
            (Unit::Gallon, Unit::Liter) => Ok(value * 3.78541),
            (Unit::Liter, Unit::Gallon) => Ok(value / 3.78541),
            (Unit::FluidOunce, Unit::Milliliter) => Ok(value * 29.5735),
            (Unit::Milliliter, Unit::FluidOunce) => Ok(value / 29.5735),

            // Clinical
            (Unit::MmolPerL, Unit::MgPerDl) => Ok(value * 18.0182), // Glucose specific approximation
            (Unit::MgPerDl, Unit::MmolPerL) => Ok(value / 18.0182),

            _ => Err(format!("Cannot convert {:?} to {:?}", self, target)),
        }
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Unit::Custom(s) => write!(f, "{}", s),
            Unit::Fahrenheit => write!(f, "°F"),
            Unit::Celsius => write!(f, "°C"),
            Unit::Percent => write!(f, "%"),
            Unit::Milligram => write!(f, "mg"),
            Unit::Gram => write!(f, "g"),
            Unit::Kilogram => write!(f, "kg"),
            Unit::Pound => write!(f, "lb"),
            Unit::Ounce => write!(f, "oz"),
            Unit::Meter => write!(f, "m"),
            Unit::Centimeter => write!(f, "cm"),
            Unit::Millimeter => write!(f, "mm"),
            Unit::Kilometer => write!(f, "km"),
            Unit::Inch => write!(f, "inch"),
            Unit::Foot => write!(f, "foot"),
            Unit::Mile => write!(f, "mile"),
            Unit::Liter => write!(f, "l"),
            Unit::Milliliter => write!(f, "ml"),
            Unit::FluidOunce => write!(f, "fl oz"),
            Unit::Gallon => write!(f, "gal"),
            Unit::Year => write!(f, "year"),
            Unit::Month => write!(f, "month"),
            Unit::Week => write!(f, "week"),
            Unit::Day => write!(f, "day"),
            Unit::Hour => write!(f, "hour"),
            Unit::Minute => write!(f, "minute"),
            Unit::Second => write!(f, "second"),
            Unit::MillimeterOfMercury => write!(f, "mmHg"),
            Unit::Bpm => write!(f, "bpm"),
            Unit::MgPerDl => write!(f, "mg/dL"),
            Unit::MmolPerL => write!(f, "mmol/L"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValueType {
    Number,
    Enumeration,
    String,
    TimeIndication,
    Period,
    Plan,
    Drug,
    Addressee,
    AddresseeGroup,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValueDefinition {
    pub name: String,
    pub value_type: ValueType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RuntimeValue {
    Number(f64),
    Quantity(f64, Unit),
    String(String),
    Boolean(bool),
    Enumeration(String),
    List(Vec<RuntimeValue>),
    Date(NaiveDateTime),
    Void,
    NotEnoughData,
    Missing(String),
}

impl RuntimeValue {
    pub fn as_date(&self) -> Option<NaiveDateTime> {
        match self {
            RuntimeValue::Date(d) => Some(*d),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            RuntimeValue::Number(n) => Some(*n),
            RuntimeValue::Quantity(n, _) => Some(*n),
            _ => None,
        }
    }
}
impl fmt::Display for RuntimeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeValue::Number(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            RuntimeValue::Quantity(n, u) => {
                if n.fract() == 0.0 {
                    write!(f, "{} {}", *n as i64, u)
                } else {
                    write!(f, "{} {}", n, u)
                }
            }
            RuntimeValue::String(s) => write!(f, "{}", s),
            RuntimeValue::Boolean(b) => write!(f, "{}", b),
            RuntimeValue::Enumeration(e) => write!(f, "{}", e),
            RuntimeValue::Void => write!(f, "Void"),
            RuntimeValue::List(l) => write!(f, "{:?}", l),
            RuntimeValue::Date(d) => write!(f, "{}", d),
            RuntimeValue::NotEnoughData => write!(f, "Not Enough Data"),
            RuntimeValue::Missing(v) => write!(f, "Missing({})", v),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValueInstance {
    pub value: RuntimeValue,
    pub timestamp: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuestionStyle {
    Text,
    Selection,
    Likert,
    VisualAnalogueScale {
        min: f64,
        max: f64,
        min_label: String,
        max_label: String,
    },
    Numeric,
    Date,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputMessage {
    pub variable: String,
    pub value: RuntimeValue,
    pub timestamp: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationMode {
    Once,
    Twice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskRequest {
    pub variable_name: String,
    pub question_text: String,
    pub style: QuestionStyle,
    pub options: Vec<String>,
    pub range: Option<(f64, f64)>,
    pub validation_mode: Option<ValidationMode>,
    pub validation_timeout: Option<i64>,
    pub timestamp: i64,
    pub valid_after: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct EngineError {
    pub message: String,
    pub line: usize,
    pub column: usize, // Optional, default 0
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error at line {}, col {}: {}", self.line, self.column, self.message)
    }
}

impl std::error::Error for EngineError {}
