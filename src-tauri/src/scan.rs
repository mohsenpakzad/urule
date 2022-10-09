use crate::region::{CandidateLocations, Region};
use std::{borrow::Borrow, cmp::Ordering, str::FromStr};
use winapi::um::winnt::MEMORY_BASIC_INFORMATION;

// TODO: cleanup docs
pub trait Scannable<const SIZE: usize>: Copy {
    // TODO: write document
    fn from_bytes<T: Scannable<SIZE>>(bytes: [u8; SIZE]) -> T;
    /// Returns `true` if the current instance is considered equal to the given chunk of memory.
    ///
    /// Callers must `assert_eq!(left.len(), right.len())`, and the length must also match that of
    /// the length represented by `Self`.
    fn eq(&self, bytes: [u8; SIZE]) -> bool;

    /// Compares `self` to the given chunk of memory.
    ///
    /// Callers must `assert_eq!(left.len(), right.len())`, and the length must also match that of
    /// the length represented by `Self`.
    fn cmp(&self, bytes: [u8; SIZE]) -> Ordering;

    /// Substracts the given chunk of memory from `self`.
    ///
    /// Callers must `assert_eq!(left.len(), right.len())`, and the length must also match that of
    /// the length represented by `Self`.
    fn sub(&mut self, bytes: [u8; SIZE]);

    /// Substracts `self` from the given chunk of memory.
    ///
    /// Callers must `assert_eq!(left.len(), right.len())`, and the length must also match that of
    /// the length represented by `Self`.
    fn rsub(&mut self, bytes: [u8; SIZE]);
}

macro_rules! impl_scannable_for_int {
    ( $( $type:ty : $type_size:expr ),+ ) => {
        $(
            // SAFETY: caller is responsible to `assert_eq!(memory.len(), mem::size_of::<T>())`
            impl Scannable<$type_size> for $type {
                // type Type = $type;

                fn from_bytes<T: Scannable<$type_size>>(bytes: [u8; $type_size]) -> T {
                    unsafe { bytes.as_ptr().cast::<T>().read_unaligned() }
                }

                 fn eq(&self, bytes: [u8; $type_size]) -> bool {
                    let other = <$type>::from_ne_bytes(bytes);
                    *self == other
                }

                 fn cmp(&self, bytes: [u8; $type_size]) -> Ordering {
                    let other = <$type>::from_ne_bytes(bytes);
                    <$type as Ord>::cmp(self, &other)
                }

                fn sub(&mut self, bytes: [u8; $type_size]){
                    todo!()
                }

                fn rsub(&mut self, bytes: [u8; $type_size]){
                    todo!()
                }
            }
        )*
    };
}

impl_scannable_for_int!(i8: 1, u8: 1, i16: 2, u16:2 , i32: 4, u32: 4, i64: 8, u64: 8);

/// A scan type.
///
/// The variant determines how a memory scan should be performed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scan<const SIZE: usize, T: Scannable<SIZE>> {
    /// Perform an exact memory scan.
    /// Only memory locations containing this exact value will be considered.
    Exact(T),
    /// The value is unknown.
    /// Every memory location is considered valid. This only makes sense for a first scan.
    Unknown,
    /// The value is contained within a given range.
    InRange(T, T),
    /// The value has not changed since the last scan.
    /// This only makes sense for subsequent scans.
    Unchanged,
    /// The value has changed since the last scan.
    /// This only makes sense for subsequent scans.
    Changed,
    /// The value has decreased by some unknown amount since the last scan.
    /// This only makes sense for subsequent scans.
    Decreased,
    /// The value has increased by some unknown amount since the last scan.
    /// This only makes sense for subsequent scans.
    Increased,
    /// The value has decreased by the given amount since the last scan.
    /// This only makes sense for subsequent scans.
    DecreasedBy(T),
    /// The value has increased by the given amount since the last scan.
    /// This only makes sense for subsequent scans.
    IncreasedBy(T),
}

