pub mod scan_meta;
mod scannable;

use crate::region::{LocationsStyle, Region};
pub use scannable::Scannable;
use std::{borrow::Borrow, cmp::Ordering};
use winapi::um::winnt::MEMORY_BASIC_INFORMATION;

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
                    locations: LocationsStyle::SameValue { locations, value },
                }
            }
            Scan::InRange(low, high) => {
                let mut locations = LocationsStyle::KeyValue(
                    memory
                        .windows(SIZE)
                        .enumerate()
                        .step_by(SIZE)
                        .flat_map(|(offset, window)| {
                            let n: [u8; SIZE] = window.try_into().unwrap();
                            if low.cmp(n) != Ordering::Greater && high.cmp(n) != Ordering::Less {
                                Some((base + offset, T::from_bytes(n)))
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
                locations: LocationsStyle::Range {
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
                let locations = LocationsStyle::SameValue {
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
                let mut locations = LocationsStyle::KeyValue(
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
            Scan::DecreasedBy(n) => n.eq(old.sub(new)),
            Scan::IncreasedBy(n) => {
                let old = old.to_bytes();
                let new = T::from_bytes::<T>(new);
                n.eq(new.sub(old))
            }
        }
    }
}
