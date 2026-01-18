use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

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
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
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

#[derive(Debug, Clone, PartialEq)]
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
