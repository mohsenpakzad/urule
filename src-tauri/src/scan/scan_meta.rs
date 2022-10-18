use super::{Scan, Scannable};
use paste::paste;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ScanInfo {
    typ: ScanType,
    value: Option<ScanValue>,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub enum ScanType {
    Exact,
    Unknown,
    InRange,
    SmallerThan,
    BiggerThan,
    Unchanged,
    Changed,
    Decreased,
    Increased,
    DecreasedBy,
    IncreasedBy,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub enum ScanValue {
    Exact(String),
    Range { start: String, end: String },
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub enum ValueType {
    I8,
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

                    if let Some(ScanValue::Exact(exact_val)) = self.value {
                        match self.typ {
                            ScanType::Exact => return Some(Scan::<$type_size, $type>::Exact(exact_val.parse().unwrap())),
                            ScanType::SmallerThan => {
                                return Some(Scan::<$type_size, $type>::SmallerThan(exact_val.parse().unwrap()))
                            }
                            ScanType::BiggerThan => {
                                return Some(Scan::<$type_size, $type>::BiggerThan(exact_val.parse().unwrap()))
                            }
                            ScanType::DecreasedBy => {
                                return Some(Scan::<$type_size, $type>::DecreasedBy(exact_val.parse().unwrap()))
                            }
                            ScanType::IncreasedBy => {
                                return Some(Scan::<$type_size, $type>::IncreasedBy(exact_val.parse().unwrap()))
                            }
                            _ => (),
                        };
                    } else if let Some(ScanValue::Range { start, end }) = self.value {
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

impl_into_scan!(i8: 1, u8: 1, i16: 2, u16:2 , i32: 4, u32: 4, i64: 8, u64: 8, f32: 4, f64: 8);
