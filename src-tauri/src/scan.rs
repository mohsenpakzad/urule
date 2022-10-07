use crate::region::{CandidateLocations, Region};
use std::str::FromStr;
use winapi::um::winnt::MEMORY_BASIC_INFORMATION;

/// A scan type.
///
/// The variant determines how a memory scan should be performed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scan {
    /// Perform an exact memory scan.
    /// Only memory locations containing this exact value will be considered.
    Exact(i32),
    /// The value is unknown.
    /// Every memory location is considered valid. This only makes sense for a first scan.
    Unknown,
    /// The value is contained within a given range.
    InRange(i32, i32),
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
    DecreasedBy(i32),
    /// The value has increased by the given amount since the last scan.
    /// This only makes sense for subsequent scans.
    IncreasedBy(i32),
}

impl Scan {
    /// Run the scan over the memory corresponding to the given region information.
    ///
    /// Returns a scanned region with all the results found.
    pub fn run(&self, info: MEMORY_BASIC_INFORMATION, memory: Vec<u8>) -> Region {
        let base = info.BaseAddress as usize;
        match *self {
            Scan::Exact(value) => {
                let target = value.to_ne_bytes();
                let locations = memory
                    .windows(target.len())
                    .enumerate()
                    .step_by(4)
                    .flat_map(|(offset, window)| {
                        if window == target {
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
                        .windows(4)
                        .enumerate()
                        .step_by(4)
                        .flat_map(|(offset, window)| {
                            let n =
                                i32::from_ne_bytes([window[0], window[1], window[2], window[3]]);
                            if low <= n && n <= high {
                                Some((base + offset, n))
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
                        .windows(4)
                        .step_by(4)
                        .map(|value| i32::from_ne_bytes([value[0], value[1], value[2], value[3]]))
                        .collect(),
                },
            },
        }
    }

    /// Re-run the scan over a previously-scanned memory region.
    ///
    /// Returns the new scanned region with all the results found.
    pub fn rerun(&self, region: &Region, memory: Vec<u8>) -> Region {
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
                            let bytes = &memory[base..base + 4];
                            let new = i32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                            if new == value {
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
                            let bytes = &memory[base..base + 4];
                            let new = i32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                            println!("Old: {}, new: {}", old, new);
                            if self.acceptable(old, new) {
                                Some((addr, new))
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
    fn acceptable(&self, old: i32, new: i32) -> bool {
        match *self {
            Scan::Exact(n) => new == n,
            Scan::Unknown => true,
            Scan::InRange(low, high) => low <= new && new <= high,
            Scan::Unchanged => new == old,
            Scan::Changed => new != old,
            Scan::Decreased => new < old,
            Scan::Increased => new > old,
            Scan::DecreasedBy(n) => old.wrapping_sub(new) == n,
            Scan::IncreasedBy(n) => new.wrapping_sub(old) == n,
        }
    }
}

impl FromStr for Scan {
    type Err = std::num::ParseIntError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value.as_bytes()[0] {
            b'u' => Scan::Unknown,
            b'=' => Scan::Unchanged,
            b'~' => Scan::Changed,
            t @ b'd' | t @ b'i' => {
                let n = value[1..].trim();
                if n.is_empty() {
                    if t == b'd' {
                        Scan::Decreased
                    } else {
                        Scan::Increased
                    }
                } else {
                    let n = n.parse()?;
                    if t == b'd' {
                        Scan::DecreasedBy(n)
                    } else {
                        Scan::IncreasedBy(n)
                    }
                }
            }
            _ => {
                let (low, high) = if let Some(i) = value.find("..=") {
                    (value[..i].parse()?, value[i + 3..].parse()?)
                } else if let Some(i) = value.find("..") {
                    (value[..i].parse()?, value[i + 2..].parse::<i32>()? - 1)
                } else {
                    let n = value.parse()?;
                    (n, n)
                };

                if low == high {
                    Scan::Exact(low)
                } else {
                    Scan::InRange(low, high)
                }
            }
        })
    }
}

#[cfg(test)]
mod scan_tests {
    use super::*;

    #[test]
    fn exact() {
        assert_eq!("42".parse(), Ok(Scan::Exact(42)));
        assert_eq!("-42".parse(), Ok(Scan::Exact(-42)));
    }

    #[test]
    fn unknown() {
        assert_eq!("u".parse(), Ok(Scan::Unknown));
    }

    #[test]
    fn in_range() {
        assert_eq!("12..34".parse(), Ok(Scan::InRange(12, 33)));
        assert_eq!("12..=34".parse(), Ok(Scan::InRange(12, 34)));
    }

    #[test]
    fn unchanged() {
        assert_eq!("=".parse(), Ok(Scan::Unchanged));
    }

    #[test]
    fn changed() {
        assert_eq!("~".parse(), Ok(Scan::Changed));
    }

    #[test]
    fn decreased() {
        assert_eq!("d".parse(), Ok(Scan::Decreased));
    }

    #[test]
    fn increased() {
        assert_eq!("i".parse(), Ok(Scan::Increased));
    }

    #[test]
    fn decreased_by() {
        assert_eq!("d42".parse(), Ok(Scan::DecreasedBy(42)));
        assert_eq!("d 42".parse(), Ok(Scan::DecreasedBy(42)));
        assert_eq!("d-42".parse(), Ok(Scan::DecreasedBy(-42)));
    }

    #[test]
    fn increased_by() {
        assert_eq!("i42".parse(), Ok(Scan::IncreasedBy(42)));
        assert_eq!("i 42".parse(), Ok(Scan::IncreasedBy(42)));
        assert_eq!("i-42".parse(), Ok(Scan::IncreasedBy(-42)));
    }
}

#[cfg(test)]
mod candidate_location_tests {
    use std::collections::BTreeMap;

    use super::*;

    const VALUE: i32 = 3;
    const VALUES: Vec<i32> = Vec::new();

    #[test]
    fn compact_uncompactable() {
        // Same value
        let mut locations = CandidateLocations::SameValue {
            locations: vec![0x2000],
            value: VALUE,
        };
        locations.try_compact();
        assert!(matches!(locations, CandidateLocations::SameValue { .. }));

        // Range
        let mut locations = CandidateLocations::Range {
            range: 0x2000..0x2100,
            values: VALUES,
        };
        locations.try_compact();
        assert!(matches!(locations, CandidateLocations::Range { .. }));

        // Already compacted
        let mut locations = CandidateLocations::Offsetted {
            base: 0x2000,
            offsets: vec![0, 0x20, 0x40],
            values: VALUES,
        };
        locations.try_compact();
        assert!(matches!(locations, CandidateLocations::Offsetted { .. }));

        let mut locations = CandidateLocations::Masked {
            base: 0x2000,
            mask: vec![true, false, false, false],
            values: VALUES,
        };
        locations.try_compact();
        assert!(matches!(locations, CandidateLocations::Masked { .. }));
    }

    #[test]
    fn compact_not_worth() {
        // Too small
        let mut locations = CandidateLocations::KeyValue(BTreeMap::from([(0x2000, 0)]));
        let original = locations.clone();
        locations.try_compact();
        assert_eq!(locations, original);

        // Too sparse and too large to fit in `Offsetted`.
        let mut locations =
            CandidateLocations::KeyValue(BTreeMap::from([(0x2000, 0), (0x42000, 1)]));
        let original = locations.clone();
        locations.try_compact();
        assert_eq!(locations, original);
    }

    #[test]
    fn compact_offsetted() {
        let mut locations =
            CandidateLocations::KeyValue(BTreeMap::from([(0x2000, 0), (0x2004, 1), (0x2040, 2)]));
        locations.try_compact();
        assert_eq!(
            locations,
            CandidateLocations::Offsetted {
                base: 0x2000,
                offsets: vec![0x0000, 0x0004, 0x0040],
                values: vec![0, 1, 2]
            }
        );
    }

    #[test]
    fn compact_masked() {
        let mut locations = CandidateLocations::KeyValue(BTreeMap::from([
            (0x2000, 0),
            (0x2004, 1),
            (0x200c, 2),
            (0x2010, 3),
            (0x2014, 4),
            (0x2018, 5),
            (0x201c, 6),
            (0x2020, 7),
        ]));
        locations.try_compact();
        assert_eq!(
            locations,
            CandidateLocations::Masked {
                base: 0x2000,
                mask: vec![true, true, false, true, true, true, true, true],
                values: vec![0, 1, 2, 3, 4, 5, 6, 7]
            }
        );
    }

    #[test]
    fn iter_same_value() {
        let locations = CandidateLocations::SameValue {
            locations: vec![0x2000, 0x2004, 0x200c],
            value: VALUE,
        };
        assert_eq!(
            locations.iter().collect::<Vec<_>>(),
            vec![0x2000, 0x2004, 0x200c],
        );
    }

    #[test]
    fn iter_key_value() {
        let locations =
            CandidateLocations::KeyValue(BTreeMap::from([(0x2000, 0), (0x2004, 1), (0x200c, 2)]));
        assert_eq!(
            locations.iter().collect::<Vec<_>>(),
            vec![0x2000, 0x2004, 0x200c],
        );
    }

    #[test]
    fn iter_offsetted() {
        let locations = CandidateLocations::Offsetted {
            base: 0x2000,
            offsets: vec![0x0000, 0x0004, 0x000c],
            values: VALUES,
        };
        assert_eq!(
            locations.iter().collect::<Vec<_>>(),
            vec![0x2000, 0x2004, 0x200c]
        );
    }

    #[test]
    fn iter_range() {
        let locations = CandidateLocations::Range {
            range: 0x2000..0x2010,
            values: VALUES,
        };
        assert_eq!(
            locations.iter().collect::<Vec<_>>(),
            vec![0x2000, 0x2004, 0x2008, 0x200c]
        );
    }

    #[test]
    fn iter_masked() {
        let locations = CandidateLocations::Masked {
            base: 0x2000,
            mask: vec![true, true, false, true],
            values: VALUES,
        };
        assert_eq!(
            locations.iter().collect::<Vec<_>>(),
            vec![0x2000, 0x2004, 0x200c]
        );
    }
}
