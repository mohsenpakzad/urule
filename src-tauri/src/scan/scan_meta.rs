use super::{Scan, Scannable};
use paste::paste;
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

macro_rules! impl_into_scan {
    ($( $type:ty : $type_size:expr ),+ ) => {
        paste!{$(
            impl IntoScan<$type_size, $type> for ScanInfo {
                fn to_scan(self, value_type: &ValueType) -> Option<Scan<$type_size, $type>> {
                     if value_type != &ValueType::[<$type:upper>] {
                        return None;
                    }

                    match self.typ {
                        ScanType::Unknown => return Some(Scan::<$type_size, $type>::Unknown),
                        ScanType::Unchanged => return Some(Scan::<$type_size, $type>::Unchanged),
                        ScanType::Changed => return Some(Scan::<$type_size, $type>::Changed),
                        ScanType::Decreased => return Some(Scan::<$type_size, $type>::Decreased),
                        ScanType::Increased => return Some(Scan::<$type_size, $type>::Increased),
                        _ => (),
                    }

                    if let ScanValue::Exact(exact_val) = self.value {
                        match self.typ {
                            ScanType::Exact => return Some(Scan::<$type_size, $type>::Exact(exact_val.parse().unwrap())),
                            ScanType::DecreasedBy => {
                                return Some(Scan::<$type_size, $type>::DecreasedBy(exact_val.parse().unwrap()))
                            }
                            ScanType::IncreasedBy => {
                                return Some(Scan::<$type_size, $type>::IncreasedBy(exact_val.parse().unwrap()))
                            }
                            _ => (),
                        };
                    } else if let ScanValue::Range { start, end } = self.value {
                        match self.typ {
                            ScanType::InRange => {
                                return Some(Scan::<$type_size, $type>::InRange(
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

        )+}
    };
}

impl_into_scan!(i8: 1, u8: 1, i16: 2, u16:2 , i32: 4, u32: 4, i64: 8, u64: 8);
