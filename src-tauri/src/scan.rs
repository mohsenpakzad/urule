use crate::region::{CandidateLocations, Region, Value};
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
            Scan::Exact(n) => {
                let target = n.to_ne_bytes();
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
                    locations: CandidateLocations::Discrete { locations },
                    value: Value::Exact(n),
                }
            }
            Scan::InRange(low, high) => {
                let locations = memory
                    .windows(4)
                    .enumerate()
                    .step_by(4)
                    .flat_map(|(offset, window)| {
                        let n = i32::from_ne_bytes([window[0], window[1], window[2], window[3]]);
                        if low <= n && n <= high {
                            Some(base + offset)
                        } else {
                            None
                        }
                    })
                    .collect();
                Region {
                    info,
                    locations: CandidateLocations::Discrete { locations },
                    value: Value::AnyWithin(memory),
                }
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
                locations: CandidateLocations::Dense {
                    range: base..base + info.RegionSize,
                },
                value: Value::AnyWithin(memory),
            },
        }
    }

    /// Re-run the scan over a previously-scanned memory region.
    ///
    /// Returns the new scanned region with all the results found.
    pub fn rerun(&self, region: &Region, memory: Vec<u8>) -> Region {
        match self {
            // Optimization: unknown scan won't narrow down the region at all.
            Scan::Unknown => region.clone(),
            _ => {
                let mut locations = CandidateLocations::Discrete {
                    locations: region
                        .locations
                        .iter()
                        .flat_map(|addr| {
                            let old = region.value_at(addr);
                            let base = addr - region.info.BaseAddress as usize;
                            let bytes = &memory[base..base + 4];
                            let new = i32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                            if self.acceptable(old, new) {
                                Some(addr)
                            } else {
                                None
                            }
                        })
                        .collect(),
                };
                locations.try_compact();

                Region {
                    info: region.info.clone(),
                    locations,
                    value: Value::AnyWithin(memory),
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
    use super::*;

    #[test]
    fn compact_uncompactable() {
        // Dense
        let mut locations = CandidateLocations::Dense {
            range: 0x2000..0x2100,
        };
        locations.try_compact();
        assert!(matches!(locations, CandidateLocations::Dense { .. }));

        // Already compacted
        let mut locations = CandidateLocations::SmallDiscrete {
            base: 0x2000,
            offsets: vec![0, 0x20, 0x40],
        };
        locations.try_compact();
        assert!(matches!(
            locations,
            CandidateLocations::SmallDiscrete { .. }
        ));

        let mut locations = CandidateLocations::Sparse {
            base: 0x2000,
            mask: vec![true, false, false, false],
        };
        locations.try_compact();
        assert!(matches!(locations, CandidateLocations::Sparse { .. }));
    }

    #[test]
    fn compact_not_worth() {
        // Too small
        let mut locations = CandidateLocations::Discrete {
            locations: vec![0x2000],
        };
        let original = locations.clone();
        locations.try_compact();
        assert_eq!(locations, original);

        // Too sparse and too large to fit in `SmallDiscrete`.
        let mut locations = CandidateLocations::Discrete {
            locations: vec![0x2000, 0x42000],
        };
        let original = locations.clone();
        locations.try_compact();
        assert_eq!(locations, original);
    }

    #[test]
    fn compact_small_discrete() {
        let mut locations = CandidateLocations::Discrete {
            locations: vec![0x2000, 0x2004, 0x2040],
        };
        locations.try_compact();
        assert_eq!(
            locations,
            CandidateLocations::SmallDiscrete {
                base: 0x2000,
                offsets: vec![0x0000, 0x0004, 0x0040],
            }
        );
    }

    #[test]
    fn compact_sparse() {
        let mut locations = CandidateLocations::Discrete {
            locations: vec![
                0x2000, 0x2004, 0x200c, 0x2010, 0x2014, 0x2018, 0x201c, 0x2020,
            ],
        };
        locations.try_compact();
        assert_eq!(
            locations,
            CandidateLocations::Sparse {
                base: 0x2000,
                mask: vec![true, true, false, true, true, true, true, true],
            }
        );
    }

    #[test]
    fn iter_discrete() {
        let locations = CandidateLocations::Discrete {
            locations: vec![0x2000, 0x2004, 0x200c],
        };
        assert_eq!(
            locations.iter().collect::<Vec<_>>(),
            vec![0x2000, 0x2004, 0x200c]
        );
    }

    #[test]
    fn iter_small_discrete() {
        let locations = CandidateLocations::SmallDiscrete {
            base: 0x2000,
            offsets: vec![0x0000, 0x0004, 0x000c],
        };
        assert_eq!(
            locations.iter().collect::<Vec<_>>(),
            vec![0x2000, 0x2004, 0x200c]
        );
    }

    #[test]
    fn iter_dense() {
        let locations = CandidateLocations::Dense {
            range: 0x2000..0x2010,
        };
        assert_eq!(
            locations.iter().collect::<Vec<_>>(),
            vec![0x2000, 0x2004, 0x2008, 0x200c]
        );
    }

    #[test]
    fn iter_sparse() {
        let locations = CandidateLocations::Sparse {
            base: 0x2000,
            mask: vec![true, true, false, true],
        };
        assert_eq!(
            locations.iter().collect::<Vec<_>>(),
            vec![0x2000, 0x2004, 0x200c]
        );
    }
}