impl<const SIZE: usize, T: Scannable<SIZE>> Scan<SIZE, T> {
    /// Run the scan over the memory corresponding to the given region information.
    ///
    /// Returns a scanned region with all the results found.
    pub fn run(&self, info: MEMORY_BASIC_INFORMATION, memory: Vec<u8>) -> Region<SIZE, T> {
        let base = info.BaseAddress as usize;
        match *self {
            Scan::Exact(value) => {
                let locations = memory
                    .windows(SIZE)
                    .enumerate()
                    .step_by(SIZE)
                    .flat_map(|(offset, window)| {
                        if value.eq(window.try_into().unwrap()) {
                            Some(base + offset)
                        } else {
                            None
                        }
                    })
                    .collect();
                Region {
                    info,
                    locations: CandidateLocations::SameValue { locations, value },
                }
            }
            Scan::InRange(low, high) => {
                let mut locations = CandidateLocations::KeyValue(
                    memory
                        .windows(SIZE)
                        .enumerate()
                        .step_by(SIZE)
                        .flat_map(|(offset, window)| {
                            let n: [u8; SIZE] = window.try_into().unwrap();
                            if low.cmp(n) != Ordering::Greater && high.cmp(n) != Ordering::Less {
                                Some((base + offset, T::from_bytes(n))) // TODO
                            } else {
                                None
                            }
                        })
                        .collect(),
                );
                locations.try_compact();

                Region { info, locations }
            }
            // For scans that make no sense on a first run, treat them as unknown.
            Scan::Unknown
            | Scan::Unchanged
            | Scan::Changed
            | Scan::Decreased
            | Scan::Increased
            | Scan::DecreasedBy(_)
            | Scan::IncreasedBy(_) => Region {
                info,
                locations: CandidateLocations::Range {
                    range: base..base + info.RegionSize,
                    values: memory
                        .windows(SIZE)
                        .step_by(SIZE)
                        .map(|value| T::from_bytes(value.try_into().unwrap()))
                        .collect(),
                },
            },
        }
    }

    /// Re-run the scan over a previously-scanned memory region.
    ///
    /// Returns the new scanned region with all the results found.
    pub fn rerun(&self, region: &Region<SIZE, T>, memory: Vec<u8>) -> Region<SIZE, T> {
        match *self {
            // Optimization: unknown scan won't narrow down the region at all.
            Scan::Unknown => region.clone(),
            Scan::Exact(value) => {
                let locations = CandidateLocations::SameValue {
                    locations: region
                        .locations
                        .iter()
                        .flat_map(|addr| {
                            let base = addr - region.info.BaseAddress as usize;
                            let new = memory[base..base + SIZE].borrow().try_into().unwrap();
                            if value.eq(new) {
                                Some(addr)
                            } else {
                                None
                            }
                        })
                        .collect(),
                    value,
                };
                Region {
                    info: region.info.clone(),
                    locations,
                }
            }
            _ => {
                let mut locations = CandidateLocations::KeyValue(
                    region
                        .locations
                        .iter()
                        .flat_map(|addr| {
                            let old = region.value_at(addr);
                            let base = addr - region.info.BaseAddress as usize;
                            let new = memory[base..base + SIZE].borrow().try_into().unwrap();
                            if self.acceptable(old, new) {
                                Some((addr, T::from_bytes(new)))
                            } else {
                                None
                            }
                        })
                        .collect(),
                );
                locations.try_compact();

                Region {
                    info: region.info.clone(),
                    locations,
                }
            }
        }
    }

