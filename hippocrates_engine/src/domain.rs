use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum EventType {
    Log = 0,
    Message = 1,
    Question = 2,
    Answer = 3,
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
    // Custom
    Custom(String),
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Unit::Custom(s) => write!(f, "{}", s),
            _ => write!(f, "{:?}", self),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValueType {
    Number,
    Enumeration,
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
    Date(DateTime<Utc>),
    Void,
    NotEnoughData,
}

impl RuntimeValue {
    pub fn as_date(&self) -> Option<DateTime<Utc>> {
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
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValueInstance {
    pub value: RuntimeValue,
    pub timestamp: DateTime<Utc>,
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
