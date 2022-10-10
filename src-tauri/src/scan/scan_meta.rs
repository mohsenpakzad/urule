use super::{Scan, Scannable};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ScanInfo {
    typ: ScanType,
    value: ScanValue,
}

#[derive(PartialEq, Eq, Deserialize)]
pub enum ScanType {
    Exact,
    Unknown,
    InRange,
    Unchanged,
    Changed,
    Decreased,
    Increased,
    DecreasedBy,
    IncreasedBy,
}

#[derive(PartialEq, Eq, Deserialize)]
pub enum ScanValue {
    Exact(String),
    Range { start: String, end: String },
}

#[derive(PartialEq, Eq, Deserialize)]
pub enum ValueType {
    I8 = 0,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    F32,
    F64,
}

pub trait IntoScan<const SIZE: usize, T: Scannable<SIZE>> {
    fn to_scan(self, value_type: &ValueType) -> Option<Scan<SIZE, T>>;
}

impl IntoScan<4, i32> for ScanInfo {
    fn to_scan(self, value_type: &ValueType) -> Option<Scan<4, i32>> {
        if value_type != &ValueType::I32 {
            return None;
        }

        match self.typ {
            ScanType::Unknown => return Some(Scan::<4, i32>::Unknown),
            ScanType::Unchanged => return Some(Scan::<4, i32>::Unchanged),
            ScanType::Changed => return Some(Scan::<4, i32>::Changed),
            ScanType::Decreased => return Some(Scan::<4, i32>::Decreased),
            ScanType::Increased => return Some(Scan::<4, i32>::Increased),
            _ => (),
        }

        if let ScanValue::Exact(exact_val) = self.value {
            match self.typ {
                ScanType::Exact => return Some(Scan::<4, i32>::Exact(exact_val.parse().unwrap())),
                ScanType::DecreasedBy => {
                    return Some(Scan::<4, i32>::DecreasedBy(exact_val.parse().unwrap()))
                }
                ScanType::IncreasedBy => {
                    return Some(Scan::<4, i32>::IncreasedBy(exact_val.parse().unwrap()))
                }
                _ => (),
            };
        } else if let ScanValue::Range { start, end } = self.value {
            match self.typ {
                ScanType::InRange => {
                    return Some(Scan::<4, i32>::InRange(
                        start.parse().unwrap(),
                        end.parse().unwrap(),
                    ))
                }
                _ => (),
            }
        }

        None
    }
}