    /// Check if the change from the given `old` value to the `new` value is acceptable according
    /// to the current scan type.
    ///
    /// # Examples
    ///
    /// ```
    /// let scan = Scan::Increased;
    /// assert!(scan.acceptable(5, 7));
    /// ```
    fn acceptable(&self, old: T, new: [u8; SIZE]) -> bool {
        match *self {
            Scan::Exact(n) => n.eq(new),
            Scan::Unknown => true,
            Scan::InRange(low, high) => {
                // low <= new && new <= high
                low.cmp(new) != Ordering::Greater && high.cmp(new) != Ordering::Less
            }
            Scan::Unchanged => old.eq(new),
            Scan::Changed => !old.eq(new),
            Scan::Decreased => old.cmp(new) == Ordering::Greater,
            Scan::Increased => old.cmp(new) == Ordering::Less,
            _ => true // TODO: implement these variants
            // Scan::DecreasedBy(n) => old.wrapping_sub(new) == n,
            // Scan::IncreasedBy(n) => new.wrapping_sub(old) == n,
        }
    }
}

// TODO:
// impl<const SIZE: usize, T: Scannable<SIZE> + std::str::FromStr> FromStr for Scan<SIZE, T> {
//     type Err = std::num::ParseIntError;

//     fn from_str(value: &str) -> Result<Self, Self::Err> {
//         Ok(match value.as_bytes()[0] {
//             b'u' => Scan::Unknown,
//             b'=' => Scan::Unchanged,
//             b'~' => Scan::Changed,
//             t @ b'd' | t @ b'i' => {
//                 let n = value[1..].trim();
//                 if n.is_empty() {
//                     if t == b'd' {
//                         Scan::Decreased
//                     } else {
//                         Scan::Increased
//                     }
//                 } else {
//                     let n = n.parse()?;
//                     if t == b'd' {
//                         Scan::DecreasedBy(n)
//                     } else {
//                         Scan::IncreasedBy(n)
//                     }
//                 }
//             }
//             _ => {
//                 let (low, high) = if let Some(i) = value.find("..=") {
//                     (value[..i].parse()?, value[i + 3..].parse()?)
//                 } else if let Some(i) = value.find("..") {
//                     (value[..i].parse()?, value[i + 2..].parse::<i32>()? - 1)
//                 } else {
//                     let n = value.parse()?;
//                     (n, n)
//                 };

//                 if low == high {
//                     Scan::Exact(low)
//                 } else {
//                     Scan::InRange(low, high)
//                 }
//             }
//         })
//     }
// }

// #[cfg(test)]
// mod scan_tests {
//     use super::*;

//     #[test]
//     fn exact() {
//         assert_eq!("42".parse(), Ok(Scan::Exact(42)));
//         assert_eq!("-42".parse(), Ok(Scan::Exact(-42)));
//     }

//     #[test]
//     fn unknown() {
//         assert_eq!("u".parse(), Ok(Scan::Unknown));
//     }

//     #[test]
//     fn in_range() {
//         assert_eq!("12..34".parse(), Ok(Scan::InRange(12, 33)));
//         assert_eq!("12..=34".parse(), Ok(Scan::InRange(12, 34)));
//     }

//     #[test]
//     fn unchanged() {
//         assert_eq!("=".parse(), Ok(Scan::Unchanged));
//     }

//     #[test]
//     fn changed() {
//         assert_eq!("~".parse(), Ok(Scan::Changed));
//     }

//     #[test]
//     fn decreased() {
//         assert_eq!("d".parse(), Ok(Scan::Decreased));
//     }

//     #[test]
//     fn increased() {
//         assert_eq!("i".parse(), Ok(Scan::Increased));
//     }

//     #[test]
//     fn decreased_by() {
//         assert_eq!("d42".parse(), Ok(Scan::DecreasedBy(42)));
//         assert_eq!("d 42".parse(), Ok(Scan::DecreasedBy(42)));
//         assert_eq!("d-42".parse(), Ok(Scan::DecreasedBy(-42)));
//     }

//     #[test]
//     fn increased_by() {
//         assert_eq!("i42".parse(), Ok(Scan::IncreasedBy(42)));
//         assert_eq!("i 42".parse(), Ok(Scan::IncreasedBy(42)));
//         assert_eq!("i-42".parse(), Ok(Scan::IncreasedBy(-42)));
//     }
// }
