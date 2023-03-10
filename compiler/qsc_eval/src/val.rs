// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{collections::HashMap, ffi::c_void, fmt::Display, ops::Neg};

use num_bigint::BigInt;
use qir_backend::Pauli;
use qsc_frontend::resolve::DefId;

#[derive(Clone, Debug)]
pub enum Value {
    Array(Vec<Value>),
    BigInt(BigInt),
    Bool(bool),
    Closure(DefId, HashMap<DefId, Value>),
    Double(f64),
    Global(DefId),
    Int(i64),
    Pauli(Pauli),
    Qubit(*mut c_void),
    Range(Option<i64>, Option<i64>, Option<i64>),
    Result(bool),
    String(String),
    Tuple(Vec<Value>),
    Udt,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Array(arr) => {
                write!(f, "[")?;
                join(f, arr.iter(), ", ")?;
                write!(f, "]")
            }
            Value::BigInt(v) => write!(f, "{v}"),
            Value::Bool(v) => write!(f, "{v}"),
            Value::Closure(_, _) => todo!(),
            Value::Double(v) => {
                if (v.floor() - v.ceil()).abs() < f64::EPSILON {
                    // The value is a whole number, which by convention is displayed with one decimal point
                    // to differentiate it from an integer value.
                    write!(f, "{v:.1}")
                } else {
                    write!(f, "{v}")
                }
            }
            Value::Global(_) => todo!(),
            Value::Int(v) => write!(f, "{v}"),
            Value::Pauli(v) => match v {
                Pauli::I => write!(f, "PauliI"),
                Pauli::X => write!(f, "PauliX"),
                Pauli::Z => write!(f, "PauliZ"),
                Pauli::Y => write!(f, "PauliY"),
            },
            Value::Qubit(v) => write!(f, "{}", (*v as usize)),
            Value::Range(start, step, end) => match (start, step, end) {
                (Some(start), Some(step), Some(end)) => write!(f, "{start}..{step}..{end}"),
                (Some(start), Some(step), None) => write!(f, "{start}..{step}..."),
                (Some(start), None, Some(end)) => write!(f, "{start}..{end}"),
                (Some(start), None, None) => write!(f, "{start}..."),
                (None, Some(step), Some(end)) => write!(f, "...{step}..{end}"),
                (None, Some(step), None) => write!(f, "...{step}..."),
                (None, None, Some(end)) => write!(f, "...{end}"),
                (None, None, None) => write!(f, "..."),
            },
            Value::Result(v) => {
                if *v {
                    write!(f, "One")
                } else {
                    write!(f, "Zero")
                }
            }
            Value::String(v) => write!(f, "{v}"),
            Value::Tuple(tup) => {
                write!(f, "(")?;
                join(f, tup.iter(), ", ")?;
                write!(f, ")")
            }
            Value::Udt => todo!(),
        }
    }
}

pub struct ConversionError {
    pub expected: &'static str,
    pub actual: &'static str,
}

impl TryFrom<Value> for i64 {
    type Error = ConversionError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Int(v) = value {
            Ok(v)
        } else {
            Err(ConversionError {
                expected: "Int",
                actual: value.type_name(),
            })
        }
    }
}

impl TryFrom<Value> for bool {
    type Error = ConversionError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Bool(v) = value {
            Ok(v)
        } else {
            Err(ConversionError {
                expected: "Bool",
                actual: value.type_name(),
            })
        }
    }
}

impl TryFrom<Value> for String {
    type Error = ConversionError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::String(v) = value {
            Ok(v)
        } else {
            Err(ConversionError {
                expected: "String",
                actual: value.type_name(),
            })
        }
    }
}

impl Value {
    /// Convert the [Value] into an array of [Value]
    /// # Errors
    /// This will return an error if the [Value] is not a [`Value::Array`].
    pub fn try_into_array(self) -> Result<Vec<Self>, ConversionError> {
        if let Value::Array(v) = self {
            Ok(v)
        } else {
            Err(ConversionError {
                expected: "Array",
                actual: self.type_name(),
            })
        }
    }

    /// Convert the [Value] into an tuple of [Value]
    /// # Errors
    /// This will return an error if the [Value] is not a [`Value::Tuple`].
    pub fn try_into_tuple(self) -> Result<Vec<Self>, ConversionError> {
        if let Value::Tuple(v) = self {
            Ok(v)
        } else {
            Err(ConversionError {
                expected: "Tuple",
                actual: self.type_name(),
            })
        }
    }

    #[must_use]
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Array(_) => "Array",
            Value::BigInt(_) => "BigInt",
            Value::Bool(_) => "Bool",
            Value::Closure(_, _) => "Closure",
            Value::Double(_) => "Double",
            Value::Global(_) => "Global",
            Value::Int(_) => "Int",
            Value::Pauli(_) => "Pauli",
            Value::Qubit(_) => "Qubit",
            Value::Range(_, _, _) => "Range",
            Value::Result(_) => "Result",
            Value::String(_) => "String",
            Value::Tuple(_) => "Tuple",
            Value::Udt => "Udt",
        }
    }

    /// Returns the arithmetic negate of this [`Value`].
    ///
    /// # Errors
    ///
    /// This function will return an error if the value is not an `Int`, `BigInt`, or `Double`.
    pub fn arithmetic_negate(&self) -> Result<Self, ConversionError> {
        match self {
            Value::BigInt(v) => Ok(Value::BigInt(v.neg())),
            Value::Double(v) => Ok(Value::Double(v.neg())),
            Value::Int(v) => Ok(Value::Int(v.wrapping_neg())),
            _ => Err(ConversionError {
                expected: "Int, BigInt, or Double",
                actual: self.type_name(),
            }),
        }
    }
}

fn join<'a>(
    f: &mut std::fmt::Formatter<'_>,
    mut vals: impl Iterator<Item = &'a Value>,
    sep: &str,
) -> std::fmt::Result {
    if let Some(v) = vals.next() {
        v.fmt(f)?;
    }
    for v in vals {
        write!(f, "{sep}")?;
        v.fmt(f)?;
    }
    Ok(())
}